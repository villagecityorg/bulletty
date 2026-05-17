/// vchat.email agent identity — read/write Nkey seed from disk.
///
/// The vchat binary (installed via the signup script) generates
/// and manages the ED25519 keypair. VReader simply reads the public key
/// from a well-known file for display purposes.

use std::path::Path;

use color_eyre::Result;

/// The filename where the vchat CLI stores its Nkey seed (relative to identity dir).
pub const AGENT_NKEY_FILE: &str = "agent.nkey";

/// Read the seed file and return the raw seed string.
pub fn read_seed(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

/// Extract the public Nkey from a seed file by calling the vchat binary.
/// Falls back to showing the seed prefix if the binary isn't available.
pub fn get_public_key(identity_dir: &Path) -> Result<Option<String>> {
    let seed_path = identity_dir.join(AGENT_NKEY_FILE);
    if !seed_path.exists() {
        return Ok(None);
    }

    // Try using the vchat binary to decode the public key
    if let Some(bin) = super::find_vchat_binary() {
        let output = std::process::Command::new(bin)
            .arg("key")
            .arg("public")
            .arg(seed_path.to_string_lossy().as_ref())
            .output()?;
        if output.status.success() {
            let pk = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !pk.is_empty() {
                return Ok(Some(pk));
            }
        }
    }

    // Fallback: show a preview of the seed (we can't decode without nkeys)
    let seed = read_seed(&seed_path)?;
    let preview = if seed.len() > 12 {
        format!("{}...{}", &seed[..6], &seed[seed.len()-4..])
    } else {
        seed
    };
    Ok(Some(format!("[seed: {}]", preview)))
}
