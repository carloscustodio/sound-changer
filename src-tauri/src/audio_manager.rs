use crate::error::{AudioError, AudioResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Performance thresholds from Step 17
const DEVICE_LISTING_TIMEOUT: Duration = Duration::from_secs(2);
const DEVICE_SWITCHING_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_BASE_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub state: DeviceState,
    pub is_default: bool,
    pub is_communication_default: bool,
    pub last_seen: Option<String>, // ISO timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Playback,
    Recording,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceState {
    Active,
    Disabled,
    NotPresent,
    Unplugged,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AudioManagerState {
    cached_devices: HashMap<String, AudioDevice>,
    last_refresh: Option<Instant>,
    cache_ttl: Duration,
    previous_default_playback: Option<String>,
    previous_default_recording: Option<String>,
}

impl Default for AudioManagerState {
    fn default() -> Self {
        Self {
            cached_devices: HashMap::new(),
            last_refresh: None,
            cache_ttl: Duration::from_secs(30), // Cache for 30 seconds
            previous_default_playback: None,
            previous_default_recording: None,
        }
    }
}

pub struct AudioManager {
    state: std::sync::Arc<tokio::sync::RwLock<AudioManagerState>>,
    session_id: String,
}

impl AudioManager {
    /// Initialize audio manager with session tracking (Step 6)
    pub fn new() -> AudioResult<Self> {
        let session_id = Uuid::new_v4().to_string();
        info!("Initializing AudioManager with session ID: {}", session_id);

        Ok(Self {
            state: std::sync::Arc::new(tokio::sync::RwLock::new(AudioManagerState::default())),
            session_id,
        })
    }

    /// Compatibility method - calls get_audio_devices
    pub async fn get_devices(&self) -> AudioResult<Vec<AudioDevice>> {
        self.get_audio_devices().await
    }

    /// Get all audio devices with caching and performance monitoring (Steps 6, 17)
    pub async fn get_audio_devices(&self) -> AudioResult<Vec<AudioDevice>> {
        let start_time = Instant::now();
        info!("Fetching audio devices (session: {})", self.session_id);

        // Check cache validity
        {
            let state = self.state.read().await;
            if let Some(last_refresh) = state.last_refresh {
                if start_time.duration_since(last_refresh) < state.cache_ttl {
                    debug!("Returning cached devices");
                    return Ok(state.cached_devices.values().cloned().collect());
                }
            }
        }

        let devices = self.fetch_devices_from_powershell().await?;

        // Update cache
        {
            let mut state = self.state.write().await;
            state.cached_devices.clear();
            for device in &devices {
                state
                    .cached_devices
                    .insert(device.id.clone(), device.clone());
            }
            state.last_refresh = Some(start_time);
        }

        let elapsed = start_time.elapsed();
        if elapsed > DEVICE_LISTING_TIMEOUT {
            warn!(
                "Device listing took {}ms, exceeds target of {}ms",
                elapsed.as_millis(),
                DEVICE_LISTING_TIMEOUT.as_millis()
            );
        }

        info!(
            "Found {} audio devices in {}ms",
            devices.len(),
            elapsed.as_millis()
        );
        Ok(devices)
    }

    /// Fetch devices from PowerShell with enhanced error handling (Steps 9, 18)
    async fn fetch_devices_from_powershell(&self) -> AudioResult<Vec<AudioDevice>> {
        let powershell_script = r#"
            try {
                # Check and import AudioDeviceCmdlets module
                if (-not (Get-Module -ListAvailable -Name AudioDeviceCmdlets)) {
                    throw "AudioDeviceCmdlets module not installed. Run: Install-Module AudioDeviceCmdlets -Force -Scope CurrentUser"
                }
                
                Import-Module AudioDeviceCmdlets -ErrorAction Stop
                
                # Get all devices with detailed information
                $allAudioDevices = Get-AudioDevice -List
                $defaultPlayback = Get-AudioDevice -Playback -ErrorAction SilentlyContinue
                $defaultRecording = Get-AudioDevice -Recording -ErrorAction SilentlyContinue
                $defaultCommunicationPlayback = Get-AudioDevice -PlaybackCommunication -ErrorAction SilentlyContinue
                $defaultCommunicationRecording = Get-AudioDevice -RecordingCommunication -ErrorAction SilentlyContinue
                
                $allDevices = @()
                
                foreach ($device in $allAudioDevices) {
                    $isDefaultPlayback = $defaultPlayback -and ($device.ID -eq $defaultPlayback.ID)
                    $isDefaultRecording = $defaultRecording -and ($device.ID -eq $defaultRecording.ID)
                    $isDefaultCommunicationPlayback = $defaultCommunicationPlayback -and ($device.ID -eq $defaultCommunicationPlayback.ID)
                    $isDefaultCommunicationRecording = $defaultCommunicationRecording -and ($device.ID -eq $defaultCommunicationRecording.ID)
                    
                    $isDefault = $isDefaultPlayback -or $isDefaultRecording
                    $isCommunicationDefault = $isDefaultCommunicationPlayback -or $isDefaultCommunicationRecording
                    
                    $allDevices += @{
                        id = $device.ID
                        name = $device.Name
                        device_type = $device.Type
                        state = $device.State
                        is_default = $isDefault
                        is_communication_default = $isCommunicationDefault
                        last_seen = (Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ")
                    }
                }
                
                # Output structured JSON
                @{
                    devices = $allDevices
                    timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ")
                    session = $env:COMPUTERNAME
                } | ConvertTo-Json -Depth 4 -Compress
            }
            catch {
                $errorInfo = @{
                    error = $_.Exception.Message
                    type = "PowerShellExecutionError"
                    timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ")
                }
                $errorInfo | ConvertTo-Json -Compress
                exit 1
            }
        "#;

        self.execute_powershell_with_retry(powershell_script, "device enumeration")
            .await
            .and_then(|output| self.parse_device_list_response(&output))
    }

    /// Set default audio device with validation and fallback (Steps 6, 7, 19)
    pub async fn set_default_device(
        &self,
        device_id: &str,
        _device_type: &DeviceType,
    ) -> AudioResult<()> {
        self.set_default_audio_device(device_id).await
    }

    /// Set default audio device with validation and fallback (Steps 6, 7, 19)
    pub async fn set_default_audio_device(&self, device_id: &str) -> AudioResult<()> {
        let start_time = Instant::now();
        info!(
            "Setting default audio device: {} (session: {})",
            device_id, self.session_id
        );

        // Validate device exists first (Step 12)
        self.validate_device_id(device_id).await?;

        // Store current default for fallback (Step 19)
        let current_defaults = self.get_current_defaults().await?;

        let result = self.change_default_device(device_id).await;

        match result {
            Ok(_) => {
                let elapsed = start_time.elapsed();
                if elapsed > DEVICE_SWITCHING_TIMEOUT {
                    warn!(
                        "Device switching took {}ms, exceeds target of {}ms",
                        elapsed.as_millis(),
                        DEVICE_SWITCHING_TIMEOUT.as_millis()
                    );
                }
                info!(
                    "Successfully set default device: {} in {}ms",
                    device_id,
                    elapsed.as_millis()
                );

                // Update cache to reflect changes
                self.invalidate_cache().await;
                Ok(())
            }
            Err(e) => {
                error!("Failed to set default device, attempting fallback");
                self.fallback_to_previous_device(&current_defaults).await?;
                Err(e)
            }
        }
    }

    /// Change audio output with device validation (Step 7)
    pub async fn change_audio_output(
        &self,
        from_device_id: &str,
        to_device_id: &str,
    ) -> AudioResult<()> {
        info!(
            "Changing audio output from {} to {} (session: {})",
            from_device_id, to_device_id, self.session_id
        );

        // Validate both devices exist
        self.validate_device_id(from_device_id).await?;
        self.validate_device_id(to_device_id).await?;

        // Verify the from_device is currently the default
        let devices = self.get_audio_devices().await?;
        let from_device = devices
            .iter()
            .find(|d| d.id == from_device_id)
            .ok_or_else(|| AudioError::DeviceNotFound(from_device_id.to_string()))?;

        if !from_device.is_default {
            return Err(AudioError::CommandFailed(format!(
                "Device {} is not currently the default",
                from_device_id
            )));
        }

        self.set_default_audio_device(to_device_id).await
    }

    /// Quick switch to device by name (Step 7)
    pub async fn quick_switch_to_device(&self, device_name: &str) -> AudioResult<()> {
        info!(
            "Quick switching to device: {} (session: {})",
            device_name, self.session_id
        );

        let devices = self.get_audio_devices().await?;
        let target_device = devices
            .iter()
            .find(|d| d.name.to_lowercase().contains(&device_name.to_lowercase()))
            .ok_or_else(|| AudioError::DeviceNotFound(device_name.to_string()))?;

        self.set_default_audio_device(&target_device.id).await
    }

    /// Validate device ID exists (Step 12)
    pub async fn validate_device_id(&self, device_id: &str) -> AudioResult<bool> {
        debug!("Validating device ID: {}", device_id);

        let devices = self.get_audio_devices().await?;
        let exists = devices.iter().any(|d| d.id == device_id);

        if !exists {
            return Err(AudioError::DeviceNotFound(device_id.to_string()));
        }

        Ok(true)
    }

    /// Check module availability with detailed diagnostics (Step 18)
    pub async fn check_module_availability(&self) -> AudioResult<bool> {
        debug!("Checking AudioDeviceCmdlets module availability...");

        let powershell_script = r#"
            try {
                $module = Get-Module -ListAvailable -Name AudioDeviceCmdlets
                if ($module) {
                    @{
                        available = $true
                        version = $module.Version.ToString()
                        path = $module.ModuleBase
                    } | ConvertTo-Json -Compress
                } else {
                    @{
                        available = $false
                        message = "AudioDeviceCmdlets module not found"
                        install_command = "Install-Module AudioDeviceCmdlets -Force -Scope CurrentUser"
                    } | ConvertTo-Json -Compress
                }
            }
            catch {
                @{
                    available = $false
                    error = $_.Exception.Message
                } | ConvertTo-Json -Compress
            }
        "#;

        let output = self
            .execute_powershell_with_retry(powershell_script, "module availability check")
            .await?;
        let response: serde_json::Value = serde_json::from_str(&output)?;

        let is_available = response["available"].as_bool().unwrap_or(false);
        debug!("AudioDeviceCmdlets module available: {}", is_available);

        if !is_available {
            if let Some(message) = response["message"].as_str() {
                info!("Module availability check: {}", message);
            }
        }

        Ok(is_available)
    }

    /// Install AudioDeviceCmdlets module (Step 9)
    pub async fn install_module(&self) -> AudioResult<()> {
        info!("Installing AudioDeviceCmdlets module...");

        let powershell_script = r#"
            try {
                # Check if running as administrator for system-wide install
                $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
                $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
                $isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
                
                if ($isAdmin) {
                    Write-Output "Installing AudioDeviceCmdlets for all users..."
                    Install-Module AudioDeviceCmdlets -Force -Scope AllUsers -AllowClobber
                } else {
                    Write-Output "Installing AudioDeviceCmdlets for current user..."
                    Install-Module AudioDeviceCmdlets -Force -Scope CurrentUser -AllowClobber
                }
                
                # Verify installation
                Import-Module AudioDeviceCmdlets -ErrorAction Stop
                $version = (Get-Module AudioDeviceCmdlets).Version
                
                @{
                    success = $true
                    version = $version.ToString()
                    scope = if ($isAdmin) { "AllUsers" } else { "CurrentUser" }
                    message = "AudioDeviceCmdlets module installed successfully"
                } | ConvertTo-Json -Compress
            }
            catch {
                @{
                    success = $false
                    error = $_.Exception.Message
                    suggestion = "Try running as administrator or check internet connection"
                } | ConvertTo-Json -Compress
                exit 1
            }
        "#;

        let output = self
            .execute_powershell_with_retry(powershell_script, "module installation")
            .await?;
        let response: serde_json::Value = serde_json::from_str(&output)?;

        if response["success"].as_bool().unwrap_or(false) {
            info!("Successfully installed AudioDeviceCmdlets module");
        } else {
            let error_msg = response["error"]
                .as_str()
                .unwrap_or("Unknown installation error");
            return Err(AudioError::CommandFailed(error_msg.to_string()));
        }

        Ok(())
    }

    /// Execute PowerShell with retry logic (Step 20)
    async fn execute_powershell_with_retry(
        &self,
        script: &str,
        operation: &str,
    ) -> AudioResult<String> {
        let mut last_error = None;

        for attempt in 1..=MAX_RETRY_ATTEMPTS {
            debug!(
                "Executing PowerShell {} (attempt {}/{})",
                operation, attempt, MAX_RETRY_ATTEMPTS
            );

            let result = Command::new("powershell")
                .args(&[
                    "-ExecutionPolicy",
                    "Bypass",
                    "-NoProfile",
                    "-Command",
                    script,
                ])
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        debug!("PowerShell {} succeeded on attempt {}", operation, attempt);
                        return Ok(stdout.to_string());
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        last_error = Some(AudioError::CommandFailed(stderr.to_string()));
                        warn!(
                            "PowerShell {} failed on attempt {}: {}",
                            operation, attempt, stderr
                        );
                    }
                }
                Err(e) => {
                    last_error = Some(AudioError::from(e));
                    warn!(
                        "PowerShell execution error on attempt {}: {:?}",
                        attempt, last_error
                    );
                }
            }

            if attempt < MAX_RETRY_ATTEMPTS {
                let delay = RETRY_BASE_DELAY * attempt;
                debug!("Retrying {} in {}ms", operation, delay.as_millis());
                tokio::time::sleep(delay).await;
            }
        }

        error!(
            "PowerShell {} failed after {} attempts",
            operation, MAX_RETRY_ATTEMPTS
        );
        Err(last_error
            .unwrap_or_else(|| AudioError::CommandFailed("Unknown PowerShell error".to_string())))
    }

    /// Parse device list response from PowerShell
    fn parse_device_list_response(&self, json_output: &str) -> AudioResult<Vec<AudioDevice>> {
        let response: serde_json::Value = serde_json::from_str(json_output)?;

        if let Some(error) = response.get("error") {
            return Err(AudioError::CommandFailed(
                error.as_str().unwrap_or("Unknown error").to_string(),
            ));
        }

        let devices_array = response["devices"]
            .as_array()
            .ok_or_else(|| AudioError::ParseError("Missing devices array".to_string()))?;

        let mut audio_devices = Vec::new();
        for device in devices_array {
            let device_type = match device["device_type"].as_str() {
                Some("Playback") => DeviceType::Playback,
                Some("Recording") => DeviceType::Recording,
                _ => continue,
            };

            let state = match device["state"].as_str() {
                Some("Active") => DeviceState::Active,
                Some("Disabled") => DeviceState::Disabled,
                Some("NotPresent") => DeviceState::NotPresent,
                Some("Unplugged") => DeviceState::Unplugged,
                _ => DeviceState::Unknown,
            };

            audio_devices.push(AudioDevice {
                id: device["id"].as_str().unwrap_or("").to_string(),
                name: device["name"].as_str().unwrap_or("").to_string(),
                device_type,
                state,
                is_default: device["is_default"].as_bool().unwrap_or(false),
                is_communication_default: device["is_communication_default"]
                    .as_bool()
                    .unwrap_or(false),
                last_seen: device["last_seen"].as_str().map(|s| s.to_string()),
            });
        }

        Ok(audio_devices)
    }

    /// Change default device implementation
    async fn change_default_device(&self, device_id: &str) -> AudioResult<()> {
        let powershell_script = format!(
            r#"
                try {{
                    Import-Module AudioDeviceCmdlets -ErrorAction Stop
                    
                    $device = Get-AudioDevice -List | Where-Object {{ $_.ID -eq "{}" }}
                    
                    if (-not $device) {{
                        throw "Device not found: {}"
                    }}
                    
                    # Set as default for both regular and communication
                    if ($device.Type -eq "Playback") {{
                        Set-AudioDevice -ID "{}" -DefaultOnly
                        Set-AudioDevice -ID "{}" -CommunicationOnly
                    }} else {{
                        Set-AudioDevice -ID "{}" -DefaultOnly
                        Set-AudioDevice -ID "{}" -CommunicationOnly
                    }}
                    
                    @{{
                        success = $true
                        device_id = "{}"
                        device_name = $device.Name
                        device_type = $device.Type
                    }} | ConvertTo-Json -Compress
                }}
                catch {{
                    @{{
                        success = $false
                        error = $_.Exception.Message
                        device_id = "{}"
                    }} | ConvertTo-Json -Compress
                    exit 1
                }}
            "#,
            device_id, device_id, device_id, device_id, device_id, device_id, device_id, device_id
        );

        self.execute_powershell_with_retry(&powershell_script, "set default device")
            .await?;
        Ok(())
    }

    /// Get current default devices for fallback
    async fn get_current_defaults(&self) -> AudioResult<(Option<String>, Option<String>)> {
        let devices = self.get_audio_devices().await?;

        let default_playback = devices
            .iter()
            .find(|d| matches!(d.device_type, DeviceType::Playback) && d.is_default)
            .map(|d| d.id.clone());

        let default_recording = devices
            .iter()
            .find(|d| matches!(d.device_type, DeviceType::Recording) && d.is_default)
            .map(|d| d.id.clone());

        Ok((default_playback, default_recording))
    }

    /// Fallback to previous device on failure
    async fn fallback_to_previous_device(
        &self,
        defaults: &(Option<String>, Option<String>),
    ) -> AudioResult<()> {
        warn!("Attempting to fallback to previous default devices");

        if let Some(playback_id) = &defaults.0 {
            match self.change_default_device(playback_id).await {
                Ok(_) => info!(
                    "Successfully restored previous playback device: {}",
                    playback_id
                ),
                Err(e) => error!("Failed to restore previous playback device: {}", e),
            }
        }

        if let Some(recording_id) = &defaults.1 {
            match self.change_default_device(recording_id).await {
                Ok(_) => info!(
                    "Successfully restored previous recording device: {}",
                    recording_id
                ),
                Err(e) => error!("Failed to restore previous recording device: {}", e),
            }
        }

        Ok(())
    }

    /// Invalidate device cache
    async fn invalidate_cache(&self) {
        let mut state = self.state.write().await;
        state.last_refresh = None;
        state.cached_devices.clear();
        debug!("Audio device cache invalidated");
    }

    /// Get session ID for debugging
    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }
}
