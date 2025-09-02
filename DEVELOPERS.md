# Developer Guide for Sound Changer

This document provides detailed technical information for developers working on or extending Sound Changer.

## Project Structure

```
sound-changer/
├── public/               # Static assets
│   ├── drag-drop.js      # Drag & drop implementation with InteractJS
│   └── tauri.svg         # Logo and other assets
├── src/                  # Yew frontend code
│   ├── app.rs            # Main Yew application, UI components & state
│   └── main.rs           # WASM entrypoint
├── src-tauri/            # Rust backend code
│   ├── capabilities/     # Tauri capabilities configuration
│   ├── icons/            # App icons
│   └── src/              # Rust source code
│       ├── audio_manager.rs # Audio device management via PowerShell
│       ├── error.rs      # Error types and handling
│       ├── lib.rs        # Tauri commands and state
│       └── main.rs       # Application entry point
├── Cargo.toml            # Rust dependencies (frontend)
├── src-tauri/Cargo.toml  # Rust dependencies (backend)
├── index.html            # HTML template
├── styles.css            # Global styles
└── Trunk.toml            # Trunk configuration for WASM build
```

## Architecture

### Frontend (Yew/WASM)

The frontend is built with Yew, a Rust framework that compiles to WebAssembly:

- **Component Structure**: Single-page application with modular components
- **State Management**: Uses Yew's `use_state` hooks with localStorage persistence
- **Tauri Integration**: Invokes backend commands via `window.__TAURI__.core.invoke`

### Backend (Tauri/Rust)

The backend is built with Tauri, providing desktop integration:

- **Command Structure**: Exposes functions to the frontend via `#[tauri::command]` annotations
- **Audio Management**: Integrates with PowerShell's AudioDeviceCmdlets module
- **Error Handling**: Custom `AudioError` types with proper propagation

## Key Components

### Audio Manager (`audio_manager.rs`)

Responsible for:

- Executing PowerShell commands to interact with AudioDeviceCmdlets
- Enumerating audio devices
- Setting default devices
- Managing module installation and availability checks

#### PowerShell Integration Pattern

```rust
pub async fn get_devices(&self) -> AudioResult<Vec<AudioDevice>> {
    // PowerShell script embedded as string literal
    let script = r#"
        Get-AudioDevice -List | ConvertTo-Json -Depth 10
    "#;

    // Execute with retry logic
    let output = self.execute_powershell_with_retry(script).await?;

    // Parse JSON output
    // ...
}
```

### Error Handling (`error.rs`)

Custom error enum with thiserror integration:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio device not found: {0}")]
    DeviceNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),
}

pub type AudioResult<T> = Result<T, AudioError>;
```

### Tauri Commands (`lib.rs`)

Interface between frontend and backend:

```rust
#[tauri::command]
async fn set_default_device(args: SetDefaultArgs, state: State<'_, AppState>) -> AudioResult<()> {
    // Destructure args struct with serde aliases for camelCase support
    let SetDefaultArgs { device_id, device_type } = args;

    // Process command...
    state.audio_manager.set_default_device(&device_id, &device_type).await
}
```

### Drag & Drop Implementation (`drag-drop.js`)

Uses InteractJS for drag and drop functionality:

```javascript
interact(".draggable-tile").draggable({
  // Configuration...
});

interact("[data-priority-slot]").dropzone({
  accept: ".draggable-tile",
  overlap: 0.05,
  ondrop(event) {
    // Handle drop event
    // Update localStorage
    // Update UI immediately
    // Call Tauri commands to persist changes
  },
});
```

## Frontend-Backend Communication Flow

1. **UI Event** → User drags a device to a priority slot
2. **JS Handler** → `ondrop` event in InteractJS
3. **Local Update** → Update localStorage and UI immediately
4. **Backend Invoke** → Call Tauri command with device info
5. **Rust Backend** → Execute PowerShell command
6. **User Feedback** → Show toast notification based on result

## Debugging Tips

### Frontend

- Check browser console for errors
- Examine localStorage entries for priority chains
- Use toast notifications for user feedback

### Backend

- Enable Rust logging with `RUST_LOG=debug`
- Check Tauri logs in the terminal
- Use `tracing` macros for structured logging

### PowerShell

- Test AudioDeviceCmdlets commands directly in PowerShell
- Check execution policy with `Get-ExecutionPolicy -List`
- Verify module availability with `Get-Module -ListAvailable -Name AudioDeviceCmdlets`

## Common Issues

1. **Missing AudioDeviceCmdlets Module**

   - Solution: Use the in-app installer or run `Install-Module AudioDeviceCmdlets -Force -Scope CurrentUser`

2. **PowerShell Execution Policy**

   - Solution: Run `Set-ExecutionPolicy RemoteSigned -Scope CurrentUser`

3. **Parameter Naming in Tauri Commands**

   - Solution: Ensure JS payload matches Rust struct field names using the `args` wrapper object

4. **Null Reference in localStorage**
   - Solution: Use robust parsing with fallbacks and null checks

## Performance Considerations

- Cache device lists to reduce PowerShell invocations
- Batch UI updates where possible
- Use setTimeout for non-critical operations that could block the UI

## Security Considerations

- Validate all device IDs against enumerated devices before operations
- Handle PowerShell errors and timeouts gracefully
- Restrict PowerShell commands to necessary operations only
