use std::{fmt::Debug, ops::Deref};

use base64::{DecodeError, Engine};

#[derive(Clone)]
pub struct CookieKey(pub tower_cookies::Key);

impl Deref for CookieKey {
    type Target = tower_cookies::Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CookieKey {
    pub fn from_env_or_generate() -> Result<Self, DecodeError> {
        let cookie_key = std::env::var("COOKIE_KEY");
        let cookie_key = if let Ok(cookie_key) = cookie_key {
            let cookie_key =
                base64::engine::general_purpose::STANDARD.decode(cookie_key.as_bytes())?;

            tower_cookies::Key::derive_from(&cookie_key)
        } else {
            tracing::info!("Generating new cookie key");
            let k = tower_cookies::Key::generate();
            let based = base64::engine::general_purpose::STANDARD.encode(k.master());
            dbg!(&based);
            k
        };
        Ok(Self(cookie_key))
    }
}

impl Debug for CookieKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CookieKey")
            .field("value", &"[omitted]")
            .finish()
    }
}
