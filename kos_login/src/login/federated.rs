use reqwest::Url;

use crate::{
    client::Client,
    login::{cvut::cvut_login, error::LoginError},
    parser::wsignin::{WSignInForm, WSignInParseError, parse_wsignin},
};

/// `GET {FederationRedirectUrl}`
///
/// If the user is already logged in to the CVUT AD FS page, the response will
/// be a basic HTML form with the result and context.
///
/// The form is normally submitted on load using JavaScript, here we have to
/// parse it manually.
///
/// If the user isn't logged in, the parsing will fail, because the server
/// responds with a login page. In this case we just ensure that the user is
/// logged in separately and then retry.
pub async fn federated_login(client: &Client, url: Url) -> Result<WSignInForm, LoginError> {
    let res = client.get(url.clone()).send().await?;

    let form = match parse_wsignin(&mut res.bytes_stream()).await {
        Ok(form) => form,
        Err(WSignInParseError::MissingData) => {
            // Try signing in again
            cvut_login(client).await?;

            let res = client.get(url).send().await?;

            parse_wsignin(&mut res.bytes_stream()).await?
        }

        Err(err) => Err(err)?,
    };

    Ok(form)
}
