use cookie_store::{Cookie, CookieStore};
use reqwest::{Client, redirect::Policy};
use reqwest_cookie_store::CookieStoreRwLock;
use securestore::{KeySource, SecretsManager};
use std::io::ErrorKind;
use std::sync::Arc;
use crate::credentials::Credentials;
use crate::login::{cvut, ms_login};

pub mod credentials;
pub mod firefox;
pub mod login;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sm = SecretsManager::load("secrets.json", KeySource::File("secrets.key"))?;
    let creds = Credentials::from_secrets(&sm)?;

    let cookies: Vec<Cookie> = match tokio::fs::read_to_string("cookies.json").await {
        Ok(cks) => serde_json::from_str(&cks)?,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                vec![]
            }
            _ => Err(e)?,
        },
    };

    let store = CookieStore::from_cookies(cookies.into_iter().map(Ok::<_, ()>), true).unwrap();
    let store = Arc::new(CookieStoreRwLock::new(store));

    let client = Client::builder()
        .cookie_provider(store.clone())
        .default_headers(firefox::get_headers())
        .redirect(Policy::none())
        .build()?;

    cvut(&client, &creds).await?;

    ms_login(&client, &creds).await?;

    if let Ok(store) = store.read() {
        let cookies: Vec<_> = store.iter_any().collect();
        let cookies = serde_json::to_string_pretty(&cookies)?;

        println!("{cookies}");
        tokio::fs::write("cookies.json", cookies).await?;
    }

    Ok(())
}
