use reqwest::header::LOCATION;
use serde::Deserialize;
use url::Url;

use crate::{client::Client, login::error::LoginError, parser::config::parse_config, urls};

#[derive(Debug, Deserialize)]
pub struct AuthorizeConfig {
    #[serde(rename = "sFT")]
    pub flow_token: String,

    #[serde(rename = "sCtx")]
    pub context: String,
}

#[derive(Debug)]
pub enum MsOnlineResponse {
    Redirect(Url),
    AuthorizeConfig(AuthorizeConfig),
}

/// `GET /{TENANT_ID}/oauth2/v2.0/authorize`
///
/// This is the first request in the login flow.
///
/// It can either redirect to the return url straight away if the user logged
/// in recently or respond with the HTML login page.
///
/// On the login page, the first inline script contains a config variable,
/// which holds the flow token and context. These strings are then used in
/// [`super::cred_type::get_federation_url`].
pub async fn get_msonline(client: &Client) -> Result<MsOnlineResponse, LoginError> {
    let res = client.get(urls::AUTHORIZE).send().await?;

    if let Some(redirect) = res.headers().get(LOCATION) {
        let redirect = redirect.to_str()?.parse()?;

        return Ok(MsOnlineResponse::Redirect(redirect));
    }

    let config = parse_config(&mut res.bytes_stream()).await?;
    let config: AuthorizeConfig = serde_json::from_value(config)?;

    Ok(MsOnlineResponse::AuthorizeConfig(config))
}
