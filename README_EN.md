# Mira - Desktop Camera Sprite 🎥

<div align="center">

**A modern desktop camera application with real-time shape masks and flexible window interactions**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](#system-requirements)
[![Build Status](https://github.com/Vogadero/Mira/workflows/Build%20and%20Release/badge.svg)](https://github.com/Vogadero/Mira/actions)
[![Release](https://img.shields.io/github/v/release/Vogadero/Mira)](https://github.com/Vogadero/Mira/releases)

English | [简体中文](README.md)

[Features](#features) • [Quick Start](#quick-start) • [Usage](#usage) • [Development](#development) • [Contributing](#contributing)

</div>

## Overview

Mira is a cross-platform desktop camera application developed in Rust, featuring real-time video streaming, multiple shape masks, flexible window interactions, and high-performance GPU rendering. Whether for video conferencing, live streaming, or content creation, Mira adds creativity and fun to your camera feed.

### Why Mira?

- 🚀 **High Performance**: GPU-based rendering maintaining 30+ FPS
- 🎨 **Creative Shapes**: 5 built-in shape masks (Circle, Ellipse, Rectangle, Rounded Rectangle, Heart)
- 🖱️ **Flexible Interaction**: Drag to move, scroll to zoom, Ctrl+scroll to rotate
- 🎯 **System Tray**: Convenient tray icon with context menu
- 💾 **Smart Memory**: Auto-save window position, size, and settings
- 🔧 **Lightweight**: Installation < 25MB, Memory usage < 200MB
- 🌍 **Cross-Platform**: Windows 10+, macOS 11+, and Linux support

## Features

### ✅ Implemented Features

#### 🎥 Camera Management
- ✅ Multi-device support: Auto-detect and support multiple cameras
- ✅ Smart switching: One-key switch between cameras (Tab key)
- ✅ Error recovery: Auto-handle device disconnection and reconnection
- ✅ Permission management: Friendly permission prompts and error handling

#### 🪟 Window Interaction
- ✅ Always on top: Stay above all windows
- ✅ Transparent background: Perfect desktop integration
- ✅ Drag to move: Left-click drag to move window (optimized, no drift)
- ✅ Smart constraints: Ensure at least 20% of window stays on screen

#### 🎨 Shape Masks
- ✅ 5 preset shapes: Circle, Ellipse, Rectangle, Rounded Rectangle, Heart
- ✅ Quick switch: F1-F5 keys for quick shape switching, Space for cycling
- ✅ Real-time rendering: Shape switching < 100ms
- ✅ Adaptive adjustment: Masks auto-adapt to window size changes

#### 🖱️ Zoom and Rotation
- ✅ Scroll zoom: Mouse wheel for ±10% precise zooming
- ✅ Rotation: Ctrl + scroll for ±15° rotation
- ✅ Tray rotation: Rotate window via tray menu
- ✅ Smart alignment: Auto-align to 0°, 90°, 180°, 270° (±5° range)
- ✅ Size limits: Minimum 100x100, maximum 80% of screen

#### 🎯 System Tray
- ✅ Tray icon: Blue circular icon representing camera lens
- ✅ Context menu: Complete feature menu
  - Shape selection (5 shapes)
  - Window controls (reset position, rotation, size)
  - Rotation controls (clockwise/counterclockwise 15°)
  - Show info
  - Quit application
- ✅ Cross-platform: Windows, macOS, Linux

#### ⚙️ Configuration Management
- ✅ Auto-save: Save window state and settings on exit
- ✅ Cross-platform config: Use platform-standard config paths
- ✅ Config recovery: Auto-restore last window state on startup
- ✅ Error handling: Use default config if config file is corrupted

#### 📊 Performance Optimization
- ✅ GPU acceleration: High-performance rendering with wgpu
- ✅ Memory management: Smart memory pools and texture caching
- ✅ Performance monitoring: Real-time FPS, CPU, and memory monitoring
- ✅ Resource cleanup: Auto-cleanup of unused resources

### 🚧 Planned Features

- ⏳ Custom tray icons
- ⏳ Multi-language support (English, Japanese, etc.)
- ⏳ More shape masks (Star, Polygon, etc.)
- ⏳ Filter effects (B&W, Vintage, etc.)
- ⏳ Recording functionality
- ⏳ Screenshot functionality
- ⏳ Virtual camera support

## Quick Start

### System Requirements

| Platform | Minimum | Recommended |
|----------|---------|-------------|
| Windows | Windows 10 (1903+) | Windows 11 |
| macOS | macOS 11 (Big Sur) | macOS 12+ |
| Linux | Ubuntu 20.04+ | Ubuntu 22.04+ |
| RAM | 4GB | 8GB+ |
| GPU | DirectX 11/Metal/Vulkan | Dedicated GPU |
| Camera | Any USB/Built-in | 1080p+ |

### Download and Install

#### Option 1: Download Pre-built Binaries (Recommended)

Visit the [Releases page](https://github.com/Vogadero/Mira/releases) to download the latest version:

- **Windows**: `mira-windows-x64.zip`
- **macOS**: `mira-macos-x64.tar.gz`
- **Linux**: `mira-linux-x64.tar.gz`

#### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/Vogadero/Mira.git
cd Mira

# Windows
.\scripts\build_release.ps1

# macOS/Linux
chmod +x scripts/build_release.sh
./scripts/build_release.sh
```

For detailed environment setup, see [SETUP_GUIDE.md](SETUP_GUIDE.md).

## Usage

### Basic Operations

| Operation | Method | Description |
|-----------|--------|-------------|
| **Move Window** | Left-click drag | Hold left button and drag anywhere on window |
| **Zoom Window** | Mouse wheel | Scroll up to zoom in, down to zoom out |
| **Rotate Window** | Ctrl + wheel | Hold Ctrl and scroll mouse wheel |
| **Switch Shape** | F1-F5 keys | F1=Circle, F2=Ellipse, F3=Rectangle, F4=Rounded, F5=Heart |
| **Cycle Shapes** | Space key | Press Space to cycle through all shapes |
| **Switch Camera** | Tab key | Switch between multiple camera devices |
| **Tray Menu** | Right-click tray icon | Show complete feature menu |
| **Quit App** | Tray menu -> Quit | Or close window directly |

### System Tray Features

Right-click the blue circular icon in the system tray to access:

- **Shape Selection**: Quick switch between 5 shapes
- **Window Controls**: Reset position, rotation, size
- **Rotation Controls**: Rotate clockwise/counterclockwise by 15°
- **Show Info**: Display current status in console
- **Quit**: Close application

For detailed tray functionality, see [TRAY_ICON_GUIDE.md](TRAY_ICON_GUIDE.md).

### Keyboard Shortcuts

```
Movement:
  Left-click drag  - Move window
  Mouse wheel      - Zoom (±10%)
  Ctrl + wheel     - Rotate (±15°)

Shape Switching:
  F1              - Circle
  F2              - Ellipse
  F3              - Rectangle
  F4              - Rounded Rectangle
  F5              - Heart
  Space           - Cycle through shapes

Device Management:
  Tab             - Switch camera device
  
System:
  Right-click tray - Show menu
  Tray menu->Quit  - Quit application
```

## Development

### Project Structure

```
mira/
├── src/
│   ├── main.rs              # Application entry point
│   ├── camera/              # Camera management module
│   ├── window/              # Window management module
│   ├── render/              # GPU rendering module
│   ├── shape/               # Shape mask module
│   ├── config/              # Configuration module
│   ├── tray.rs              # System tray module
│   ├── event.rs             # Event handler
│   └── ...
├── docs/                    # Technical documentation
├── scripts/                 # Build scripts
├── tests/                   # Test files
└── examples/                # Example code
```

### Tech Stack

- **Language**: Rust 1.75+
- **Windowing**: winit 0.29
- **Rendering**: wgpu 0.20
- **Camera**: nokhwa 0.10
- **Tray**: tray-icon 0.14
- **Config**: serde + toml

### Build and Test

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Benchmarks
cargo bench

# Linting
cargo clippy

# Formatting
cargo fmt
```

### Performance Metrics

| Metric | Target | Actual |
|--------|--------|--------|
| **FPS** | ≥ 30 FPS | 30-60 FPS |
| **Startup** | < 3s | 1-2s |
| **Memory** | < 200 MB | 100-150 MB |
| **CPU** | < 25% | 10-20% |
| **Response** | < 16 ms | 8-12 ms |

## Contributing

We welcome all forms of contributions!

### How to Contribute

1. Fork the project
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Code Standards

- Format code with `cargo fmt`
- Check code quality with `cargo clippy`
- Write tests for new features
- Update relevant documentation

## Troubleshooting

### Common Issues

**Q: No camera feed after starting the app?**
A: Please check:
1. Is the camera being used by another application?
2. Have you granted camera permissions?
3. Are camera drivers working properly?

**Q: Can't find the system tray icon?**
A: 
- Windows: Click the "^" in the taskbar bottom-right to expand hidden icons
- macOS: Check the menu bar in the top-right corner
- Linux: Ensure your system supports tray icons

**Q: Window dragging is not smooth?**
A: Please check:
1. Are graphics drivers up to date?
2. Are system resources sufficient?
3. Close other GPU-intensive applications

For more issues, see [Issues](https://github.com/Vogadero/Mira/issues).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Thanks to these open-source projects:

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [wgpu](https://wgpu.rs/) - Modern GPU API
- [winit](https://github.com/rust-windowing/winit) - Window management library
- [nokhwa](https://github.com/l1npengtul/nokhwa) - Camera capture library
- [tray-icon](https://github.com/tauri-apps/tray-icon) - System tray library

## Contact

- 📧 Email: 15732651140@163.com
- 🐙 GitHub: https://github.com/Vogadero/Mira
- 💬 Discussions: [GitHub Discussions](https://github.com/Vogadero/Mira/discussions)

---

<div align="center">

**If Mira helps you, please give us a ⭐ Star!**

Made with ❤️ by the Mira Team

</div>
