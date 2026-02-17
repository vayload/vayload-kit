use anyhow::{Context, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, ChaCha20Poly1305, Nonce,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    access_token: String,
    access_expires_at: u64,
    refresh_token: String,
    refresh_expires_at: u64,
}

pub struct RawCredentials {
    pub access_token: String,
    pub access_expires_in: u64,
    pub refresh_token: String,
    pub refresh_expires_in: u64,
}

impl RawCredentials {
    pub fn new(access_token: String, refresh_token: String, access_expires_in: u64) -> Self {
        Self {
            access_token,
            access_expires_in,
            refresh_token,
            refresh_expires_in: (access_expires_in / 60) * 24 * 60 * 60,
        }
    }

    fn to_credentials(&self) -> Result<Credentials> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        Ok(Credentials {
            access_token: self.access_token.clone(),
            access_expires_at: now + self.access_expires_in,
            refresh_token: self.refresh_token.clone(),
            refresh_expires_at: now + self.refresh_expires_in * 60,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedCredentials {
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
}

pub struct CredentialManager {
    config_dir: PathBuf,
}

impl CredentialManager {
    pub fn new() -> Result<Self> {
        let config_dir =
            dirs::config_dir().context("No se pudo encontrar el directorio de configuración")?.join("vayload-kit");

        fs::create_dir_all(&config_dir).context("Error al crear el directorio de configuración")?;

        Ok(Self { config_dir })
    }

    pub fn store_tokens(&self, credentials: RawCredentials) -> Result<()> {
        let creds = credentials.to_credentials()?;

        let json = serde_json::to_string(&creds)?;
        self.encrypt_and_write(json.as_bytes())
    }

    pub fn is_access_token_expired(&self) -> bool {
        self.check_expiration(|c| c.access_expires_at)
    }

    pub fn is_refresh_token_expired(&self) -> bool {
        self.check_expiration(|c| c.refresh_expires_at)
    }

    pub fn get_access_token(&self) -> Result<String> {
        Ok(self.get_credentials()?.access_token)
    }

    pub fn get_refresh_token(&self) -> Result<String> {
        Ok(self.get_credentials()?.refresh_token)
    }

    pub fn clear_all(&self) -> Result<()> {
        let _ = fs::remove_file(self.credentials_path());
        let _ = fs::remove_file(self.key_path());
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        !self.is_refresh_token_expired() || !self.is_access_token_expired()
    }

    fn check_expiration<F>(&self, selector: F) -> bool
    where
        F: Fn(&Credentials) -> u64,
    {
        match self.get_credentials() {
            Ok(creds) => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                now >= (selector(&creds).saturating_sub(30))
            },
            Err(_) => true,
        }
    }

    fn get_credentials(&self) -> Result<Credentials> {
        let encrypted_json = fs::read(self.credentials_path()).context("No hay credenciales guardadas")?;

        let encrypted: EncryptedCredentials = serde_json::from_slice(&encrypted_json)?;
        let key = self.get_or_create_key()?;

        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Error de descifrado: {}", e))?;

        Ok(serde_json::from_str(&String::from_utf8(plaintext)?)?)
    }

    fn encrypt_and_write(&self, plaintext: &[u8]) -> Result<()> {
        let key = self.get_or_create_key()?;
        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(|e| anyhow::anyhow!("Cifrado fallido: {}", e))?;

        let data = serde_json::to_vec(&EncryptedCredentials { ciphertext, nonce: nonce.to_vec() })?;

        let path = self.credentials_path();
        fs::write(&path, data)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    fn credentials_path(&self) -> PathBuf {
        self.config_dir.join("credentials.enc")
    }
    fn key_path(&self) -> PathBuf {
        self.config_dir.join(".key")
    }

    fn get_or_create_key(&self) -> Result<[u8; 32]> {
        let path = self.key_path();
        if path.exists() {
            let b = fs::read(&path)?;
            let mut key = [0u8; 32];
            key.copy_from_slice(&b);
            Ok(key)
        } else {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            fs::write(&path, key.to_vec())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            }
            Ok(key.into())
        }
    }
}
