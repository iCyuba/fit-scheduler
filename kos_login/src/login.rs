use std::time::Duration;

use anyhow::{Context, bail};
use fantoccini::{
    Client, Locator,
    cookies::Cookie,
    error::{CmdError, ErrorStatus},
};
use tokio::time::sleep;

use crate::credentials::Credentials;

pub async fn login(driver: &str, creds: &Credentials) -> anyhow::Result<Vec<Cookie<'static>>> {
    let client = fantoccini::ClientBuilder::native().connect(driver).await?;

    client.goto("https://kos.cvut.cz").await?;

    login_flow(&client, creds).await?;

    wait_for(
        &mut async || match client.get_named_cookie("JSESSIONID").await {
            Ok(_) => Ok(true),
            Err(err) => match err {
                CmdError::Standard(wd) => match wd.error {
                    ErrorStatus::NoSuchCookie => Ok(false),
                    err => Err(err.into()),
                },
                _ => Err(err.into()),
            },
        },
    )
    .await?;

    let cookies = client.get_all_cookies().await?;

    Ok(cookies)
}

async fn login_flow(client: &Client, creds: &Credentials) -> anyhow::Result<()> {
    login_microsoft(client, creds).await?;
    login_cvut(client, creds).await?;
    login_topt(client, creds).await?;

    wait_for(&mut async || {
        let url = client.current_url().await?;
        let host = url.host_str().context("Missing host")?;

        Ok(matches!(host, "kos.cvut.cz"))
    })
    .await?;

    Ok(())
}

async fn login_microsoft(client: &Client, creds: &Credentials) -> anyhow::Result<()> {
    wait_for(&mut async || {
        let url = client.current_url().await?;
        let host = url.host_str().context("Missing host")?;

        Ok(matches!(host, "login.microsoftonline.com"))
    })
    .await?;

    let username = client
        .wait()
        .for_element(Locator::Css("input[placeholder=\"username@cvut.cz\"]"))
        .await?;

    username.send_keys(&creds.username).await?;

    let submit = client.find(Locator::Css("input[type=submit]")).await?;
    submit.click().await?;

    Ok(())
}

async fn login_cvut(client: &Client, creds: &Credentials) -> anyhow::Result<()> {
    wait_for(&mut async || {
        let url = client.current_url().await?;
        let host = url.host_str().context("Missing host")?;

        Ok(matches!(host, "logon.ms.cvut.cz"))
    })
    .await?;

    let password = client
        .wait()
        .for_element(Locator::Css("input#passwordInput"))
        .await?;

    password.send_keys(&creds.password).await?;

    let submit = client.find(Locator::Css("#submitButton")).await?;
    submit.click().await?;

    Ok(())
}

async fn login_topt(client: &Client, creds: &Credentials) -> anyhow::Result<()> {
    wait_for(&mut async || {
        let url = client.current_url().await?;
        let host = url.host_str().context("Missing host")?;

        Ok(matches!(host, "login.microsoftonline.com"))
    })
    .await?;

    let code = client
        .wait()
        .for_element(Locator::Css("input[type=tel]"))
        .await?;

    code.send_keys(&creds.totp.generate_current()?).await?;

    let submit = client.find(Locator::Css("input[type=submit]")).await?;
    submit.click().await?;

    Ok(())
}

async fn wait_for(fun: &mut impl AsyncFnMut() -> anyhow::Result<bool>) -> anyhow::Result<()> {
    for _ in 0..120 {
        if (fun)().await? {
            return Ok(());
        }

        sleep(Duration::from_millis(250)).await;
    }

    bail!("Timed out");
}
