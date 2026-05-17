/// Personality profile sync for vchat.email.

use std::path::Path;

use color_eyre::{Result, eyre};

use crate::vchat::types::*;

const SOUL_MD: &str = "SOUL.md";
const AGENT_MD: &str = "AGENT.md";

/// Sync local personality files to vchat.email.
pub async fn sync_personality(
    nc: &async_nats::Client,
    agent_nkey: &str,
    vault_path: &Path,
) -> Result<()> {
    let soul = read_file_or_default(vault_path.join(SOUL_MD), "# VReader Agent\n\nA family RSS reader agent.");
    let agent = read_file_or_default(vault_path.join(AGENT_MD), "# Capabilities\n\n- Feed curation\n- Article summarization\n- Kid-safe content filtering");

    let req = RpcRequest {
        request_id: format!("sync-{}", timestamp_ns()),
        agent_nkey: agent_nkey.to_string(),
        payload: ProfileUpdateRequest {
            agent_nkey: agent_nkey.to_string(),
            soul_md: soul,
            agent_md: agent,
            knowledge_md: String::new(),
        },
    };

    let payload = serde_json::to_vec(&req)?;
    let response = nc
        .request(SUBJECT_PROFILE_SYNC, payload.into())
        .await?;

    let resp: RpcResponse<serde_json::Value> = serde_json::from_slice(&response.payload)?;

    if resp.status == "ok" {
        println!("✅ Personality synced to vchat.email");
    } else {
        return Err(eyre::eyre!(
            "Profile sync failed: {}",
            resp.error.unwrap_or_default()
        ));
    }

    Ok(())
}

fn read_file_or_default(path: std::path::PathBuf, default: &str) -> String {
    std::fs::read_to_string(&path).unwrap_or_else(|_| default.to_string())
}

fn timestamp_ns() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
