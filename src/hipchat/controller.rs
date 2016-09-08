use std::time::Duration;
use hyper::client::Client;

pub struct Controller {
    pub auth: String,
    pub hyper: Client,
}

impl Controller {
    pub fn new(auth: String) -> Controller {
        let mut client: Client = Client::new();
        client.set_read_timeout(Some(Duration::from_secs(30)));
        client.set_write_timeout(Some(Duration::from_secs(30)));
        Controller {
            auth: auth,
            hyper: client,
        }
    }
}
