use securestore::SecretsManager;
use totp_rs::TOTP;

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub totp: TOTP,
}

impl Credentials {
    pub fn from_secrets(sm: &SecretsManager) -> Result<Self, securestore::Error> {
        Ok(Self {
            username: sm.get("username")?,
            password: sm.get("password")?,
            totp: TOTP::from_url_unchecked(sm.get("totp")?).unwrap(),
        })
    }
}
