# Sound Changer

![Sound Changer](public/tauri.svg)

A modern, Metro-styled Windows audio device manager built with Tauri (Rust) and Yew (WebAssembly). Manage your audio devices with an intuitive, priority-based switching system.

## Features

- **Metro UI Interface**: Clean, intuitive Windows Metro-style design
- **Priority Chain System**: Create custom priority chains for playback and recording devices
- **Drag & Drop Management**: Effortlessly assign device priorities with interactive drag and drop
- **Auto-Switching**: Automatically switch to highest priority available device
- **Visual Status Feedback**: Clear visual indicators for default/active devices
- **Toast Notifications**: Real-time feedback for operations
- **PowerShell Integration**: Deep Windows audio system integration via AudioDeviceCmdlets

## Prerequisites

- **Windows 10/11** (required)
- **PowerShell 5.1+** (verify with `$PSVersionTable.PSVersion`)
- **AudioDeviceCmdlets Module** (app can install if missing)
- **Execution Policy**: Must allow script execution (`Set-ExecutionPolicy RemoteSigned -Scope CurrentUser`)

## Installation

1. Download the latest release from the [Releases](https://github.com/carloscustodio/sound-changer/releases) page
2. Run the installer
3. Launch Sound Changer from your Start Menu
4. If prompted, allow the app to install the AudioDeviceCmdlets module

## Building from Source

### Prerequisites

- Rust 1.70+ (`rustup update`)
- Tauri CLI (`cargo install tauri-cli`)
- Node.js 18+ (for Trunk)
- Trunk (`cargo install trunk`)

### Development

```powershell
# Clone the repository
git clone https://github.com/carloscustodio/sound-changer.git
cd sound-changer

# Install AudioDeviceCmdlets module if not already installed
Install-Module AudioDeviceCmdlets -Force -Scope CurrentUser

# Start the development server
cargo tauri dev
```

### Production Build

```powershell
# Build for production
cargo tauri build
```

The packaged application will be available in `target/release/bundle/`.

## Usage Guide

### Main Interface

The main interface is divided into four sections:

1. **Available Playback Devices**: List of output devices (speakers, headphones)
2. **Playback Priority Chain**: Configure priority order for output devices
3. **Available Recording Devices**: List of input devices (microphones)
4. **Recording Priority Chain**: Configure priority order for input devices

### Creating a Priority Chain

1. **Drag and Drop**: Drag a device from the available devices list to a slot in the priority chain
2. **Assign Priority**: Higher slots (1-5) have higher priority
3. **Auto-Switch**: Toggle auto-switching to automatically select the highest priority available device
4. **Remove Assignment**: Click the 'X' button to remove a device from the chain

### Module Management

If the AudioDeviceCmdlets module is not installed, the app will display a warning and provide an "Install now" button to install it automatically.

## Architecture

- **Frontend**: Yew (Rust-based WASM framework)
- **Backend**: Tauri with Rust
- **Audio Management**: PowerShell AudioDeviceCmdlets integration
- **Drag & Drop**: Interactive JS with robust event handling
- **State Management**: Local storage for persistence, backend integration for audio routing

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Distributed under the MIT License. See `LICENSE` for more information.

## Acknowledgments

- [Tauri](https://tauri.app/) - For the desktop app framework
- [Yew](https://yew.rs/) - For the Rust/WASM frontend framework
- [AudioDeviceCmdlets](https://github.com/frgnca/AudioDeviceCmdlets) - For Windows audio device management
- [SubwayUI](https://github.com/carloscustodio/subwayui) - For Metro-style UI components
- [InteractJS](https://interactjs.io/) - For drag and drop functionality
