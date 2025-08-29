fn change_audio_volume(volume: i32) -> Result<(), CustomError> {
    let command = format!("Set-Volume -Volume {}", volume);
    let output = std::process::Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .output()
        .map_err(|_| CustomError::PowerShellExecutionError)?;

    if !output.status.success() {
        return Err(CustomError::PowerShellExecutionError);
    }
    Ok(())
}

fn mute_audio() -> Result<(), CustomError> {
    let command = "Set-Volume -Mute";
    let output = std::process::Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .output()
        .map_err(|_| CustomError::PowerShellExecutionError)?;

    if !output.status.success() {
        return Err(CustomError::PowerShellExecutionError);
    }
    Ok(())
}

fn unmute_audio() -> Result<(), CustomError> {
    let command = "Set-Volume -Unmute";
    let output = std::process::Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .output()
        .map_err(|_| CustomError::PowerShellExecutionError)?;

    if !output.status.success() {
        return Err(CustomError::PowerShellExecutionError);
    }
    Ok(())
}