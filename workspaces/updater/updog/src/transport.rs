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

    /// Set the query string appended to tough requests, sorting the queries
    /// by key name first
    fn set_query_string(&self, mut url: Url) -> Url {
        if let Ok(mut queries) = self.parameters.try_borrow_mut() {
            queries.sort_by(|(a,_), (b,_)| a.cmp(b));

            for (key, val) in queries.iter() {
                url.query_pairs_mut().append_pair(&key, &val);
            }
        } else {
            // We can't sort the actual data at the moment, but we can sort
            // what we append to the URL.
            let mut queries = self.parameters.borrow().clone();
            queries.sort_by(|(a,_), (b,_)| a.cmp(b));

            for (key, val) in queries {
                url.query_pairs_mut().append_pair(&key, &val);
            }
        }

        url
    }
}

pub type HttpQueryRepo<'a> = Repository<'a, HttpQueryTransport>;

impl Transport for HttpQueryTransport {
    type Stream = reqwest::blocking::Response;
    type Error = reqwest::Error;

    fn fetch(&self, url: Url) -> Result<Self::Stream, Self::Error> {
        self.inner.fetch(self.set_query_string(url))
    }
}
