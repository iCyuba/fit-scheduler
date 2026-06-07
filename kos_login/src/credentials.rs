use std::env::{VarError, var};

/// Username and password
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn from_env() -> Result<Self, VarError> {
        Ok(Self::new(var("USERNAME")?, var("PASSWORD")?))
    }
}
