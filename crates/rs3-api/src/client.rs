use crate::Rs3ApiError;
use reqwest::Client;

const USER_AGENT: &str = "gielinor-toolkit/0.1.0";

pub struct Rs3Client {
    pub(crate) http: Client,
}

impl Rs3Client {
    pub fn new() -> Result<Self, Rs3ApiError> {
        let http = Client::builder().user_agent(USER_AGENT).build()?;

        Ok(Self { http })
    }
}
