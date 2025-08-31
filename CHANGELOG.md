# Changelog

All notable changes to the Sound Changer project will be documented in this file.

## [0.1.0] - 2025-08-31

### Added
- Initial release with Metro-style UI
- Playback and Recording device management
- Priority-based auto-switching system
- Drag & drop interface for device priority assignment
- Visual feedback with toast notifications
- AudioDeviceCmdlets module installer
- Support for Windows 10/11
- Local storage persistence of priority chains
- Auto-switch toggle functionality

### Technical Details
- Built with Tauri 2.0 and Yew 0.21
- PowerShell integration via AudioDeviceCmdlets
- InteractJS-based drag & drop implementation
- Metro UI styling with custom components
- Error handling with custom AudioError types
