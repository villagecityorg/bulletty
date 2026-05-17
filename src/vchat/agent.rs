/// Agent onboarding and authenticated NATS connection.

use std::path::Path;

use color_eyre::{Result, eyre};

use crate::vchat::identity::AgentIdentity;
use crate::vchat::types::*;

/// Onboard this VReader instance as a vchat.email agent.
///
/// Flow:
/// 1. Generate ED25519 keypair (seed stays local)
/// 2. Connect to NATS anonymously (can only call vgate.rpc.ensoul)
/// 3. Send vgate.rpc.ensoul with public key
/// 4. Server creates identity + wallet + seeds personality
/// 5. Save seed, reconnect with Nkey auth
pub async fn onboard_vreader(
    nats_url: &str,
    template: &str,
    name: &str,
    steward: &str,
    identity_dir: &Path,
) -> Result<AgentIdentity> {
    let identity = AgentIdentity::generate()?;
    println!("🔑 Generated Nkey: {}", identity.public_key());

    // Connect anonymously (per NATS config, anonymous → vgate.rpc.ensoul only)
    let nc = async_nats::connect(nats_url).await?;

    // Call vgate.rpc.ensoul
    let req = RpcRequest {
        request_id: format!(
            "vreader-onboard-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ),
        agent_nkey: identity.public_key(),
        payload: EnsoulRequest {
            template: template.to_string(),
            name: name.to_string(),
            nkey_public: identity.public_key(),
            steward: steward.to_string(),
        },
    };

    let payload = serde_json::to_vec(&req)?;
    let response = nc
        .request(SUBJECT_ENSOUL, payload.into())
        .await?;

    let resp: RpcResponse<EnsoulResponse> = serde_json::from_slice(&response.payload)?;

    if resp.status == "error" {
        return Err(eyre::eyre!(
            "ensoul failed: {}",
            resp.error.unwrap_or_default()
        ));
    }

    let ensoul = resp.data.ok_or_else(|| eyre::eyre!("ensoul returned empty data"))?;

    println!("✅ Ensouled as: {}", ensoul.agent_nkey);

    // Save seed to disk
    identity.save(&identity_dir.join("agent.nkey"))?;

    // Print summary
    println!();
    println!("═══════════════════════════════════════════");
    println!("  VReader Agent Online");
    println!("═══════════════════════════════════════════");
    println!("  Agent ID:  {}", ensoul.agent_nkey);
    println!("  Name:      {}", ensoul.name);
    println!("  Template:  {}", ensoul.template);
    println!("  Wallet:    {}", ensoul.tb_account_id.unwrap_or(0));
    println!("  NATS:      {}", nats_url);
    println!();
    println!("  View in Gomuks: chat.vchat.email");
    println!("═══════════════════════════════════════════");

    Ok(identity)
}

/// Connect to NATS with full Nkey authentication (after onboarding).
///
/// Uses the identity's seed to authenticate via Nkey challenge-response.
pub async fn connect_authenticated(
    nats_url: &str,
    identity: &AgentIdentity,
) -> Result<async_nats::Client> {
    let seed = identity.seed_string()?;
    let kp = nkeys::KeyPair::from_seed(&seed)?;
    let pk = kp.public_key();
    let nc = async_nats::ConnectOptions::new()
        .jwt(pk, move |nonce| {
            let kp = kp.clone();
            async move {
                kp.sign(&nonce)
                    .map(|sig| sig.to_vec())
                    .map_err(|e| async_nats::AuthError::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
            }
        })
        .connect(nats_url)
        .await?;
    Ok(nc)
}
