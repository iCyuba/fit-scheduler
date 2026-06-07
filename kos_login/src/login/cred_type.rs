use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::{
    client::Client,
    login::{error::LoginError, msonline::AuthorizeConfig},
    urls,
};

#[derive(Debug, Deserialize)]
struct CredentialType {
    #[serde(rename = "Credentials")]
    pub credentials: Credentials,
}

#[derive(Debug, Deserialize)]
struct Credentials {
    #[serde(rename = "FederationRedirectUrl")]
    pub federation_redirect_url: String,
}

/// `GET /{TENANT_ID}/GetCredentialType`
///
/// This endpoint is called on the ms login page after the user enters their
/// email and presses next.
///
/// The response is in json and contains a bunch of random things, here we
/// extract just the `FederationRedirectUrl`, which is where the user would
/// normally get redirected to.
pub async fn get_federation_url(
    client: &Client,
    auth_conf: AuthorizeConfig,
) -> Result<Url, LoginError> {
    let res: CredentialType = client
        .post(urls::CRED_TYPE)
        .json(&json!({
            "flowToken": auth_conf.flow_token,
            "originalRequest": auth_conf.context,
            "username": client.username(),
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let url = res.credentials.federation_redirect_url.parse()?;

    Ok(url)
}
