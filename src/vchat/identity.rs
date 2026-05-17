/// ED25519 identity generation and persistence for vchat.email.

use std::path::Path;

use color_eyre::Result;

/// A sovereign agent identity backed by an ED25519 keypair.
///
/// The seed (private key) is stored locally and NEVER transmitted.
/// The public key in NATS Nkey format (e.g. "SA...") is used for
/// onboarding and authentication.
pub struct AgentIdentity {
    keypair: nkeys::KeyPair,
}

impl AgentIdentity {
    /// Generate a new sovereign identity.
    pub fn generate() -> Result<Self> {
        let kp = nkeys::KeyPair::new_user();
        Ok(Self { keypair: kp })
    }

    /// Load an identity from a seed file.
    pub fn load(path: &Path) -> Result<Self> {
        let seed = std::fs::read_to_string(path)?;
        let kp = nkeys::KeyPair::from_seed(&seed.trim())?;
        Ok(Self { keypair: kp })
    }

    /// Save the seed to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let seed = self.keypair.seed()?;
        std::fs::write(path, seed.as_bytes())?;
        Ok(())
    }

    /// Get the public Nkey string (e.g. "SA...").
    pub fn public_key(&self) -> String {
        self.keypair.public_key()
    }

    /// Check if an identity file exists at the given path.
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Get the seed as a string (for NATS auth).
    pub fn seed_string(&self) -> Result<String> {
        Ok(self.keypair.seed()?)
    }
}
