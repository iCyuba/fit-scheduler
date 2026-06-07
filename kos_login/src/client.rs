use std::{ops::Deref, sync::Arc};

use reqwest::redirect::Policy;

use crate::{cookies::CookieJar, credentials::Credentials, firefox};

/// reqwest client extension
///
/// Holds the cookies and credentials
#[derive(Debug, Clone)]
pub struct Client {
    pub credentials: Credentials,
    pub cookies: Arc<CookieJar>,

    client: reqwest::Client,
}

impl Client {
    pub fn new(
        credentials: Credentials,
        cookies: impl Into<Arc<CookieJar>>,
    ) -> reqwest::Result<Self> {
        let cookies = cookies.into();

        let client = reqwest::Client::builder()
            .cookie_provider(cookies.clone())
            .default_headers(firefox::get_headers())
            .redirect(Policy::none())
            .build()?;

        Ok(Self {
            credentials,
            cookies,
            client,
        })
    }

    pub fn username(&self) -> &str {
        &self.credentials.username
    }

    pub fn password(&self) -> &str {
        &self.credentials.password
    }
}

impl Deref for Client {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
