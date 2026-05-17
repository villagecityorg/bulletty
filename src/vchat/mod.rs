/// vchat.email agent integration.
///
/// VReader does NOT embed NATS directly. The agent is a separate `vchat` CLI
/// installed via:
///   curl -sL https://vchat.email/signup | bash
///
/// This module checks for the binary, reads identity files it produces,
/// and delegates lifecycle commands to it.

pub mod agent;
pub mod identity;

/// Name of the vchat.email agent binary (installed by the signup script).
pub const VCHAT_BIN: &str = "vchat";

/// Resolved path to the vchat binary, if installed.
pub fn find_vchat_binary() -> Option<std::path::PathBuf> {
    which::which(VCHAT_BIN).ok()
}
