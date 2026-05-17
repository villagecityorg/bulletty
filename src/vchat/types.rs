/// NATS RPC envelope types for vchat.email agent communication.

use serde::{Deserialize, Serialize};

/// Generic NATS RPC request wrapper.
#[derive(Debug, Serialize)]
pub struct RpcRequest<T: Serialize> {
    pub request_id: String,
    pub agent_nkey: String,
    pub payload: T,
}

/// Generic NATS RPC response wrapper.
#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Onboarding request sent to `vgate.rpc.ensoul`.
#[derive(Debug, Serialize)]
pub struct EnsoulRequest {
    pub template: String,
    pub name: String,
    pub nkey_public: String,
    pub steward: String,
}

/// Response from `vgate.rpc.ensoul`.
#[derive(Debug, Deserialize)]
pub struct EnsoulResponse {
    pub agent_nkey: String,
    pub name: String,
    pub template: String,
    pub steward: String,
    pub tb_account_id: Option<u64>,
}

/// Profile update request sent to `vgate.rpc.profile.sync`.
#[derive(Debug, Serialize)]
pub struct ProfileUpdateRequest {
    pub agent_nkey: String,
    pub soul_md: String,
    pub agent_md: String,
    pub knowledge_md: String,
}

/// Subject constants for vchat.email NATS RPC calls.
pub const SUBJECT_ENSOUL: &str = "vgate.rpc.ensoul";
pub const SUBJECT_PROFILE_SYNC: &str = "vgate.rpc.profile.sync";
