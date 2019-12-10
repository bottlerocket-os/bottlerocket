use std::cell::{BorrowMutError, RefCell};
use tough::{HttpTransport, Repository, Transport};
use url::Url;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct HttpQueryTransport {
    pub inner: HttpTransport,
    parameters: RefCell<Vec<(String, String)>>,
}

impl HttpQueryTransport {
    pub fn new() -> Self {
        Self {
            inner: HttpTransport::new(),
            parameters: RefCell::new(vec![]),
        }
    }

    /// Try to borrow a mutable reference to parameters; returns an error if
    /// a borrow is already active
    pub fn queries_get_mut(
        &self,
    ) -> Result<std::cell::RefMut<'_, Vec<(String, String)>>, BorrowMutError> {
        self.parameters.try_borrow_mut()
    }

    fn set_query_string(&self, mut url: Url) -> Url {
        for (key, val) in self.parameters.borrow().iter() {
            url.query_pairs_mut().append_pair(&key, &val);
        }
        url
    }
}

pub type HttpQueryRepo<'a> = Repository<'a, HttpQueryTransport>;

impl Transport for HttpQueryTransport {
    type Stream = reqwest::Response;
    type Error = reqwest::Error;

    fn fetch(&self, url: Url) -> Result<Self::Stream, Self::Error> {
        self.inner.fetch(self.set_query_string(url))
    }
}
