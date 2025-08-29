#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_audio_manager_integration() {
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("Your-PowerShell-Command-Here")
            .output()
            .expect("Failed to execute PowerShell command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Expected Output"));
    }
}