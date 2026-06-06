use crate::credentials::Credentials;
use anyhow::{Context, bail};
use const_format::formatc;
use html_escape::decode_html_entities;
use lol_html::{element, errors::RewritingError, text, HtmlRewriter};
use reqwest::{
    Client, Response, Url,
    header::{self, HeaderValue, LOCATION, ORIGIN, REFERER},
};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use thiserror::Error;

pub struct URLS;

pub const TENANT_ID: &str = "f345c406-5268-43b0-b19f-5862fa6833f8";
pub const CLIENT_ID: &str = "9e5f94bc-e8a4-4e73-b8be-63364c29d753";

impl URLS {
    cfg_select! {
        feature = "real" => {
            pub const AUTHORIZE: &str = formatc!("https://login.microsoftonline.com/{TENANT_ID}/oauth2/v2.0/authorize?client_id={CLIENT_ID}&response_type=code&scope=openid%20profile%20offline_access&redirect_uri=http%3A%2F%2Flocalhost%2F");
            pub const CRED_TYPE: &str = formatc!("https://login.microsoftonline.com/{TENANT_ID}/GetCredentialType");
            pub const TOKEN: &str = formatc!("https://login.microsoftonline.com/{TENANT_ID}/oauth2/v2.0/token");
            pub const CVUT: &str = "https://logon.ms.cvut.cz/";
            pub const FORM: &str = "https://login.microsoftonline.com/login.srf";
        }

        not(feature = "real") => {
            pub const AUTHORIZE: &str = "http://localhost:3000/login_fresh";
            pub const CRED_TYPE: &str = "http://localhost:3000/cred_type.json";
            pub const TOKEN: &str = "http://localhost:3000/token";
            pub const CVUT: &str = "http://localhost:3000/";
            pub const FORM: &str = "http://localhost:3000/login.srf";
        }
    }
}

pub async fn cvut(client: &Client, creds: &Credentials) -> anyhow::Result<()> {
    // Get login url
    let res = client.get(URLS::CVUT).send().await?;
    let location = res
        .headers()
        .get(LOCATION)
        .context("No redirect")?
        .to_str()?;

    if location == "https://logon.ms.cvut.cz/adfs/ls/IdpInitiatedSignon.aspx/" {
        return Ok(());
    }

    // Send login info
    let mut headers = header::HeaderMap::new();
    headers.insert(ORIGIN, HeaderValue::from_static(URLS::CVUT));
    headers.insert(REFERER, location.parse()?);

    let mut res = client
        .post(location)
        .headers(headers)
        .form(&[
            ("UserName", creds.username.as_str()),
            ("Password", creds.password.as_str()),
            ("AuthMethod", "FormsAuthentication"),
        ])
        .send()
        .await?;

    let mut location = res
        .headers()
        .get(LOCATION)
        .context("No redirect")?
        .to_str()?;

    if location == "https://logon.ms.cvut.cz/adfs/ls/IdpInitiatedSignon.aspx/" {
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
            .context("No redirect")?
            .to_str()?;

        if location == "https://logon.ms.cvut.cz/adfs/ls/IdpInitiatedSignon.aspx/" {
            return Ok(());
        }
    }

    bail!("Too many redirects")
}

pub async fn ms_login(client: &Client, creds: &Credentials) -> anyhow::Result<()> {
    let mut res = client.get(URLS::AUTHORIZE).send().await?;

    if let Some(redirect) = res.headers().get(LOCATION) {
        let redirect = redirect.to_str()?;

        let code = parse_code(redirect)?;
        let token = exchange_code(code).await?;

        println!("{token}");

        return Ok(());
    }

    let config = parse_config(&mut res).await?;

    let flow_token = config
        .get("sFT")
        .and_then(|ft| ft.as_str())
        .context("Missing flow token")?;

    let ctx = config
        .get("sCtx")
        .and_then(|ctx| ctx.as_str())
        .context("Missing context")?;

    let url = get_federated_url(client, creds, flow_token, ctx).await?;

    federated(client, &url).await?;

    Ok(())
}

pub async fn get_federated_url(
    client: &Client,
    creds: &Credentials,
    flow_token: &str,
    ctx: &str,
) -> anyhow::Result<String> {
    let res = client
        .post(URLS::CRED_TYPE)
        .json(&HashMap::from([
            ("flowToken", flow_token),
            ("originalRequest", ctx),
            ("username", &creds.username),
        ]))
        .send()
        .await?;

    let json = res.json::<Value>().await?;

    let credentials = json.get("Credentials").context("Missing Credentials")?;

    let url = credentials
        .get("FederationRedirectUrl")
        .and_then(|url| url.as_str())
        .context("Missing FederationRedirectUrl")?;

    Ok(url.to_string())
}

pub async fn federated(client: &Client, url: &str) -> anyhow::Result<()> {
    let mut res = client.get(url).send().await?;

    let form = parse_hidden_form(&mut res).await?;

    let url = post_form(client, form).await?;

    let code = parse_code(&url)?;
    let token = exchange_code(code).await?;

    println!("{token}");

    Ok(())
}

async fn post_form(client: &Client, form: HiddenForm) -> anyhow::Result<String> {
    let mut headers = header::HeaderMap::new();

    headers.insert(ORIGIN, HeaderValue::from_static(URLS::CVUT));
    headers.insert(REFERER, HeaderValue::from_static(URLS::CVUT));

    let res = client
        .post(URLS::FORM)
        .headers(headers)
        .form(&[
            ("wa", Cow::Borrowed("wsignin1.0")),
            ("wresult", Cow::Owned(form.result)),
            ("wctx", Cow::Owned(form.ctx)),
        ])
        .send()
        .await?;

    let url = res.headers().get(LOCATION).context("Missing redirect")?;

    Ok(url.to_str()?.to_string())
}

fn parse_code(url: &str) -> anyhow::Result<String> {
    let url = Url::parse(url)?;

    let (_, code) = url
        .query_pairs()
        .find(|(name, _)| name == "code")
        .context("Code missing in url query")?;

    let code = decode_html_entities(&code);

    Ok(code.into_owned())
}

async fn exchange_code(code: String) -> anyhow::Result<String> {
    let res = Client::new()
        .post(URLS::TOKEN)
        .form(&[
            ("client_id", Cow::Borrowed(CLIENT_ID)),
            ("code", Cow::Owned(code)),
            ("redirect_uri", Cow::Borrowed("http://localhost/")),
            ("grant_type", Cow::Borrowed("authorization_code")),
        ])
        .send()
        .await?;

    let json = res.json::<Value>().await?;

    let id_token = json
        .get("id_token")
        .and_then(|jwt| jwt.as_str())
        .context("Missing id_token")?;

    Ok(id_token.to_string())
}

#[derive(Debug, Error)]
#[error("Done!")]
struct DoneError;

async fn parse_config(res: &mut Response) -> anyhow::Result<Value> {
    const PREFIX: &str = "//<![CDATA[\n$Config=";
    const SUFFIX: &str = ";\n//]]>";

    let mut config = String::new();

    let settings = lol_html::Settings::new().append_element_content_handler(text!(
        "script[type=\"text/javascript\"]",
        |t| {
            config.push_str(t.as_str());

            if t.last_in_text_node() {
                if config.starts_with(PREFIX) && config.ends_with(SUFFIX) {
                    Err(DoneError)?
                } else {
                    config.clear();
                }
            }

            Ok(())
        }
    ));

    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});

    let mut done = false;

    while let Some(chunk) = res.chunk().await? {
        let result = rewriter.write(chunk.as_ref());

        if let Err(RewritingError::ContentHandlerError(err)) = result {
            if err.is::<DoneError>() {
                done = true;
                break;
            } else {
                unreachable!("DoneError is the only error that can happen.")
            }
        } else {
            result?;
        }
    }

    if !done {
        bail!("$Config variable is missing from the login page");
    }

    drop(rewriter);

    let config = &config[PREFIX.len()..(config.len() - SUFFIX.len())];

    let config = serde_json::from_str::<Value>(config)?;

    Ok(config)
}

#[derive(Debug, Default)]
struct HiddenForm {
    result: String,
    ctx: String,
}

async fn parse_hidden_form(res: &mut Response) -> anyhow::Result<HiddenForm> {
    let mut form = HiddenForm::default();

    let settings = lol_html::Settings::new()
        .append_element_content_handler(element!("input[name=\"wresult\"]", |el| {
            form.result = el.get_attribute("value").context("Missing wresult value")?;

            Ok(())
        }))
        .append_element_content_handler(element!("input[name=\"wctx\"]", |el| {
            form.ctx = el.get_attribute("value").context("Missing wctx value")?;

            Ok(())
        }));

    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});

    while let Some(chunk) = res.chunk().await? {
        rewriter.write(chunk.as_ref())?;
    }

    rewriter.end()?;

    if let Cow::Owned(result) = decode_html_entities(&form.result) {
        form.result = result;
    }

    if let Cow::Owned(ctx) = decode_html_entities(&form.ctx) {
        form.ctx = ctx;
    }

    Ok(form)
}
