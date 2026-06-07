use std::borrow::Cow;

use reqwest::{
    Url,
    header::{self, HeaderValue, LOCATION, ORIGIN, REFERER},
};

use crate::{client::Client, login::error::LoginError, parser::wsignin::WSignInForm, urls};

/// `POST /login.srf`
///
/// Submit the form returned by CVUT AD FS.
///
/// This will redirect to the return url if no further confirmation is required
/// (such as MFA).
///
/// An error will be returned if there's no redirect here.
pub async fn post_form(client: &Client, form: WSignInForm) -> Result<Url, LoginError> {
    let mut headers = header::HeaderMap::new();

    headers.insert(ORIGIN, HeaderValue::from_static(urls::CVUT));
    headers.insert(REFERER, HeaderValue::from_static(urls::CVUT));

    let res = client
        .post(urls::FORM)
        .headers(headers)
        .form(&[
            ("wa", Cow::Borrowed("wsignin1.0")),
            ("wresult", Cow::Owned(form.result)),
            ("wctx", Cow::Owned(form.ctx)),
        ])
        .send()
        .await?;

    let url = res
        .headers()
        .get(LOCATION)
        .ok_or(LoginError::NoRedirect)?
        .to_str()?
        .parse()?;

    Ok(url)
}
