# Mira - 桌面摄像精灵 🎥

<div align="center">

**一个现代化的桌面摄像头应用，支持实时形状遮罩和窗口交互**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey)](#系统要求)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#构建状态)

[功能特性](#功能特性) • [快速开始](#快速开始) • [安装指南](#安装指南) • [使用说明](#使用说明) • [开发文档](#开发文档)

</div>

## 概述

Mira 是一个使用 Rust 开发的跨平台桌面摄像头应用，提供实时视频流显示、多种形状遮罩、灵活的窗口交互和高性能 GPU 渲染。无论是视频会议、直播还是内容创作，Mira 都能为你的摄像头画面增添创意和趣味。

### 为什么选择 Mira？

- 🚀 **高性能**: 基于 GPU 渲染，保持 30+ FPS 流畅体验
- 🎨 **创意形状**: 5 种内置形状遮罩（圆形、椭圆、矩形、圆角矩形、心形）
- 🖱️ **灵活交互**: 拖拽移动、滚轮缩放、Ctrl+滚轮旋转
- 💾 **智能记忆**: 自动保存窗口位置、大小和设置
- 🔧 **轻量级**: 安装包小于 25MB，内存占用低于 200MB
- 🌍 **跨平台**: 支持 Windows 10+ 和 macOS 11+

## 功能特性

### 🎥 摄像头管理
- **多设备支持**: 自动检测并支持多个摄像头设备
- **智能切换**: 一键切换不同摄像头（Tab 键）
- **错误恢复**: 自动处理设备断开和重连
- **权限管理**: 友好的权限提示和错误处理

### 🎄 窗口交互
- **置顶显示**: 始终保持在所有窗口之上
- **透明背景**: 支持透明背景，完美融入桌面
- **拖拽移动**: 左键拖拽自由移动窗口位置
- **智能约束**: 确保窗口至少 20% 区域在屏幕内

### 🎨 形状遮罩
- **5 种预设形状**: 圆形、椭圆形、矩形、圆角矩形、心形
- **快速切换**: F1-F5 键快速切换形状，Space 键循环切换
- **实时渲染**: 形状切换时间小于 100ms
- **自适应调整**: 遮罩自动适应窗口尺寸变化

### 🖱️ 缩放和旋转
- **滚轮缩放**: 鼠标滚轮进行 ±10% 精确缩放
- **旋转功能**: Ctrl + 滚轮进行 ±15° 旋转
- **智能对齐**: 自动对齐到 0°、90°、180°、270°（±5° 范围内）
- **尺寸限制**: 最小 100x100，最大屏幕的 80%

### ⚙️ 配置管理
- **自动保存**: 退出时自动保存窗口状态和设置
- **跨平台配置**: Windows 和 macOS 使用各自标准配置路径
- **配置恢复**: 启动时自动恢复上次的窗口状态
- **错误处理**: 配置文件损坏时自动使用默认配置

### 📊 性能优化
- **GPU 加速**: 使用 wgpu 进行高性能渲染
- **内存管理**: 智能内存池和纹理缓存
- **性能监控**: 实时监控 FPS、CPU 和内存使用
- **资源清理**: 自动清理未使用的资源

## 快速开始

### 系统要求

| 平台 | 最低版本 | 推荐配置 |
|------|----------|----------|
| Windows | Windows 10 (1903+) | Windows 11 |
| macOS | macOS 11 (Big Sur) | macOS 12+ |
| 内存 | 4GB RAM | 8GB+ RAM |
| 显卡 | 支持 DirectX 11/Metal | 独立显卡 |
| 摄像头 | 任意 USB/内置摄像头 | 1080p+ 摄像头 |

### 下载安装

#### Windows 用户

1. **下载安装包**
   ```
   下载 mira-windows-x64.zip
   解压到任意目录
   ```

2. **运行应用**
   ```cmd
   # 双击运行
   mira.exe
   
   # 或命令行运行
   .\mira.exe
   ```

#### macOS 用户

1. **下载安装包**
   ```bash
   # 下载并解压
   curl -L -o mira-macos-x64.tar.gz [下载链接]
   tar -xzf mira-macos-x64.tar.gz
   ```

2. **运行应用**
   ```bash
   # 给予执行权限
   chmod +x mira
   
   # 运行应用
   ./mira
   ```

### 首次使用

1. **摄像头权限**: 首次运行时会请求摄像头权限，请点击"允许"
2. **窗口出现**: 应用会显示一个置顶的摄像头窗口
3. **开始使用**: 尝试拖拽、缩放和切换形状

## 使用说明

### 基本操作

| 操作 | 方法 | 说明 |
|------|------|------|
| **移动窗口** | 左键拖拽 | 在窗口任意位置按住左键拖拽 |
| **缩放窗口** | 鼠标滚轮 | 向上滚动放大，向下滚动缩小 |
| **旋转窗口** | Ctrl + 滚轮 | 按住 Ctrl 键同时滚动鼠标滚轮 |
| **切换形状** | F1-F5 键 | F1圆形，F2椭圆，F3矩形，F4圆角矩形，F5心形 |
| **循环形状** | Space 键 | 按 Space 键循环切换所有形状 |
| **切换摄像头** | Tab 键 | 在多个摄像头设备间切换 |
| **关闭应用** | Alt + F4 | 或点击窗口关闭按钮 |

### 快捷键参考

```
移动操作:
  左键拖拽        - 移动窗口
  鼠标滚轮        - 缩放 (±10%)
  Ctrl + 滚轮     - 旋转 (±15°)

形状切换:
  F1             - 圆形
  F2             - 椭圆形  
  F3             - 矩形
  F4             - 圆角矩形
  F5             - 心形
  Space          - 循环切换

设备管理:
  Tab            - 切换摄像头设备
  
系统操作:
  Alt + F4       - 退出应用
```

### 高级功能

#### 自动对齐
当旋转角度接近 0°、90°、180°、270° 时（±5° 范围内），窗口会自动对齐到精确角度。

#### 边界约束
拖拽窗口时，系统会确保至少 20% 的窗口区域保持在屏幕内，防止窗口完全移出屏幕。

#### 配置持久化
应用会自动保存以下设置：
- 窗口位置和大小
- 旋转角度
- 当前形状
- 摄像头设备选择

## 安装指南

### 从源码构建

如果你想从源码构建 Mira，请参考 [SETUP_GUIDE.md](SETUP_GUIDE.md) 获取详细的环境配置指南。

#### 快速构建步骤

1. **安装 Rust**
   ```bash
   # 安装 Rust 工具链
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆项目**
   ```bash
   git clone https://github.com/Vogadero/Mira.git
   cd mira
   ```

3. **构建项目**
   ```bash
   # Windows
   .\scripts\build_release.ps1
   
   # macOS/Linux  
   ./scripts/build_release.sh
   ```

### 开发环境

详细的开发环境配置请参考：
- [SETUP_GUIDE.md](SETUP_GUIDE.md) - 完整环境配置指南
- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南
- [docs/](docs/) - 技术文档

## 故障排除

### 常见问题

**Q: 应用启动后看不到摄像头画面？**
A: 请检查：
1. 摄像头是否被其他应用占用
2. 是否授予了摄像头权限
3. 摄像头驱动是否正常

**Q: 窗口拖拽不流畅？**
A: 请检查：
1. 显卡驱动是否最新
2. 系统资源是否充足
3. 关闭其他占用 GPU 的应用

**Q: 形状切换很慢？**
A: 请检查：
1. 内存使用是否过高
2. CPU 占用是否正常
3. 尝试重启应用

**Q: 配置无法保存？**
A: 请检查：
1. 配置目录是否有写入权限
2. 磁盘空间是否充足

### 获取帮助

- 📖 查看 [文档](docs/)
- 🐛 报告 [Bug](https://github.com/Vogadero/Mira/issues)
- 💬 参与 [讨论](https://github.com/Vogadero/Mira/discussions)
- 📧 联系开发者

## 开发文档

### 项目架构

Mira 采用模块化架构设计，每个模块职责明确，便于维护和扩展。

```
mira/
├── src/
│   ├── main.rs              # 应用程序入口和主循环
│   ├── lib.rs               # 库入口，导出公共 API
│   ├── error.rs             # 统一错误类型定义
│   ├── event.rs             # 事件处理器
│   ├── logging.rs           # 日志系统配置
│   ├── performance.rs       # 性能监控模块
│   ├── memory.rs            # 内存管理模块
│   ├── camera/              # 摄像头管理模块
│   │   ├── mod.rs           # 模块导出
│   │   ├── manager.rs       # 摄像头管理器实现
│   │   └── tests.rs         # 单元测试
│   ├── window/              # 窗口管理模块
│   │   ├── mod.rs           # 模块导出
│   │   ├── manager.rs       # 窗口管理器实现
│   │   ├── tests.rs         # 单元测试
│   │   └── scaling_tests.rs # 缩放功能测试
│   ├── render/              # GPU 渲染模块
│   │   ├── mod.rs           # 模块导出
│   │   ├── engine.rs        # 渲染引擎实现
│   │   └── shader.wgsl      # WGSL 着色器代码
│   ├── shape/               # 形状遮罩模块
│   │   ├── mod.rs           # 模块导出
│   │   └── mask.rs          # 形状遮罩实现
│   └── config/              # 配置管理模块
│       ├── mod.rs           # 模块导出
│       └── manager.rs       # 配置管理器实现
├── tests/                   # 集成测试
│   ├── integration_tests.rs # 完整工作流测试
│   ├── property_tests.rs    # 基于属性的测试
│   └── performance_integration_test.rs # 性能集成测试
├── benches/                 # 性能基准测试
│   ├── performance.rs       # 主要性能基准
│   └── memory_performance.rs # 内存性能基准
├── examples/                # 示例代码
│   ├── camera_demo.rs       # 摄像头功能演示
│   ├── window_demo.rs       # 窗口功能演示
│   ├── shape_demo.rs        # 形状遮罩演示
│   ├── scaling_demo.rs      # 缩放功能演示
│   ├── drag_demo.rs         # 拖拽功能演示
│   ├── rotation_demo.rs     # 旋转功能演示
│   └── error_handling_demo.rs # 错误处理演示
├── scripts/                 # 构建和部署脚本
│   ├── build_release.ps1    # Windows 构建脚本
│   ├── build_release.sh     # macOS/Linux 构建脚本
│   └── README.md            # 脚本使用说明
├── docs/                    # 项目文档
│   ├── camera_manager.md    # 摄像头管理文档
│   ├── window_manager.md    # 窗口管理文档
│   ├── scaling_implementation.md # 缩放实现文档
│   └── shape_generation.md  # 形状生成文档
├── .kiro/                   # Kiro 规范文档
│   └── specs/mira/          # Mira 项目规范
│       ├── requirements.md  # 需求文档
│       ├── design.md        # 设计文档
│       └── tasks.md         # 任务列表
├── Cargo.toml               # 项目配置和依赖
├── build.rs                 # 构建脚本
├── README.md                # 项目说明（本文件）
├── SETUP_GUIDE.md           # 环境配置指南
├── VALIDATION_REPORT.md     # 验证报告
└── LICENSE                  # 开源许可证
```

### 核心模块说明

#### 🎥 Camera Module (`src/camera/`)
负责摄像头设备的管理和视频流捕获：
- 设备枚举和选择
- 视频流启动和停止
- 帧数据捕获和格式转换
- 错误处理和重试机制

#### 🎄 Window Module (`src/window/`)
管理应用窗口的创建和交互：
- 置顶窗口创建
- 拖拽、缩放、旋转功能
- 边界约束和自动对齐
- 窗口状态管理

#### 🎨 Render Module (`src/render/`)
高性能 GPU 渲染引擎：
- wgpu 渲染管线
- 纹理管理和上传
- WGSL 着色器编译
- 帧缓冲和表面管理

#### 🔷 Shape Module (`src/shape/`)
形状遮罩生成和管理：
- 5 种预设形状算法
- 实时遮罩生成
- 尺寸自适应调整
- 性能优化

#### ⚙️ Config Module (`src/config/`)
配置文件管理和持久化：
- TOML 格式配置
- 跨平台路径处理
- 配置验证和迁移
- 默认值管理

### 技术栈

#### 核心依赖

| 库名 | 版本 | 用途 | 说明 |
|------|------|------|------|
| [nokhwa](https://crates.io/crates/nokhwa) | 0.10 | 摄像头捕获 | 跨平台摄像头库，支持多种后端 |
| [winit](https://crates.io/crates/winit) | 0.29 | 窗口管理 | 跨平台窗口创建和事件处理 |
| [wgpu](https://crates.io/crates/wgpu) | 0.20 | GPU 渲染 | 现代 GPU API，支持多种图形后端 |
| [image](https://crates.io/crates/image) | 0.24 | 图像处理 | 图像格式转换和基础处理 |
| [serde](https://crates.io/crates/serde) | 1.0 | 序列化 | 配置文件序列化和反序列化 |
| [toml](https://crates.io/crates/toml) | 0.8 | 配置格式 | TOML 格式配置文件解析 |
| [tokio](https://crates.io/crates/tokio) | 1.0 | 异步运行时 | 异步任务调度和 I/O 处理 |

#### 开发和测试依赖

| 库名 | 版本 | 用途 | 说明 |
|------|------|------|------|
| [quickcheck](https://crates.io/crates/quickcheck) | 1.0 | 属性测试 | 基于属性的随机化测试 |
| [criterion](https://crates.io/crates/criterion) | 0.5 | 性能基准 | 统计性能基准测试框架 |
| [mockall](https://crates.io/crates/mockall) | 0.12 | 模拟测试 | 创建模拟对象进行单元测试 |
| [tempfile](https://crates.io/crates/tempfile) | 3.0 | 临时文件 | 测试中的临时文件管理 |

### 构建和测试

#### 开发构建
```bash
# 标准构建
cargo build

# 发布构建
cargo build --release

# 运行应用
cargo run

# 监视模式（需要 cargo-watch）
cargo watch -x run
```

#### 测试套件
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test camera
cargo test window
cargo test integration

# 运行属性测试
cargo test property

# 详细测试输出
cargo test -- --nocapture
```

#### 性能基准
```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench performance
cargo bench memory

# 生成基准报告
cargo bench -- --output-format html
```

#### 代码质量
```bash
# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 文档生成
cargo doc --open

# 依赖检查
cargo audit
```

### 性能指标

Mira 在设计时就考虑了性能优化，以下是主要性能指标：

| 指标 | 目标值 | 实际表现 |
|------|--------|----------|
| **帧率** | ≥ 30 FPS | 30-60 FPS |
| **启动时间** | < 3 秒 | 1-2 秒 |
| **内存使用** | < 200 MB | 100-150 MB |
| **CPU 使用** | < 25% | 10-20% |
| **响应时间** | < 16 ms | 8-12 ms |
| **形状切换** | < 100 ms | 50-80 ms |

### 贡献指南

我们欢迎社区贡献！请参考以下指南：

1. **Fork 项目** 并创建功能分支
2. **遵循代码规范** 使用 `cargo fmt` 和 `cargo clippy`
3. **编写测试** 确保新功能有对应的测试
4. **更新文档** 如果 API 有变化
5. **提交 PR** 并描述你的更改

详细信息请参考 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

本项目采用 MIT 许可证 - 详情请参考 [LICENSE](LICENSE) 文件。

## 致谢

感谢以下开源项目和贡献者：

- [Rust 语言](https://www.rust-lang.org/) - 系统编程语言
- [wgpu 团队](https://wgpu.rs/) - 现代 GPU API
- [winit 维护者](https://github.com/rust-windowing/winit) - 窗口管理库
- [nokhwa 作者](https://github.com/l1npengtul/nokhwa) - 摄像头捕获库
- 所有测试用户和反馈者

## 联系方式

- 📧 邮箱: [15732651140@163.com]
- 🐙 GitHub: [https://github.com/Vogadero/Mira]
- 💬 讨论: [GitHub Discussions](https://github.com/Vogadero/Mira/discussions)

---

<div align="center">

**如果 Mira 对你有帮助，请给我们一个 ⭐ Star！**

Made with ❤️ by the Mira Team

</div>
