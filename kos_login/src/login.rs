//! Everything needed for the login flow

use crate::{
    client::Client,
    login::{
        cred_type::get_federation_url,
        cvut::cvut_login,
        error::LoginError,
        federated::federated_login,
        form::post_form,
        msonline::{MsOnlineResponse, get_msonline},
        token::{TokenResponse, exchange_code, parse_code},
    },
};

pub mod cred_type;
pub mod cvut;
pub mod error;
pub mod federated;
pub mod form;
pub mod msonline;
pub mod token;

/// Login and return the id token
///
/// This function automates the ms-cvut login flow and returns an id token
/// by Azure AD, which is supported by kos.
///
/// There's so many parts where this could probably fail, but so far it works.
pub async fn login(client: &Client) -> Result<TokenResponse, LoginError> {
    let url = match get_msonline(client).await? {
        MsOnlineResponse::Redirect(url) => url,
        MsOnlineResponse::AuthorizeConfig(auth_conf) => {
            let fed_url = get_federation_url(client, auth_conf).await?;

            cvut_login(&client).await?;
            let form = federated_login(client, fed_url).await?;
            let url = post_form(client, form).await?;

            url
        }
    };

    let code = parse_code(&url).ok_or(LoginError::MissingCode)?;
    let token = exchange_code(code).await?;

    Ok(token)
}
