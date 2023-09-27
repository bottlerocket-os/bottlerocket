use super::{error, Result};
use serde::{self, Deserialize};
use snafu::ResultExt;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

// Signature to look for in the spec files that could refer to local `sources`.
const LOCAL_SPEC_PREFIX: &str = "Requires: %{_cross_os}";

/// Simple lookup cache for finding source packages by name or path.
pub(crate) struct SourcePackageCache {
    source_map: HashMap<String, String>,
    package_cache: HashMap<String, CrateInfo>,
}

impl SourcePackageCache {
    /// Get the source package information.
    pub(crate) fn get(&self, package: &String) -> Option<CrateInfo> {
        if let Some(path) = self.source_map.get(package) {
            if let Some(info) = self.package_cache.get(path) {
                return Some(info.clone());
            }
        }

        None
    }

    /// Adds a new source package to the cache.
    fn add(&mut self, package_name: String, package_path: String, info: CrateInfo) {
        self.source_map
            .insert(package_name.clone(), package_path.clone());
        self.package_cache.insert(package_path.clone(), info);
    }
}

/// SourceProcessor manages extracting package and path information from the local source code.
pub(crate) struct SourceProcessor {
    pub(crate) repo_root: PathBuf,
}

impl SourceProcessor {
    /// Generate a source package cache.
    pub(crate) fn source_cache(&self) -> SourcePackageCache {
        // Read source package information to create a name lookup to match those used in the spec files
        let source_path = self.repo_root.as_path().join("sources");
        let mut source_cache = SourcePackageCache {
            source_map: HashMap::new(),
            package_cache: HashMap::new(),
        };

        get_source_paths(&source_path, &mut source_cache, None);
        source_cache
    }

    /// Process variant info to find all local paths that make up a variant.
    pub(crate) fn process_variant(
        &self,
        variant_path: PathBuf,
        source_cache: &mut SourcePackageCache,
    ) -> Result<CrateInfo> {
        let mut variant_info = get_crate_info(&variant_path)?;
        for dep in variant_info.local_dependency_paths(false) {
            let path = Path::new(&dep).to_path_buf();
            get_source_paths(&path, source_cache, Some(&mut variant_info));
        }
        Ok(variant_info)
    }
}

fn get_package_sources(source_path: &Path) -> HashSet<String> {
    let mut results: HashSet<String> = HashSet::new();

    // Check if there is a spec file for this package
    let package_name = source_path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .into_owned();
    let spec_file_name = format!("{}.spec", package_name);
    let rpm_spec = source_path.join(&spec_file_name);

    // Parse the spec file for anything that looks like a reference to our local sources
    let spec_file = File::open(&rpm_spec);
    if spec_file.is_err() {
        // Either the file doesn't exist or we couldn't read it for some reason - just continue on
        return results;
    }

    let spec_file = spec_file.unwrap();
    let content = BufReader::new(spec_file).lines();
    for line in content {
        let spec_line = line.unwrap_or_default();
        if spec_line.starts_with(LOCAL_SPEC_PREFIX) {
            // Looks like this could be a requirement link to our local source, add it to the list to check
            results.insert(spec_line.replace(LOCAL_SPEC_PREFIX, ""));
        }
    }

    results
}

fn get_source_paths(
    source_path: &Path,
    source_cache: &mut SourcePackageCache,
    parent: Option<&mut CrateInfo>,
) {
    // Skip some known folders and paths where we know we don't care
    if source_path.ends_with("archived")
        || source_path.ends_with("vendor")
        || source_path.ends_with("target/debug")
    {
        return;
    }

    // Only process Cargo.toml info if this isn't the root `sources` directory
    if !source_path.ends_with("sources") {
        // Parse and process the crate info to find all source paths
        let cargo = source_path.join("Cargo.toml");
        if cargo.exists() {
            let source_path_string = source_path.to_string_lossy().into_owned();
            if let Some(info) = source_cache.get(&source_path_string) {
                if let Some(parent_crate) = parent {
                    // Copy out the local path information into the parent crate
                    for path in info.local_dependency_paths(true) {
                        parent_crate.add_dependency_path(path.to_string());
                    }
                }
            } else if let Ok(mut info) = get_crate_info(&cargo) {
                // Go through all dependencies and make sure we collect all paths
                for dep in info.local_dependency_paths(false) {
                    let path = Path::new(&dep).to_path_buf();
                    get_source_paths(&path, source_cache, Some(&mut info));
                }

                // Make sure current path is added to list of sources
                info.add_dependency_path(source_path_string.clone());

                // Add the collected information to the parent
                if let Some(parent_crate) = parent {
                    // But first check if we have any local source paths to add
                    let sources = get_package_sources(source_path);
                    for source_name in sources {
                        if let Some(info) = source_cache.get(&source_name) {
                            // Matches one of our local packages, add its paths
                            for path in info.local_dependency_paths(true) {
                                parent_crate.add_dependency_path(path.clone());
                            }
                        }
                    }

                    // Bubble up the current dependency paths into the parent
                    for path in info.local_dependency_paths(true) {
                        parent_crate.add_dependency_path(path.to_string());
                    }
                }
                source_cache.add(info.package.name.clone(), source_path_string.clone(), info);
            }
            return;
        }
    }

    // Walk the directory tree
    if let Ok(items) = source_path.read_dir() {
        for item in items.flatten() {
            let path = item.path();
            if path.is_dir() {
                get_source_paths(&path, source_cache, None);
            }
        }
    }
}

fn get_crate_info(cargo_path: &Path) -> Result<CrateInfo> {
    let cargo_data = fs::read_to_string(cargo_path)
        .context(error::CargoReadFailureSnafu { path: cargo_path })?;
    let mut cargo_toml = toml::from_str::<CrateInfo>(&cargo_data)
        .context(error::CargoParseFailureSnafu { path: cargo_path })?;
    cargo_toml.root_path = cargo_path.parent().unwrap().to_string_lossy().into_owned();
    Ok(cargo_toml)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CrateInfo {
    package: Package,
    #[serde(default)]
    build_dependencies: HashMap<String, DependencyInfo>,
    #[serde(default)]
    dependencies: HashMap<String, DependencyInfo>,
    #[serde(default)]
    dev_dependencies: HashMap<String, DependencyInfo>,
    #[serde(skip, default)]
    root_path: String,
    #[serde(skip, default)]
    paths: HashSet<String>,
}

impl CrateInfo {
    fn add_dependency_path(&mut self, path: String) {
        self.paths.insert(path);
    }

    #[allow(clippy::collapsible_match)]
    fn collect_paths(
        &self,
        dependencies: &HashMap<String, DependencyInfo>,
        paths: &mut HashSet<String>,
    ) {
        for info in dependencies.values() {
            if let DependencyInfo::Dependency { path } = info {
                if let Some(p) = path {
                    if p != "." {
                        let mut path = p.clone();
                        // Handle relative paths and things like storewolf:merge-toml
                        if p.contains("..") || !p.contains('/') {
                            // Relative path, change to full path
                            let full_path =
                                Path::new(&self.root_path).join(p).canonicalize().unwrap();
                            path = full_path.to_string_lossy().into_owned();
                        }
                        paths.insert(path);
                    }
                }
            }
        }
    }

    pub(crate) fn local_dependency_paths(&self, include_package_root: bool) -> HashSet<String> {
        let mut result: HashSet<String> = HashSet::new();

        // Get all paths from our dependencies
        self.collect_paths(&self.build_dependencies, &mut result);
        self.collect_paths(&self.dependencies, &mut result);
        self.collect_paths(&self.dev_dependencies, &mut result);

        // Include any added (transient dependency paths)
        for path in &self.paths {
            result.insert(path.clone());
        }

        // See if we should include the root path
        if include_package_root {
            result.insert(self.root_path.clone());
        }

        result
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Package {
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum DependencyInfo {
    Version(String),
    Dependency { path: Option<String> },
}
