use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;

use crate::errors::{Result, WhenThenError};

const TIMEOUT: Duration = Duration::from_secs(120);

/// Runs a trivial AppleScript targeting System Events to trigger the macOS Automation permission prompt.
#[tauri::command]
pub async fn check_automation_permission() -> Result<String> {
    let output = tokio::process::Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to return 1"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| WhenThenError::Internal(format!("Failed to spawn osascript: {e}")))?;

    if output.status.success() {
        Ok("granted".into())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("-10000") || stderr.contains("errAEEventNotPermitted") {
            Err(WhenThenError::Internal(
                "Automation permission required. Open System Settings > Privacy & Security > Automation and enable When.".into(),
            ))
        } else {
            let code = output.status.code().unwrap_or(-1);
            Err(WhenThenError::Internal(format!(
                "Automation check failed (exit {code}): {stderr}"
            )))
        }
    }
}

#[tauri::command]
pub async fn run_shortcut(name: String, input_json: String) -> Result<String> {
    let mut child = tokio::process::Command::new("shortcuts")
        .args(["run", &name, "-i", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| WhenThenError::Internal(format!("Failed to spawn shortcuts: {e}")))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input_json.as_bytes()).await;
        let _ = stdin.shutdown().await;
    }

    let output = timeout(TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| {
            WhenThenError::Internal(format!("Shortcut '{name}' timed out after 120s"))
        })?
        .map_err(|e| WhenThenError::Internal(format!("Shortcut '{name}' failed: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        Err(WhenThenError::Internal(format!(
            "Shortcut '{name}' failed (exit {code}): {stderr}"
        )))
    }
}

#[tauri::command]
pub async fn run_applescript(script: String) -> Result<String> {
    let child = tokio::process::Command::new("osascript")
        .args(["-e", &script])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| WhenThenError::Internal(format!("Failed to spawn osascript: {e}")))?;

    let output = timeout(TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| WhenThenError::Internal("AppleScript timed out after 120s".into()))?
        .map_err(|e| WhenThenError::Internal(format!("AppleScript failed: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        Err(WhenThenError::Internal(format!(
            "AppleScript failed (exit {code}): {stderr}"
        )))
    }
}

#[tauri::command]
pub async fn run_shell_command(command: String) -> Result<String> {
    let child = tokio::process::Command::new("sh")
        .args(["-c", &command])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| WhenThenError::Internal(format!("Failed to spawn shell: {e}")))?;

    let output = timeout(TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| WhenThenError::Internal("Shell command timed out after 120s".into()))?
        .map_err(|e| WhenThenError::Internal(format!("Shell command failed: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        Err(WhenThenError::Internal(format!(
            "Shell command failed (exit {code}): {stderr}"
        )))
    }
}
