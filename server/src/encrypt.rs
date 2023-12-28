use std::io::{Read, Write};

use miette::{Context, IntoDiagnostic, Result};

#[derive(Debug, Clone)]
pub struct Config {
    secret_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            secret_key: std::env::var("ENCRYPTION_SECRET_KEY")
                .into_diagnostic()
                .wrap_err("Missing ENCRYPTION_SECRET_KEY, needed for encryption")?,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            secret_key: "FAKE_SECRET_KEY".to_string(),
        }
    }
}

pub fn encrypt(data: &str, config: &Config) -> Result<Vec<u8>> {
    let encrypted = {
        let encryptor = age::Encryptor::with_user_passphrase(age::secrecy::Secret::new(
            config.secret_key.to_owned(),
        ));

        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted).into_diagnostic()?;
        writer.write_all(data.as_bytes()).into_diagnostic()?;
        writer.finish().into_diagnostic()?;

        encrypted
    };

    Ok(encrypted)
}

pub fn decrypt(data: &[u8], config: &Config) -> miette::Result<String> {
    let decrypted = {
        let decryptor = match age::Decryptor::new(data).into_diagnostic()? {
            age::Decryptor::Passphrase(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted = vec![];
        let mut reader = decryptor
            .decrypt(
                &age::secrecy::Secret::new(config.secret_key.to_owned()),
                None,
            )
            .into_diagnostic()?;
        reader.read_to_end(&mut decrypted).into_diagnostic()?;

        decrypted
    };

    String::from_utf8(decrypted).into_diagnostic()
}

impl Config {
    pub fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
        encrypt(data, self)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<String> {
        decrypt(data, self)
    }
}
