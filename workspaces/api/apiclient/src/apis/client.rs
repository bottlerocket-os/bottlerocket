use std::rc::Rc;

use super::configuration::Configuration;

pub struct APIClient {
    configuration: Rc<Configuration>,
    default_api: Box<::apis::DefaultApi>,
}

impl APIClient {
    pub fn new(configuration: Configuration) -> APIClient {
        let rc = Rc::new(configuration);

        APIClient {
            configuration: rc.clone(),
            default_api: Box::new(::apis::DefaultApiClient::new(rc.clone())),
        }
    }

    pub fn default_api(&self) -> &::apis::DefaultApi{
        self.default_api.as_ref()
    }

}
