# Copilot Instructions for sound-changer

## Project Overview

- Windows-specific desktop app for audio device management using Tauri (Rust backend) + Yew (WASM frontend)
- **Critical Requirement**: Windows 10/11 with PowerShell 5.1+ and AudioDeviceCmdlets module
- Metro-style UI with drag-and-drop device switching via `subwayui` crate
- Production-ready architecture with comprehensive error handling and validation

## Core Architecture

- `src-tauri/src/audio_manager.rs`: AudioDeviceCmdlets integration, device validation, retry logic, also the de-facto audio manager
- `src-tauri/src/error.rs`: Custom `AudioError` enum with proper error propagation via `thiserror`
- `src-tauri/src/lib.rs`: Tauri commands with `State<AudioManagerState>` for thread-safe operations
- `src/app.rs`: Yew frontend with Metro tiles, drag-drop events, and WASM-JS bridge calls

## Critical Dependencies & Setup

**Required Cargo.toml dependencies:**

```toml
tauri = { version = "2.0", features = [] }
windows = { version = "0.59", features = ["Win32_Foundation", "Win32_System_Com", "Win32_Media_Audio"] }
tokio = { version = "1.0", features = ["full"] }
thiserror = "1.0"
tracing = "0.1"
uuid = { version = "1.0", features = ["v4"] }
```

**PowerShell Prerequisites:**

- AudioDeviceCmdlets module: `Install-Module AudioDeviceCmdlets -Force -Scope CurrentUser`
- Execution policy: Must allow script execution for embedded PowerShell commands
- System validation: Check `$PSVersionTable.PSVersion` ≥ 5.1

## Implementation Patterns

## Implementation Patterns

**PowerShell Integration:**

- Embed PowerShell scripts as Rust string literals in `audio_manager.rs`
- Use `Command::new("powershell").args(&["-ExecutionPolicy", "Bypass", "-Command", script])`
- Parse JSON output from PowerShell AudioDeviceCmdlets for device enumeration
- Implement retry logic (3 attempts) for transient Windows audio system conflicts

**Error Handling Strategy:**

- Custom `AudioError` enum covers: DeviceNotFound, PermissionDenied, CommandFailed, ParseError
- Use `AudioResult<T> = Result<T, AudioError>` throughout backend
- Frontend displays user-friendly messages while logging technical details
- Validate device IDs against enumerated devices before operations

**State Management:**

- `Arc<AudioManager>` in Tauri app state for thread-safe access
- Device operations are async with proper error propagation
- Frontend caches device list and refreshes on audio changes

## Build & Development Workflow

## Build & Development Workflow

- **Environment Check**: Verify PowerShell 5.1+, Rust, and AudioDeviceCmdlets module first
- **Development**: `trunk serve` (frontend) + `cargo tauri dev` (full app testing)
- **Production**: `trunk build` then `cargo tauri build` for distribution
- **Testing**: Unit tests in `src-tauri/src/` + PowerShell integration tests in `tests/`
- **Validation**: Always test device enumeration and switching on target Windows versions

## Project-Specific Conventions

## Project-Specific Conventions

- PowerShell scripts are embedded as Rust string literals and invoked with `Command::new("powershell")`
- All device state is passed as JSON between PowerShell and Rust, then (de)serialized with Serde
- Device IDs, not names, are used for backend operations to avoid ambiguity
- Drag-and-drop events are custom-dispatched in JS and handled in Yew via event listeners
- Default device tiles are visually distinct (see `.default-device` CSS class)

## Integration Points

- Requires PowerShell 5.1+ and the AudioDeviceCmdlets module installed on Windows
- Uses the `windows` crate for future native Windows API integration (currently PowerShell-based)
- Yew Metro UI depends on the `subwayui` crate (custom fork)

## Example: Adding a New Audio Command

1. Implement the logic in `audio_manager.rs` (e.g., new PowerShell script)
2. Register the command in `lib.rs` and expose via Tauri
3. Call the command from Yew using the JS `invoke` bridge
4. Update the UI and event handlers as needed

## References

- See `Step-by-Step Guide for AI Agent_ Implementing Taur.md` for a detailed implementation plan
- For Metro UI conventions, see `styles.css` and `subwayui` usage in `app.rs`

---

If any section is unclear or missing, please request clarification or provide feedback for further iteration.
