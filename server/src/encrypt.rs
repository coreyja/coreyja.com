use std::io::{Read, Write};

use cja::{color_eyre::eyre::Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    secret_key: String,
}

impl Config {
    #[tracing::instrument(name = "encrypt::Config::from_env")]
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            secret_key: std::env::var("ENCRYPTION_SECRET_KEY")
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
            config.secret_key.clone(),
        ));

        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted)?;
        writer.write_all(data.as_bytes())?;
        writer.finish()?;

        encrypted
    };

    Ok(encrypted)
}

pub fn decrypt(data: &[u8], config: &Config) -> cja::Result<String> {
    let decrypted = {
        let age::Decryptor::Passphrase(decryptor) = age::Decryptor::new(data)? else {
            unreachable!()
        };

        let mut decrypted = vec![];
        let mut reader =
            decryptor.decrypt(&age::secrecy::Secret::new(config.secret_key.clone()), None)?;
        reader.read_to_end(&mut decrypted)?;

        decrypted
    };

    Ok(String::from_utf8(decrypted)?)
}

impl Config {
    pub fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
        encrypt(data, self)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<String> {
        decrypt(data, self)
    }
}
