use crate::error::{self, Result};
use crate::run::{KnownTestType, TestType};
use bottlerocket_types::agent_config::TufRepoConfig;
use bottlerocket_variant::Variant;
use handlebars::Handlebars;
use log::{debug, info, warn};
use maplit::btreemap;
use pubsys_config::RepoConfig;
use serde::Deserialize;
use snafu::{OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use testsys_config::{rendered_cluster_name, GenericVariantConfig, TestsysImages};
use testsys_model::constants::{API_VERSION, NAMESPACE};
use testsys_model::test_manager::{SelectionParams, TestManager};
use testsys_model::Crd;

/// A type that is used for the creation of all CRDs.
pub struct CrdInput<'a> {
    pub client: &'a TestManager,
    pub arch: String,
    pub variant: Variant,
    pub config: GenericVariantConfig,
    pub repo_config: RepoConfig,
    pub test_flavor: String,
    pub starting_version: Option<String>,
    pub migrate_to_version: Option<String>,
    pub build_id: Option<String>,
    /// `CrdCreator::starting_image_id` function should be used instead of using this field, so
    /// it is not externally visible.
    pub(crate) starting_image_id: Option<String>,
    pub(crate) test_type: TestType,
    pub(crate) tests_directory: PathBuf,
    pub images: TestsysImages,
}

impl<'a> CrdInput<'a> {
    /// Retrieve the TUF repo information from `Infra.toml`
    pub fn tuf_repo_config(&self) -> Option<TufRepoConfig> {
        if let (Some(metadata_base_url), Some(targets_url)) = (
            &self.repo_config.metadata_base_url,
            &self.repo_config.targets_url,
        ) {
            debug!(
                "Using TUF metadata from Infra.toml, metadata: '{}', targets: '{}'",
                metadata_base_url, targets_url
            );
            Some(TufRepoConfig {
                metadata_url: format!("{}{}/{}/", metadata_base_url, &self.variant, &self.arch),
                targets_url: targets_url.to_string(),
            })
        } else {
            warn!("No TUF metadata was found in Infra.toml using the default TUF repos");
            None
        }
    }

    /// Create a set of labels for the CRD by adding `additional_labels` to the standard labels.
    pub fn labels(&self, additional_labels: BTreeMap<String, String>) -> BTreeMap<String, String> {
        let mut labels = btreemap! {
            "testsys/arch".to_string() => self.arch.to_string(),
            "testsys/variant".to_string() => self.variant.to_string(),
            "testsys/build-id".to_string() => self.build_id.to_owned().unwrap_or_default(),
            "testsys/test-type".to_string() => self.test_type.to_string(),
        };
        let mut add_labels = additional_labels;
        labels.append(&mut add_labels);
        labels
    }

    /// Determine all CRDs that have the same value for each `id_labels` as `labels`.
    pub async fn existing_crds(
        &self,
        labels: &BTreeMap<String, String>,
        id_labels: &[&str],
    ) -> Result<Vec<String>> {
        // Create a single string containing all `label=value` pairs.
        let checks = id_labels
            .iter()
            .map(|label| {
                labels
                    .get(&label.to_string())
                    .map(|value| format!("{}={}", label, value))
                    .context(error::InvalidSnafu {
                        what: format!("The label '{}' was missing", label),
                    })
            })
            .collect::<Result<Vec<String>>>()?
            .join(",");

        // Create a list of all CRD names that match all of the specified labels.
        Ok(self
            .client
            .list(&SelectionParams {
                labels: Some(checks),
                ..Default::default()
            })
            .await?
            .iter()
            .filter_map(Crd::name)
            .collect())
    }

    /// Use the provided userdata path to create the encoded userdata.
    pub fn encoded_userdata(&self) -> Result<Option<String>> {
        let userdata_path = match self.config.userdata.as_ref() {
            Some(userdata) => self.custom_userdata_file_path(userdata)?,
            None => return Ok(None),
        };

        info!("Using userdata at '{}'", userdata_path.display());

        let userdata = std::fs::read_to_string(&userdata_path).context(error::FileSnafu {
            path: userdata_path,
        })?;

        Ok(Some(base64::encode(userdata)))
    }

    /// Find the userdata file for the test type
    fn custom_userdata_file_path(&self, userdata: &str) -> Result<PathBuf> {
        let test_type = &self.test_type.to_string();

        // List all acceptable paths to the custom crd to allow users some freedom in the way
        // `tests` is organized.
        let acceptable_paths = vec![
            // Check the absolute path
            userdata.into(),
            // Check for <TESTSYS_FOLDER>/<TEST-TYPE>/<USERDATA>
            self.tests_directory.join(test_type).join(userdata),
            // Check for <TESTSYS_FOLDER>/<TEST-TYPE>/<USERDATA>.toml
            self.tests_directory
                .join(test_type)
                .join(userdata)
                .with_extension("toml"),
            // Check for <TESTSYS_FOLDER>/shared/<USERDATA>
            self.tests_directory.join("shared").join(userdata),
            // Check for <TESTSYS_FOLDER>/shared/<USERDATA>.toml
            self.tests_directory
                .join("shared")
                .join(userdata)
                .with_extension("toml"),
            // Check for <TESTSYS_FOLDER>/shared/userdata/<USERDATA>
            self.tests_directory
                .join("shared")
                .join("userdata")
                .join(userdata),
            // Check for <TESTSYS_FOLDER>/shared/userdata/<USERDATA>.toml
            self.tests_directory
                .join("shared")
                .join("userdata")
                .join(userdata)
                .with_extension("toml"),
            // Check for the path in the top level directory
            PathBuf::new().join(userdata),
        ];

        // Find the first acceptable path that exists and return that.
        acceptable_paths
            .into_iter()
            .find(|path| path.exists())
            .context(error::InvalidSnafu {
                what: format!(
                    "Could not find userdata '{}' for test type '{}'",
                    userdata, test_type
                ),
            })
    }

    /// Fill in the templated cluster name with `arch` and `variant`.
    fn rendered_cluster_name(&self, raw_cluster_name: String) -> Result<String> {
        Ok(rendered_cluster_name(
            raw_cluster_name,
            self.kube_arch(),
            self.kube_variant(),
        )?)
    }

    /// Get the k8s safe architecture name
    fn kube_arch(&self) -> String {
        self.arch.replace('_', "-")
    }

    /// Get the k8s safe variant name
    fn kube_variant(&self) -> String {
        self.variant.to_string().replace('.', "")
    }

    /// Bottlerocket cluster naming convention.
    fn default_cluster_name(&self) -> String {
        format!("{}-{}", self.kube_arch(), self.kube_variant())
    }

    /// Get a list of cluster_names for this variant. If there are no cluster names, the default
    /// cluster name will be used.
    fn cluster_names(&self) -> Result<Vec<String>> {
        Ok(if self.config.cluster_names.is_empty() {
            vec![self.default_cluster_name()]
        } else {
            self.config
                .cluster_names
                .iter()
                .map(String::to_string)
                // Fill the template fields in the clusters name before using it.
                .map(|cluster_name| self.rendered_cluster_name(cluster_name))
                .collect::<Result<Vec<String>>>()?
        })
    }

    /// Creates a `BTreeMap` of all configurable fields from this input
    fn config_fields(&self, cluster_name: &str) -> BTreeMap<String, String> {
        btreemap! {
            "arch".to_string() => self.arch.clone(),
            "variant".to_string() => self.variant.to_string(),
            "kube-arch".to_string() => self.kube_arch(),
            "kube-variant".to_string() => self.kube_variant(),
            "flavor".to_string() => some_or_null(&self.variant.variant_flavor().map(str::to_string)),
            "version".to_string() => some_or_null(&self.variant.version().map(str::to_string)),
            "cluster-name".to_string() => cluster_name.to_string(),
            "instance-type".to_string() => some_or_null(&self.config.instance_type),
            "agent-role".to_string() => some_or_null(&self.config.agent_role),
            "conformance-image".to_string() => some_or_null(&self.config.conformance_image),
            "conformance-registry".to_string() => some_or_null(&self.config.conformance_registry),
            "control-plane-endpoint".to_string() => some_or_null(&self.config.control_plane_endpoint),
        }
    }

    /// Find the crd template file for the given test type
    fn custom_crd_template_file_path(&self) -> Option<PathBuf> {
        let test_type = &self.test_type.to_string();
        // List all acceptable paths to the custom crd to allow users some freedom in the way
        // `tests` is organized.
        let acceptable_paths = vec![
            // Check for <TEST-TYPE>.yaml in the top level directory
            PathBuf::new().join(test_type).with_extension("yaml"),
            // Check for <TESTSYS_FOLDER>/<TEST-TYPE>/<TEST-TYPE>.yaml
            self.tests_directory
                .join(test_type)
                .join(test_type)
                .with_extension("yaml"),
            // Check for <TESTSYS_FOLDER>/<TEST-TYPE>/crd.yaml
            self.tests_directory.join(test_type).join("crd.yaml"),
            // Check for <TESTSYS_FOLDER>/shared/<TEST-TYPE>.yaml
            self.tests_directory
                .join("shared")
                .join(test_type)
                .with_extension("yaml"),
            // Check for <TESTSYS_FOLDER>/shared/tests/<TEST-TYPE>.yaml
            self.tests_directory
                .join("shared")
                .join("tests")
                .join(test_type)
                .with_extension("yaml"),
        ];

        // Find the first acceptable path that exists and return that.
        acceptable_paths.into_iter().find(|path| path.exists())
    }

    /// Find the cluster config file for the given cluster name and test type.
    fn cluster_config_file_path(&self, cluster_name: &str) -> Option<PathBuf> {
        let test_type = &self.test_type.to_string();
        // List all acceptable paths to the custom crd to allow users some freedom in the way
        // `tests` is organized.
        let acceptable_paths = vec![
            // Check for <TESTSYS_FOLDER>/<TEST-TYPE>/<CLUSTER-NAME>.yaml
            self.tests_directory
                .join(test_type)
                .join(cluster_name)
                .with_extension("yaml"),
            // Check for <TESTSYS_FOLDER>/shared/<CLUSTER-NAME>.yaml
            self.tests_directory
                .join("shared")
                .join(cluster_name)
                .with_extension("yaml"),
            // Check for <TESTSYS_FOLDER>/shared/cluster-config/<CLUSTER-NAME>.yaml
            self.tests_directory
                .join("shared")
                .join("cluster-config")
                .join(cluster_name)
                .with_extension("yaml"),
            // Check for <TESTSYS_FOLDER>/shared/clusters/<CLUSTER-NAME>.yaml
            self.tests_directory
                .join("shared")
                .join("clusters")
                .join(cluster_name)
                .with_extension("yaml"),
            // Check for <TESTSYS_FOLDER>/shared/clusters/<CLUSTER-NAME>/cluster.yaml
            self.tests_directory
                .join("shared")
                .join("clusters")
                .join(cluster_name)
                .join("cluster")
                .with_extension("yaml"),
        ];

        // Find the first acceptable path that exists and return that.
        acceptable_paths.into_iter().find(|path| path.exists())
    }

    /// Find the resolved cluster config file for the given cluster name and test type if it exists.
    fn resolved_cluster_config(
        &self,
        cluster_name: &str,
        additional_fields: &mut BTreeMap<String, String>,
    ) -> Result<Option<String>> {
        let path = match self.cluster_config_file_path(cluster_name) {
            None => return Ok(None),
            Some(path) => path,
        };
        info!("Using cluster config at {}", path.display());
        let config = fs::read_to_string(&path).context(error::FileSnafu { path })?;

        let mut fields = self.config_fields(cluster_name);
        fields.insert("api-version".to_string(), API_VERSION.to_string());
        fields.insert("namespace".to_string(), NAMESPACE.to_string());
        fields.append(additional_fields);

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let rendered_config = handlebars.render_template(&config, &fields)?;

        Ok(Some(rendered_config))
    }

    /// Find the hardware csv file for the given hardware csv name and test type.
    fn hardware_csv_file_path(&self, hardware_csv: &str) -> Option<PathBuf> {
        let test_type = &self.test_type.to_string();
        // List all acceptable paths to the custom crd to allow users some freedom in the way
        // `tests` is organized.
        let acceptable_paths = vec![
            // Check for <TESTSYS_FOLDER>/<TEST-TYPE>/<HARDWARE_CSV>.csv
            self.tests_directory
                .join(test_type)
                .join(hardware_csv)
                .with_extension("csv"),
            // Check for <TESTSYS_FOLDER>/shared/<HARDWARE_CSV>.csv
            self.tests_directory
                .join("shared")
                .join(hardware_csv)
                .with_extension("csv"),
            // Check for <TESTSYS_FOLDER>/shared/cluster-config/<HARDWARE_CSV>.csv
            self.tests_directory
                .join("shared")
                .join("cluster-config")
                .join(hardware_csv)
                .with_extension("csv"),
            // Check for <TESTSYS_FOLDER>/shared/clusters/<HARDWARE_CSV>.csv
            self.tests_directory
                .join("shared")
                .join("clusters")
                .join(hardware_csv)
                .with_extension("csv"),
        ];

        // Find the first acceptable path that exists and return that.
        acceptable_paths.into_iter().find(|path| path.exists())
    }

    /// Find the resolved cluster config file for the given cluster name and test type if it exists.
    fn resolved_hardware_csv(&self) -> Result<Option<String>> {
        let hardware_csv = match &self.config.hardware_csv {
            Some(hardware_csv) => hardware_csv,
            None => return Ok(None),
        };

        // If the hardware csv is csv like, it probably is a csv; otherwise, it is a path to the
        // hardware csv.
        if hardware_csv.contains(',') {
            return Ok(Some(hardware_csv.to_string()));
        }

        let path = match self.hardware_csv_file_path(hardware_csv) {
            None => return Ok(None),
            Some(path) => path,
        };

        info!("Using hardware csv at {}", path.display());

        let config = fs::read_to_string(&path).context(error::FileSnafu { path })?;
        Ok(Some(config))
    }

    fn hardware_for_cluster(&self, cluster_name: &str) -> Result<Option<String>> {
        // Check for <TESTSYS_FOLDER>/shared/clusters/<CLUSTER-NAME>/hardware.csv
        let path = self
            .tests_directory
            .join("shared")
            .join("clusters")
            .join(cluster_name)
            .join("hardware")
            .with_extension("csv");

        if !path.exists() {
            return Ok(None);
        }

        info!("Using hardware csv at {}", path.display());

        let config = fs::read_to_string(&path).context(error::FileSnafu { path })?;
        Ok(Some(config))
    }
}

/// Take the value of the `Option` or `"null"` if the `Option` was `None`
fn some_or_null(field: &Option<String>) -> String {
    field.to_owned().unwrap_or_else(|| "null".to_string())
}

/// The `CrdCreator` trait is used to create CRDs. Each variant family should have a `CrdCreator`
/// that is responsible for creating the CRDs needed for testing.
#[async_trait::async_trait]
pub(crate) trait CrdCreator: Sync {
    /// Return the image id that should be used for normal testing.
    async fn image_id(&self, crd_input: &CrdInput) -> Result<String>;

    /// Return the image id that should be used as the starting point for migration testing.
    async fn starting_image_id(&self, crd_input: &CrdInput) -> Result<String>;

    /// Create a CRD for the cluster needed to launch Bottlerocket. If no cluster CRD is
    /// needed, `CreateCrdOutput::None` can be returned.
    async fn cluster_crd<'a>(&self, cluster_input: ClusterInput<'a>) -> Result<CreateCrdOutput>;

    /// Create a CRD to launch Bottlerocket. `CreateCrdOutput::None` can be returned if this CRD is
    /// not needed.
    async fn bottlerocket_crd<'a>(
        &self,
        bottlerocket_input: BottlerocketInput<'a>,
    ) -> Result<CreateCrdOutput>;

    /// Create a CRD that migrates Bottlerocket from one version to another.
    async fn migration_crd<'a>(
        &self,
        migration_input: MigrationInput<'a>,
    ) -> Result<CreateCrdOutput>;

    /// Create a testing CRD for this variant of Bottlerocket.
    async fn test_crd<'a>(&self, test_input: TestInput<'a>) -> Result<CreateCrdOutput>;

    /// Create a workload testing CRD for this variant of Bottlerocket.
    async fn workload_crd<'a>(&self, test_input: TestInput<'a>) -> Result<CreateCrdOutput>;

    /// Create a set of additional fields that may be used by an externally defined agent on top of
    /// the ones in `CrdInput`
    fn additional_fields(&self, _test_type: &str) -> BTreeMap<String, String> {
        Default::default()
    }

    /// Creates a set of CRDs for the specified variant and test type that can be added to a TestSys
    /// cluster.
    async fn create_crds(
        &self,
        test_type: &KnownTestType,
        crd_input: &CrdInput,
    ) -> Result<Vec<Crd>> {
        let mut crds = Vec::new();
        let image_id = match &test_type {
            KnownTestType::Migration => {
                if let Some(image_id) = &crd_input.starting_image_id {
                    debug!(
                        "Using the provided starting image id for migration testing '{}'",
                        image_id
                    );
                    image_id.to_string()
                } else {
                    let image_id = self.starting_image_id(crd_input).await?;
                    debug!(
                        "A starting image id was not provided, '{}' will be used instead.",
                        image_id
                    );
                    image_id
                }
            }
            _ => self.image_id(crd_input).await?,
        };
        for cluster_name in &crd_input.cluster_names()? {
            let cluster_output = self
                .cluster_crd(ClusterInput {
                    cluster_name,
                    image_id: &image_id,
                    crd_input,
                    cluster_config: &crd_input.resolved_cluster_config(
                        cluster_name,
                        &mut self
                            .additional_fields(&test_type.to_string())
                            .into_iter()
                            // Add the image id in case it is needed for cluster creation
                            .chain(Some(("image-id".to_string(), image_id.clone())).into_iter())
                            .collect::<BTreeMap<String, String>>(),
                    )?,
                    hardware_csv: &crd_input
                        .resolved_hardware_csv()
                        .transpose()
                        .or_else(|| crd_input.hardware_for_cluster(cluster_name).transpose())
                        .transpose()?,
                })
                .await?;
            let cluster_crd_name = cluster_output.crd_name();
            if let Some(crd) = cluster_output.crd() {
                debug!("Cluster crd was created for '{}'", cluster_name);
                crds.push(crd)
            }
            let bottlerocket_output = self
                .bottlerocket_crd(BottlerocketInput {
                    cluster_crd_name: &cluster_crd_name,
                    image_id: image_id.clone(),
                    test_type,
                    crd_input,
                })
                .await?;
            let bottlerocket_crd_name = bottlerocket_output.crd_name();
            match &test_type {
                KnownTestType::Conformance | KnownTestType::Quick => {
                    if let Some(crd) = bottlerocket_output.crd() {
                        debug!("Bottlerocket crd was created for '{}'", cluster_name);
                        crds.push(crd)
                    }
                    let test_output = self
                        .test_crd(TestInput {
                            cluster_crd_name: &cluster_crd_name,
                            bottlerocket_crd_name: &bottlerocket_crd_name,
                            test_type,
                            crd_input,
                            prev_tests: Default::default(),
                            name_suffix: None,
                        })
                        .await?;
                    if let Some(crd) = test_output.crd() {
                        crds.push(crd)
                    }
                }
                KnownTestType::Workload => {
                    if let Some(crd) = bottlerocket_output.crd() {
                        debug!("Bottlerocket crd was created for '{}'", cluster_name);
                        crds.push(crd)
                    }
                    let test_output = self
                        .workload_crd(TestInput {
                            cluster_crd_name: &cluster_crd_name,
                            bottlerocket_crd_name: &bottlerocket_crd_name,
                            test_type,
                            crd_input,
                            prev_tests: Default::default(),
                            name_suffix: None,
                        })
                        .await?;
                    if let Some(crd) = test_output.crd() {
                        crds.push(crd)
                    }
                }
                KnownTestType::Migration => {
                    if let Some(crd) = bottlerocket_output.crd() {
                        debug!("Bottlerocket crd was created for '{}'", cluster_name);
                        crds.push(crd)
                    }
                    let mut tests = Vec::new();
                    let test_output = self
                        .test_crd(TestInput {
                            cluster_crd_name: &cluster_crd_name,
                            bottlerocket_crd_name: &bottlerocket_crd_name,
                            test_type,
                            crd_input,
                            prev_tests: tests.clone(),
                            name_suffix: "1-initial".into(),
                        })
                        .await?;
                    if let Some(name) = test_output.crd_name() {
                        tests.push(name)
                    }
                    if let Some(crd) = test_output.crd() {
                        crds.push(crd)
                    }
                    let migration_output = self
                        .migration_crd(MigrationInput {
                            cluster_crd_name: &cluster_crd_name,
                            bottlerocket_crd_name: &bottlerocket_crd_name,
                            crd_input,
                            prev_tests: tests.clone(),
                            name_suffix: "2-migrate".into(),
                            migration_direction: MigrationDirection::Upgrade,
                        })
                        .await?;
                    if let Some(name) = migration_output.crd_name() {
                        tests.push(name)
                    }
                    if let Some(crd) = migration_output.crd() {
                        crds.push(crd)
                    }
                    let test_output = self
                        .test_crd(TestInput {
                            cluster_crd_name: &cluster_crd_name,
                            bottlerocket_crd_name: &bottlerocket_crd_name,
                            test_type,
                            crd_input,
                            prev_tests: tests.clone(),
                            name_suffix: "3-migrated".into(),
                        })
                        .await?;
                    if let Some(name) = test_output.crd_name() {
                        tests.push(name)
                    }
                    if let Some(crd) = test_output.crd() {
                        crds.push(crd)
                    }
                    let migration_output = self
                        .migration_crd(MigrationInput {
                            cluster_crd_name: &cluster_crd_name,
                            bottlerocket_crd_name: &bottlerocket_crd_name,
                            crd_input,
                            prev_tests: tests.clone(),
                            name_suffix: "4-migrate".into(),
                            migration_direction: MigrationDirection::Downgrade,
                        })
                        .await?;
                    if let Some(name) = migration_output.crd_name() {
                        tests.push(name)
                    }
                    if let Some(crd) = migration_output.crd() {
                        crds.push(crd)
                    }
                    let test_output = self
                        .test_crd(TestInput {
                            cluster_crd_name: &cluster_crd_name,
                            bottlerocket_crd_name: &bottlerocket_crd_name,
                            test_type,
                            crd_input,
                            prev_tests: tests,
                            name_suffix: "5-final".into(),
                        })
                        .await?;
                    if let Some(crd) = test_output.crd() {
                        crds.push(crd)
                    }
                }
            }
        }

        Ok(crds)
    }

    /// Creates a set of CRDs for the specified variant and test type that can be added to a TestSys
    /// cluster.
    async fn create_custom_crds(
        &self,
        test_type: &str,
        crd_input: &CrdInput,
        override_crd_template: Option<PathBuf>,
    ) -> Result<Vec<Crd>> {
        debug!("Creating custom CRDs for '{}' test", test_type);
        let crd_template_file_path = &override_crd_template
            .or_else(|| crd_input.custom_crd_template_file_path())
            .context(error::InvalidSnafu {
                what: format!(
                    "A custom yaml file could not be found for test type '{}'",
                    test_type
                ),
            })?;
        info!(
            "Creating custom crd from '{}'",
            crd_template_file_path.display()
        );
        let mut crds = Vec::new();
        for cluster_name in &crd_input.cluster_names()? {
            let mut fields = crd_input.config_fields(cluster_name);
            fields.insert("api-version".to_string(), API_VERSION.to_string());
            fields.insert("namespace".to_string(), NAMESPACE.to_string());
            fields.insert("image-id".to_string(), self.image_id(crd_input).await?);
            fields.append(&mut self.additional_fields(test_type));

            let mut handlebars = Handlebars::new();
            handlebars.set_strict_mode(true);
            let rendered_manifest = handlebars.render_template(
                &std::fs::read_to_string(crd_template_file_path).context(error::FileSnafu {
                    path: crd_template_file_path,
                })?,
                &fields,
            )?;

            for crd_doc in serde_yaml::Deserializer::from_str(&rendered_manifest) {
                let value =
                    serde_yaml::Value::deserialize(crd_doc).context(error::SerdeYamlSnafu {
                        what: "Unable to deserialize rendered manifest",
                    })?;
                let mut crd: Crd =
                    serde_yaml::from_value(value).context(error::SerdeYamlSnafu {
                        what: "The manifest did not match a `CRD`",
                    })?;
                // Add in the secrets from the config manually.
                match &mut crd {
                    Crd::Test(test) => {
                        test.spec.agent.secrets = Some(crd_input.config.secrets.clone())
                    }
                    Crd::Resource(resource) => {
                        resource.spec.agent.secrets = Some(crd_input.config.secrets.clone())
                    }
                }
                crds.push(crd);
            }
        }
        Ok(crds)
    }
}

/// The input used for cluster crd creation
pub struct ClusterInput<'a> {
    pub cluster_name: &'a String,
    pub image_id: &'a String,
    pub crd_input: &'a CrdInput<'a>,
    pub cluster_config: &'a Option<String>,
    pub hardware_csv: &'a Option<String>,
}

/// The input used for bottlerocket crd creation
pub struct BottlerocketInput<'a> {
    pub cluster_crd_name: &'a Option<String>,
    /// The image id that should be used by this CRD
    pub image_id: String,
    pub test_type: &'a KnownTestType,
    pub crd_input: &'a CrdInput<'a>,
}

/// The input used for test crd creation
pub struct TestInput<'a> {
    pub cluster_crd_name: &'a Option<String>,
    pub bottlerocket_crd_name: &'a Option<String>,
    pub test_type: &'a KnownTestType,
    pub crd_input: &'a CrdInput<'a>,
    /// The set of tests that have already been created that are related to this test
    pub prev_tests: Vec<String>,
    /// The suffix that should be appended to the end of the test name to prevent naming conflicts
    pub name_suffix: Option<&'a str>,
}

/// The input used for migration crd creation
pub struct MigrationInput<'a> {
    pub cluster_crd_name: &'a Option<String>,
    pub bottlerocket_crd_name: &'a Option<String>,
    pub crd_input: &'a CrdInput<'a>,
    /// The set of tests that have already been created that are related to this test
    pub prev_tests: Vec<String>,
    /// The suffix that should be appended to the end of the test name to prevent naming conflicts
    pub name_suffix: Option<&'a str>,
    pub migration_direction: MigrationDirection,
}

pub enum MigrationDirection {
    Upgrade,
    Downgrade,
}

pub enum CreateCrdOutput {
    /// A new CRD was created and needs to be applied to the cluster.
    NewCrd(Box<Crd>),
    /// An existing CRD is already representing this object.
    ExistingCrd(String),
    /// There is no CRD to create for this step of this family.
    None,
}

impl Default for CreateCrdOutput {
    fn default() -> Self {
        Self::None
    }
}

impl CreateCrdOutput {
    /// Get the name of the CRD that was created or already existed
    pub(crate) fn crd_name(&self) -> Option<String> {
        match self {
            CreateCrdOutput::NewCrd(crd) => {
                Some(crd.name().expect("A CRD is missing the name field."))
            }
            CreateCrdOutput::ExistingCrd(name) => Some(name.to_string()),
            CreateCrdOutput::None => None,
        }
    }

    /// Get the CRD if it was created
    pub(crate) fn crd(self) -> Option<Crd> {
        match self {
            CreateCrdOutput::NewCrd(crd) => Some(*crd),
            CreateCrdOutput::ExistingCrd(_) => None,
            CreateCrdOutput::None => None,
        }
    }
}
