use std::sync::{Arc, LazyLock};

use reqwest::{
    Client, Url,
    header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
};
use reqwest_cookie_store::{CookieStore, CookieStoreRwLock, RawCookie};
use securestore::{KeySource, SecretsManager};

pub const KOS_URL: &str = "https://kos.cvut.cz/rest/api/";
pub static KOS: LazyLock<Url> = LazyLock::new(|| Url::parse(KOS_URL).unwrap());

pub fn get_client() -> anyhow::Result<Client> {
    let sm = SecretsManager::load("secrets.json", KeySource::File("secrets.key"))?;

    let cookies = sm.get("cookies")?;
    let cookies: Vec<String> = serde_json::from_str(&cookies)?;

    let mut store = CookieStore::new();
    for c in cookies {
        store.insert_raw(&RawCookie::parse(c)?, &KOS)?;
    }

    let store = Arc::new(CookieStoreRwLock::new(store));
    let client = Client::builder()
        .cookie_provider(store)
        .default_headers(get_headers())
        .build()?;

    Ok(client)
}

fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:147.0) Gecko/20100101 Firefox/147.0",
        ),
    );

    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    headers
}
