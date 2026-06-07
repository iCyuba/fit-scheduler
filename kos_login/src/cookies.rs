use std::{
    ops::{Deref, DerefMut},
    sync::RwLock,
};

use bytes::Bytes;
use cookie_store::{CookieStore, RawCookie};
use reqwest::{Url, header::HeaderValue};
use serde::{Deserialize, Serialize};

/// A cookie store wrapper with a custom serialize impl
#[derive(Debug, Default, Deserialize)]
pub struct Cookies(CookieStore);

/// Cookie Jar struct
///
/// Copied entirely from reqwest, the only difference is that this struct is
/// generic and holds anything that can be dereferenced into a CookieStore
#[derive(Debug)]
pub struct CookieJar<T = Cookies>(RwLock<T>)
where
    T: AsRef<CookieStore> + AsMut<CookieStore>;

impl<T> CookieJar<T>
where
    T: AsRef<CookieStore> + AsMut<CookieStore> + Send + Sync,
{
    pub fn new(store: T) -> Self {
        Self(RwLock::new(store))
    }
}

impl DerefMut for Cookies {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Cookies {
    type Target = CookieStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsMut<CookieStore> for Cookies {
    fn as_mut(&mut self) -> &mut CookieStore {
        &mut *self
    }
}

impl AsRef<CookieStore> for Cookies {
    fn as_ref(&self) -> &CookieStore {
        &*self
    }
}

impl Serialize for Cookies {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.iter_unexpired())
    }
}

impl<T> DerefMut for CookieJar<T>
where
    T: AsRef<CookieStore> + AsMut<CookieStore> + Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Deref for CookieJar<T>
where
    T: AsRef<CookieStore> + AsMut<CookieStore> + Send + Sync,
{
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> reqwest::cookie::CookieStore for CookieJar<T>
where
    T: AsRef<CookieStore> + AsMut<CookieStore> + Send + Sync,
{
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &Url) {
        let iter = cookie_headers
            .filter_map(|val| RawCookie::parse(val.to_str().unwrap()).ok())
            .map(|c| c.into_owned());

        self.0
            .write()
            .unwrap()
            .as_mut()
            .store_response_cookies(iter, url);
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let s = self
            .0
            .read()
            .unwrap()
            .as_ref()
            .get_request_values(url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");

        if s.is_empty() {
            return None;
        }

        HeaderValue::from_maybe_shared(Bytes::from(s)).ok()
    }
}
