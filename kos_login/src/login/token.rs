use std::borrow::Cow;

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::urls;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenResponse {
    pub refresh_token: String,
    pub id_token: String,
    // pub access_token: String,
    // pub token_type: String,
    // pub expires_in: i64,
    // pub scope: String,
}

/// Parse the auth code from the return url
///
/// Normally the browser would make a request here, but we can skip it.
pub fn parse_code<'url>(url: &'url Url) -> Option<Cow<'url, str>> {
    url.query_pairs()
        .find(|(name, _)| name == "code")
        .map(|(_, code)| code)
}

/// `POST /{TENANT_ID}/oauth2/v2.0/token`
///
/// This is the last step, which is also the only part of the flow that is
/// officially documented.
///
/// We "trade in" the auth code for an access token. We don't actually need the
/// access token, so we don't parse it. (The id token is better)
///
/// The id and refresh token are the only things returned, because the rest is
/// useless.
pub async fn exchange_code(code: Cow<'_, str>) -> reqwest::Result<TokenResponse> {
    let res = Client::new()
        .post(urls::TOKEN)
        .form(&json!({
            "client_id": Cow::Borrowed(urls::CLIENT_ID),
            "code": code,
            "redirect_uri": Cow::Borrowed(urls::REDIRECT),
            "grant_type": Cow::Borrowed("authorization_code"),
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(res)
}
