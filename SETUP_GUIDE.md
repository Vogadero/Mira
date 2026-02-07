# Mira 开发环境配置指南

## 概述

本指南将帮助你在 Windows 和 macOS 上配置 Mira 桌面摄像精灵的开发和构建环境。

## Windows 环境配置

### 1. 安装 Rust 工具链

1. **下载 Rust 安装器**
   - 访问 [https://rustup.rs/](https://rustup.rs/)
   - 下载 `rustup-init.exe`

2. **运行安装器**
   ```cmd
   # 下载后运行
   rustup-init.exe
   
   # 选择默认安装选项（按 1 然后回车）
   ```

3. **验证安装**
   ```cmd
   # 重新打开命令提示符
   rustc --version
   cargo --version
   ```

### 2. 安装 Visual Studio Build Tools

这是 **最关键** 的步骤，缺少这个会导致编译失败。

**方法一：安装 Visual Studio Community（推荐）**

1. 下载 [Visual Studio Community 2022](https://visualstudio.microsoft.com/zh-hans/vs/community/)
2. 运行安装器，选择以下工作负载：
   - ✅ **使用 C++ 的桌面开发**
   - ✅ **通用 Windows 平台开发**（可选）

**方法二：仅安装 Build Tools**

1. 下载 [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/zh-hans/downloads/#build-tools-for-visual-studio-2022)
2. 运行安装器，选择：
   - ✅ **C++ 生成工具**
   - ✅ **Windows 10/11 SDK**
   - ✅ **MSVC v143 编译器工具集**

### 3. 安装 Git（如果没有）

```cmd
# 下载并安装 Git for Windows
# https://git-scm.com/download/win
```

### 4. 验证环境

```cmd
# 检查 Rust
rustc --version
cargo --version

# 检查 C++ 编译器
where cl.exe

# 如果找不到 cl.exe，需要在 Visual Studio Developer Command Prompt 中运行
```

### 5. 克隆和构建项目

```cmd
# 克隆项目（如果还没有）
git clone https://github.com/Vogadero/Mira.git mira
cd mira

# 构建项目
cargo build --release

# 运行构建脚本
.\scripts\build_release.ps1
```

## macOS 环境配置

### 1. 安装 Xcode Command Line Tools

```bash
# 安装命令行工具
xcode-select --install

# 验证安装
xcode-select -p
```

### 2. 安装 Rust 工具链

```bash
# 下载并安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 3. 安装 Homebrew（推荐）

```bash
# 安装 Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装必要工具
brew install git
brew install bc  # 用于构建脚本中的数学计算
```

### 4. 验证环境

```bash
# 检查所有工具
rustc --version
cargo --version
git --version
bc --version
```

### 5. 克隆和构建项目

```bash
# 克隆项目
git clone https://github.com/Vogadero/Mira.git mira
cd mira

# 给构建脚本执行权限
chmod +x scripts/build_release.sh

# 构建项目
cargo build --release

# 运行构建脚本
./scripts/build_release.sh
```

## Linux 环境配置

### 1. 安装系统依赖

**Ubuntu/Debian:**

```bash
# 更新包列表
sudo apt-get update

# 安装必要的系统库
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libv4l-dev \
    libgtk-3-dev \
    libglib2.0-dev \
    libappindicator3-dev \
    git \
    curl \
    bc
```

**Fedora/RHEL/CentOS:**

```bash
# 安装必要的系统库
sudo dnf install -y \
    gcc \
    gcc-c++ \
    make \
    pkg-config \
    openssl-devel \
    systemd-devel \
    v4l-utils-devel \
    gtk3-devel \
    glib2-devel \
    libappindicator-gtk3-devel \
    git \
    curl \
    bc
```

**Arch Linux:**

```bash
# 安装必要的系统库
sudo pacman -S --needed \
    base-devel \
    pkg-config \
    openssl \
    systemd \
    v4l-utils \
    gtk3 \
    glib2 \
    libappindicator-gtk3 \
    git \
    curl \
    bc
```

### 2. 安装 Rust 工具链

```bash
# 下载并安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 3. 验证环境

```bash
# 检查所有工具
rustc --version
cargo --version
git --version
pkg-config --version

# 验证 GTK 库
pkg-config --modversion gtk+-3.0
pkg-config --modversion glib-2.0
```

### 4. 克隆和构建项目

```bash
# 克隆项目
git clone https://github.com/Vogadero/Mira.git mira
cd mira

# 给构建脚本执行权限
chmod +x scripts/build_release.sh

# 构建项目
cargo build --release

# 运行构建脚本
./scripts/build_release.sh
```

### 5. Linux 特别说明

**系统托盘图标依赖:**
- Mira 使用系统托盘图标功能，在 Linux 上需要 GTK3 和 AppIndicator 支持
- 如果你的桌面环境不支持系统托盘，托盘图标可能不会显示，但应用仍可正常使用

**摄像头权限:**
```bash
# 确保你的用户在 video 组中
sudo usermod -a -G video $USER

# 重新登录以使更改生效
```

**Wayland 支持:**
- Mira 在 Wayland 和 X11 上都能运行
- 某些功能在 Wayland 上可能有限制（如窗口拖拽）

## Docker 环境配置（可选）

如果你更喜欢使用 Docker，这里提供配置：

### Windows Docker 配置

```dockerfile
# Dockerfile.windows
FROM mcr.microsoft.com/windows/servercore:ltsc2022

# 安装 Rust 和构建工具
RUN powershell -Command \
    Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe; \
    .\rustup-init.exe -y; \
    Remove-Item rustup-init.exe

# 设置环境变量
ENV PATH="C:\Users\ContainerUser\.cargo\bin:${PATH}"

WORKDIR /app
COPY . .

RUN cargo build --release
```

### macOS/Linux Docker 配置

```dockerfile
# Dockerfile.unix
FROM rust:1.75

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    bc \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release
```

## 常见问题解决

### Windows 问题

**问题 1**: `error: linker 'link.exe' not found`
```cmd
# 解决方案：安装 Visual Studio Build Tools
# 或在 Visual Studio Developer Command Prompt 中运行
```

**问题 2**: `error: failed to run custom build command for 'windows-sys'`
```cmd
# 解决方案：确保安装了 Windows SDK
# 在 Visual Studio Installer 中添加 Windows 10/11 SDK
```

**问题 3**: PowerShell 执行策略错误
```powershell
# 解决方案：设置执行策略
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### macOS 问题

**问题 1**: `xcrun: error: invalid active developer path`
```bash
# 解决方案：重新安装命令行工具
sudo xcode-select --reset
xcode-select --install
```

**问题 2**: `ld: library not found`
```bash
# 解决方案：更新 Xcode 命令行工具
sudo xcode-select --install
```

## 性能优化建议

### 加速编译

```bash
# 设置并行编译
export CARGO_BUILD_JOBS=8

# 使用更快的链接器（可选）
# Windows: 无需额外配置
# macOS: 考虑使用 lld
```

### 减少构建时间

```toml
# 在 Cargo.toml 中添加
[profile.dev]
opt-level = 1  # 轻微优化以加快开发
incremental = true

[profile.dev.package."*"]
opt-level = 2  # 对依赖项使用更高优化
```

## 验证安装成功

运行以下命令验证环境配置正确：

### Windows
```cmd
# 进入项目目录
cd mira

# 运行完整构建和验证
.\scripts\build_release.ps1 -Verbose

# 检查输出文件
dir target\release\mira.exe
dir mira-windows-x64.zip
```

### macOS
```bash
# 进入项目目录
cd mira

# 运行完整构建和验证
./scripts/build_release.sh --verbose

# 检查输出文件
ls -la target/release/mira
ls -la mira-macos-x64.tar.gz
```

## 下一步

环境配置完成后，你可以：

1. **开发**: 使用 `cargo run` 运行应用
2. **测试**: 使用 `cargo test` 运行测试
3. **构建**: 使用构建脚本创建发布版本
4. **调试**: 使用 `cargo run` 或 IDE 调试功能

## 获取帮助

如果遇到问题：

1. 检查 [Rust 官方文档](https://doc.rust-lang.org/)
2. 查看项目的 `TROUBLESHOOTING.md`
3. 在项目 Issues 中搜索类似问题
4. 创建新的 Issue 描述你的问题