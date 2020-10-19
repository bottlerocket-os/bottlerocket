use super::error;
use std::cell::Cell;
use std::io::Read;
use tough::{FilesystemTransport, HttpTransport, Transport};
use url::Url;

/// RepoTransport delegates to FilesystemTransport or HttpTransport based on the url scheme. If we
/// detect that the repo isn't found we return a special error so we can start a new repo.
#[derive(Debug, Default, Clone)]
pub(crate) struct RepoTransport {
    // If we fail to fetch the repo, we need a way of conveying whether it happened because the
    // repo doesn't exist or because we failed to fetch/load a repo that does exist.  This
    // information can be used to determine whether we want to start a new repo from scratch or to
    // fail early, for example.
    //
    // tough uses a trait object to represent the source error inside its Error::Transport variant,
    // so we can't check our own, inner error type to determine which of our variants is inside.
    // Also, it defines the `fetch` method of `Transport` to take an immutable reference to self,
    // so we can't use a struct field naively to communicate back.
    //
    // So, we use this Cell to safely convey the information outward in our single-threaded usage.
    pub(crate) repo_not_found: Cell<bool>,
}

impl Transport for RepoTransport {
    type Stream = Box<dyn Read + Send>;
    type Error = error::Error;

    fn fetch(&self, url: Url) -> std::result::Result<Self::Stream, Self::Error> {
        if url.scheme() == "file" {
            match FilesystemTransport.fetch(url.clone()) {
                Ok(reader) => Ok(Box::new(reader)),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        self.repo_not_found.set(true);
                        error::RepoNotFound { url }.fail()
                    }
                    _ => error::RepoFetch {
                        url,
                        msg: e.to_string(),
                    }
                    .fail(),
                },
            }
        } else {
            let transport = HttpTransport::new();
            match transport.fetch(url.clone()) {
                Ok(reader) => Ok(Box::new(reader)),
                Err(e) => match e {
                    tough::error::Error::HttpFetch { .. } => {
                        self.repo_not_found.set(true);
                        error::RepoNotFound { url }.fail()
                    }
                    _ => error::RepoFetch {
                        url,
                        msg: e.to_string(),
                    }
                    .fail(),
                },
            }
        }
    }
}
