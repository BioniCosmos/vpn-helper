use anyhow::{anyhow, Result};
use std::env;

#[derive(Debug)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub totp_secret: String,
    pub host: String,
    pub route_cidr: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            username: Self::get_env("USERNAME")?,
            password: Self::get_env("PASSWORD")?,
            totp_secret: Self::get_env("TOTP_SECRET")?,
            host: Self::get_env("HOST")?,
            route_cidr: Self::get_env("ROUTE_CIDR")?,
        })
    }

    fn get_env(key: &str) -> Result<String> {
        env::var(key).or(Err(anyhow!("{key} not set")))
    }
}
