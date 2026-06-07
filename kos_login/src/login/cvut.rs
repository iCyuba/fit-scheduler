use reqwest::header::{self, HeaderValue, LOCATION, ORIGIN, REFERER};

use crate::{client::Client, login::error::LoginError, urls};

/// Login on the CVUT AD FS page.
///
/// Calling this function ensures that the user is logged in.
///
/// The login state is decided based on the redirect url, which differs
/// depending on the cookies sent with the request.
///
/// If there's no valid session, the login form is sent and cookies are stored.
pub async fn cvut_login(client: &Client) -> Result<(), LoginError> {
    // Get login url
    let res = client.get(urls::CVUT).send().await?;
    let location = res
        .headers()
        .get(LOCATION)
        .ok_or(LoginError::NoRedirect)?
        .to_str()?;

    if location == urls::CVUT_LOGGED_IN {
        return Ok(());
    }

    // Send login info
    let mut headers = header::HeaderMap::new();
    headers.insert(ORIGIN, HeaderValue::from_static(urls::CVUT));
    headers.insert(REFERER, location.parse()?);

    let mut res = client
        .post(location)
        .headers(headers)
        .form(&[
            ("UserName", client.username()),
            ("Password", client.password()),
            ("AuthMethod", "FormsAuthentication"),
        ])
        .send()
        .await?;

    let mut location = res
        .headers()
        .get(LOCATION)
        .ok_or(LoginError::NoRedirect)?
        .to_str()?;

    if location == urls::CVUT_LOGGED_IN {
        return Ok(());
    }

    // Get cookies
    for _ in 0..5 {
        let mut headers = header::HeaderMap::new();
        headers.insert(REFERER, location.parse()?);

        res = client.get(location).headers(headers).send().await?;
        location = res
            .headers()
            .get(LOCATION)
            .ok_or(LoginError::NoRedirect)?
            .to_str()?;

        if location == urls::CVUT_LOGGED_IN {
            return Ok(());
        }
    }

    Err(LoginError::TooManyRedirects)
}
