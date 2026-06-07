//! Hardcoded urls used in the login flow

use const_format::{formatc, str_replace};

pub const TENANT_ID: &str = "f345c406-5268-43b0-b19f-5862fa6833f8";
pub const CLIENT_ID: &str = "9e5f94bc-e8a4-4e73-b8be-63364c29d753";

pub const REDIRECT: &str = "http://localhost/";
pub const REDIRECT_ENCODED: &str = str_replace!(str_replace!(REDIRECT, ':', "%3A"), '/', "%2F");

pub const AUTHORIZE: &str = formatc!(
    "https://login.microsoftonline.com/{TENANT_ID}/oauth2/v2.0/authorize?client_id={CLIENT_ID}&response_type=code&scope=openid%20profile%20offline_access&redirect_uri={REDIRECT_ENCODED}"
);

pub const CRED_TYPE: &str =
    formatc!("https://login.microsoftonline.com/{TENANT_ID}/GetCredentialType");

pub const TOKEN: &str = formatc!("https://login.microsoftonline.com/{TENANT_ID}/oauth2/v2.0/token");

pub const CVUT: &str = "https://logon.ms.cvut.cz/";
pub const CVUT_LOGGED_IN: &str = "https://logon.ms.cvut.cz/adfs/ls/IdpInitiatedSignon.aspx/";

pub const FORM: &str = "https://login.microsoftonline.com/login.srf";
