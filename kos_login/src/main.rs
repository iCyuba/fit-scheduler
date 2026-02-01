use std::{
    borrow::Cow,
    env::{self, VarError},
};

use anyhow::bail;

use securestore::{KeySource, SecretsManager};

use crate::{credentials::Credentials, login::login};

pub mod credentials;
pub mod login;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut sm = SecretsManager::load("secrets.json", KeySource::File("secrets.key"))?;
    let creds = Credentials::from_secrets(&sm)?;

    let webdriver = match env::var("WEBDRIVER") {
        Ok(val) => Cow::Owned(val),
        Err(err) => match err {
            VarError::NotPresent => Cow::Borrowed("http://localhost:4444"),
            VarError::NotUnicode(_) => bail!("Invalid WebDriver url"),
        },
    };

    let cookies = login(&webdriver, &creds).await?;
    let cookies: Vec<_> = cookies.into_iter().map(|c| c.to_string()).collect();

    let json = serde_json::to_string(&cookies)?;

    sm.set("cookies", json);
    sm.save()?;

    Ok(())
}
