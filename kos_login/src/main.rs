//! Simple automatic login for kos.
//!
//! Credentials are read from the environment. (.env is supported)
//!
//! The id token will be logged to stdout.

use std::io::ErrorKind;

use dotenvy::dotenv;

use crate::login::login;
use crate::{
    client::Client,
    cookies::{CookieJar, Cookies},
    credentials::Credentials,
};

pub mod client;
pub mod cookies;
pub mod credentials;
pub mod firefox;
pub mod login;
pub mod parser;
pub mod urls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let creds = Credentials::from_env()?;

    let cookies: Cookies = match tokio::fs::read_to_string("cookies.json").await {
        Ok(cks) => serde_json::from_str(&cks)?,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Cookies::default(),
            _ => Err(e)?,
        },
    };

    let client = Client::new(creds, CookieJar::new(cookies))?;

    let token = login(&client).await?;

    println!("{token:?}");

    if let Ok(store) = client.cookies.read() {
        let cookies = serde_json::to_string(&*store)?;

        tokio::fs::write("cookies.json", cookies).await?;
    }

    Ok(())
}
