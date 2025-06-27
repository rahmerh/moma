use std::process::Command;

use anyhow::Context;

const MOMA_LOGO: &str = "/usr/share/icons/hicolor/48x48/apps/moma.png";

pub fn send_notification(text: &str) -> anyhow::Result<()> {
    Command::new("notify-send")
        .args(["-i", MOMA_LOGO, text])
        .spawn()
        .with_context(|| "Failed to send notification")?;

    Ok(())
}
