/// vchat.email agent lifecycle — delegates to the external Go binary.
use std::path::Path;

use color_eyre::Result;

/// Run onboarding via the vchat binary, or print instructions.
pub fn onboard(identity_dir: &Path, steward: &str) -> Result<()> {
    let bin = super::find_vchat_binary();

    if let Some(bin_path) = bin {
        println!("🚀 Running vchat onboarding...");

        let status = std::process::Command::new(bin_path)
            .arg("onboard")
            .arg("--steward")
            .arg(steward)
            .arg("--identity-dir")
            .arg(identity_dir)
            .status()?;

        if status.success() {
            println!("✅ Onboarding complete.");
            println!("  Identity: {}", identity_dir.join("agent.nkey").display());
        } else {
            println!("⚠ Onboarding command exited with status: {status}");
        }
    } else {
        println!();
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║  vchat not installed                                    ║");
        println!("╠══════════════════════════════════════════════════════╣");
        println!("║  Run the one-liner installer first:                ║");
        println!("║                                                    ║");
        println!("║    curl -sL https://vchat.email/signup | bash      ║");
        println!("║                                                    ║");
        println!("║  Then re-run:                                      ║");
        println!("║    vreader vchat-onboard --steward your@email.com  ║");
        println!("╚══════════════════════════════════════════════════════╝");
        println!();
    }

    Ok(())
}

/// Show agent status by checking for the identity file.
pub fn status(identity_dir: &Path) -> Result<()> {
    let bin = super::find_vchat_binary();

    println!("vchat.email Agent Status");
    println!("──────────────────────────────");

    match &bin {
        Some(p) => println!("  Agent bin: ✅ {} ({})", super::VCHAT_BIN, p.display()),
        None => println!("  Agent bin: ❌ not installed"),
    }

    let seed_path = identity_dir.join("agent.nkey");
    if seed_path.exists() {
        match crate::vchat::identity::get_public_key(identity_dir) {
            Ok(Some(pk)) => println!("  Identity:  ✅ {}", pk),
            Ok(None) => println!("  Identity:  ⚠ seed exists but unreadable"),
            Err(e) => println!("  Identity:  ⚠ error: {e}"),
        }
    } else {
        println!("  Identity:  ❌ not registered");
    }

    println!("  Seed path: {}", seed_path.display());

    if seed_path.exists() {
        println!();
        println!("  Status: ✅ Agent configured");
    } else if bin.is_none() {
        println!();
        println!("  Run the installer:");
        println!("    curl -sL https://vchat.email/signup | bash");
        println!("  Then:");
        println!("    vreader vchat-onboard --steward your@email.com");
    } else {
        println!();
        println!("  Run onboarding:");
        println!("    vreader vchat-onboard --steward your@email.com");
    }

    Ok(())
}
