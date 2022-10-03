use coldsnap::SnapshotUploader;
use indicatif::{ProgressBar, ProgressStyle};
use snafu::{OptionExt, ResultExt};
use std::path::Path;

/// Create a progress bar to show status of snapshot blocks, if wanted.
fn build_progress_bar(no_progress: bool, verb: &str) -> Result<Option<ProgressBar>> {
    if no_progress {
        return Ok(None);
    }
    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&["  ", verb, "  [{bar:50.white/black}] {pos}/{len} ({eta})"].concat())
            .context(error::ProgressBarTemplateSnafu)?
            .progress_chars("=> "),
    );
    Ok(Some(progress_bar))
}

/// Uploads the given path into a snapshot.
pub(crate) async fn snapshot_from_image<P>(
    path: P,
    uploader: &SnapshotUploader,
    desired_size: Option<i64>,
    no_progress: bool,
) -> Result<String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let progress_bar = build_progress_bar(no_progress, "Uploading snapshot");
    let filename = path
        .file_name()
        .context(error::InvalidImagePathSnafu { path })?
        .to_string_lossy();

    uploader
        .upload_from_file(path, desired_size, Some(&filename), progress_bar?)
        .await
        .context(error::UploadSnapshotSnafu)
}

mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    #[allow(clippy::large_enum_variant)]
    pub(crate) enum Error {
        #[snafu(display("Invalid image path '{}'", path.display()))]
        InvalidImagePath { path: PathBuf },

        #[snafu(display("Failed to parse progress style template: {}", source))]
        ProgressBarTemplate {
            source: indicatif::style::TemplateError,
        },

        #[snafu(display("Failed to upload snapshot: {}", source))]
        UploadSnapshot { source: coldsnap::UploadError },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
