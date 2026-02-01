# Mira 部署指南

本指南将详细说明如何在不同环境中构建、打包和部署 Mira 桌面摄像精灵。

## 目录

- [环境准备](#环境准备)
- [本地构建](#本地构建)
- [Docker 构建](#docker-构建)
- [代码签名](#代码签名)
- [分发打包](#分发打包)
- [自动化部署](#自动化部署)

## 环境准备

### 推荐方案：本地环境

本地环境是最推荐的方案，因为：
- 性能最佳，编译速度快
- 调试方便，支持 GUI 应用
- 一次配置，长期使用
- 支持代码签名和公证

### Windows 环境配置

#### 1. 安装 Rust 工具链

```powershell
# 下载并运行 Rust 安装器
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe

# 验证安装
rustc --version
cargo --version
```

#### 2. 安装 Visual Studio Build Tools

**关键步骤**：这是必须的，否则无法编译。

```powershell
# 方法一：安装 Visual Studio Community 2022（推荐）
# 下载：https://visualstudio.microsoft.com/zh-hans/vs/community/
# 选择工作负载：使用 C++ 的桌面开发

# 方法二：仅安装 Build Tools
# 下载：https://visualstudio.microsoft.com/zh-hans/downloads/#build-tools-for-visual-studio-2022
# 选择：C++ 生成工具 + Windows 10/11 SDK
```

#### 3. 验证环境

```powershell
# 检查编译器
where cl.exe

# 如果找不到，在 Visual Studio Developer Command Prompt 中运行：
# "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat"
```

### macOS 环境配置

#### 1. 安装 Xcode Command Line Tools

```bash
# 安装命令行工具
xcode-select --install

# 验证安装
xcode-select -p
```

#### 2. 安装 Rust 工具链

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### 3. 安装辅助工具

```bash
# 安装 Homebrew（如果没有）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装必要工具
brew install git bc
```

## 本地构建

### 快速构建

```bash
# 克隆项目
git clone https://github.com/Vogadero/Mira.git mira
cd mira

# Windows 构建
.\scripts\build_release.ps1

# macOS/Linux 构建
chmod +x scripts/build_release.sh
./scripts/build_release.sh
```

### 详细构建步骤

#### 1. 开发构建

```bash
# 标准构建
cargo build

# 运行应用
cargo run

# 运行测试
cargo test

# 性能基准测试
cargo bench
```

#### 2. 发布构建

```bash
# 发布构建
cargo build --release

# 运行构建脚本（推荐）
# Windows
.\scripts\build_release.ps1 -Verbose

# macOS
./scripts/build_release.sh --verbose
```

#### 3. 构建输出

构建完成后，你会得到：

```
target/release/
├── mira.exe (Windows) 或 mira (macOS)
└── ...

dist/
├── mira.exe 或 mira
├── README.md
├── LICENSE
└── VERSION.txt

mira-windows-x64.zip 或 mira-macos-x64.tar.gz
```

## Docker 构建

如果你更喜欢使用 Docker 环境：

### Windows Docker 构建

```powershell
# 构建 Windows 镜像
docker build -f Dockerfile.windows -t mira-windows .

# 运行构建
docker run --rm -v ${PWD}\dist-windows:C:\build\dist mira-windows

# 使用 Docker Compose
docker-compose --profile windows up mira-windows
```

### Unix Docker 构建

```bash
# 构建 Unix 镜像
docker build -f Dockerfile.unix -t mira-unix .

# 运行构建
docker run --rm -v $(pwd)/dist-unix:/app/dist mira-unix

# 使用 Docker Compose
docker-compose --profile unix up mira-unix
```

### Docker 开发环境

```bash
# 启动开发环境
docker-compose --profile dev up mira-dev

# 进入容器
docker-compose --profile dev exec mira-dev bash

# 在容器内开发
cargo run
cargo test
```

## 代码签名

为了让用户信任你的应用，建议对可执行文件进行代码签名。

### Windows 代码签名

#### 1. 生成开发证书

```powershell
# 生成自签名证书
.\scripts\generate_certificates.ps1 -Install

# 查看生成的文件
ls certificates\
```

#### 2. 签名可执行文件

```powershell
# 使用生成的脚本签名
.\certificates\sign-mira.ps1 -FilePath target\release\mira.exe

# 或手动签名
$password = Get-Content certificates\certificate-password.txt
signtool sign /f certificates\mira-cert.pfx /p $password /t http://timestamp.digicert.com target\release\mira.exe
```

#### 3. 验证签名

```powershell
# 验证签名
signtool verify /pa target\release\mira.exe

# 查看签名信息
Get-AuthenticodeSignature target\release\mira.exe
```

### macOS 代码签名

#### 1. 生成开发证书

```bash
# 生成自签名证书
chmod +x scripts/generate_certificates.sh
./scripts/generate_certificates.sh --install

# 查看生成的文件
ls certificates/
```

#### 2. 签名可执行文件

```bash
# 使用生成的脚本签名
./certificates/sign-mira.sh target/release/mira

# 或手动签名
codesign --sign "Mira Development Certificate" --verbose target/release/mira
```

#### 3. 验证签名

```bash
# 验证签名
codesign --verify --verbose target/release/mira

# 查看签名信息
codesign --display --verbose target/release/mira
```

### 生产环境证书

**重要**：自签名证书仅用于开发和测试。生产环境需要：

- **Windows**: 从受信任的 CA（如 DigiCert、Sectigo）购买代码签名证书
- **macOS**: 需要 Apple Developer 账户和 Developer ID 证书

## 分发打包

### 创建安装包

#### Windows 安装包

可以使用以下工具创建 Windows 安装包：

1. **NSIS** (推荐)
2. **Inno Setup**
3. **WiX Toolset**
4. **Advanced Installer**

示例 NSIS 脚本：

```nsis
; Mira 安装脚本
!define APP_NAME "Mira"
!define APP_VERSION "1.0.0"
!define APP_PUBLISHER "Mira Team"
!define APP_EXE "mira.exe"

Name "${APP_NAME}"
OutFile "MiraInstaller.exe"
InstallDir "$PROGRAMFILES\${APP_NAME}"

Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    File "dist\mira.exe"
    File "dist\README.md"
    File "dist\LICENSE"
    
    CreateDirectory "$SMPROGRAMS\${APP_NAME}"
    CreateShortCut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
    CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
    
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" "$INSTDIR\uninstall.exe"
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\mira.exe"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\uninstall.exe"
    
    Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
    Delete "$DESKTOP\${APP_NAME}.lnk"
    RMDir "$SMPROGRAMS\${APP_NAME}"
    RMDir "$INSTDIR"
    
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
SectionEnd
```

#### macOS 应用包

```bash
# 创建应用包结构
mkdir -p Mira.app/Contents/MacOS
mkdir -p Mira.app/Contents/Resources

# 复制可执行文件
cp target/release/mira Mira.app/Contents/MacOS/

# 创建 Info.plist
cat > Mira.app/Contents/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>mira</string>
    <key>CFBundleIdentifier</key>
    <string>com.mira.desktop</string>
    <key>CFBundleName</key>
    <string>Mira</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>NSCameraUsageDescription</key>
    <string>Mira needs camera access to display video feed</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
</dict>
</plist>
EOF

# 签名应用包
codesign --sign "Developer ID Application: Your Name" --verbose Mira.app

# 创建 DMG
hdiutil create -volname "Mira" -srcfolder Mira.app -ov -format UDZO Mira.dmg
```

### 自动化构建脚本

创建一个完整的自动化构建脚本：

#### Windows 自动化脚本

```powershell
# build-and-package.ps1
param(
    [switch]$Sign = $false,
    [switch]$Package = $false
)

Write-Host "Mira 自动化构建和打包" -ForegroundColor Green

# 1. 清理和构建
Write-Host "1. 构建应用..." -ForegroundColor Cyan
.\scripts\build_release.ps1 -Clean -Verbose

# 2. 代码签名（如果请求）
if ($Sign) {
    Write-Host "2. 代码签名..." -ForegroundColor Cyan
    if (Test-Path "certificates\sign-mira.ps1") {
        .\certificates\sign-mira.ps1 -FilePath target\release\mira.exe
    } else {
        Write-Warning "未找到签名脚本，跳过代码签名"
    }
}

# 3. 创建安装包（如果请求）
if ($Package) {
    Write-Host "3. 创建安装包..." -ForegroundColor Cyan
    if (Get-Command makensis -ErrorAction SilentlyContinue) {
        makensis installer.nsi
        Write-Host "✓ 安装包创建完成: MiraInstaller.exe" -ForegroundColor Green
    } else {
        Write-Warning "未找到 NSIS，跳过安装包创建"
    }
}

Write-Host "构建完成！" -ForegroundColor Green
```

#### macOS 自动化脚本

```bash
#!/bin/bash
# build-and-package.sh

set -e

SIGN=false
PACKAGE=false

# 参数解析
while [[ $# -gt 0 ]]; do
    case $1 in
        --sign)
            SIGN=true
            shift
            ;;
        --package)
            PACKAGE=true
            shift
            ;;
        *)
            echo "未知参数: $1"
            exit 1
            ;;
    esac
done

echo "Mira 自动化构建和打包"

# 1. 清理和构建
echo "1. 构建应用..."
./scripts/build_release.sh --clean --verbose

# 2. 代码签名（如果请求）
if [ "$SIGN" = true ]; then
    echo "2. 代码签名..."
    if [ -f "certificates/sign-mira.sh" ]; then
        ./certificates/sign-mira.sh target/release/mira
    else
        echo "警告: 未找到签名脚本，跳过代码签名"
    fi
fi

# 3. 创建应用包（如果请求）
if [ "$PACKAGE" = true ]; then
    echo "3. 创建应用包..."
    
    # 创建应用包结构
    rm -rf Mira.app
    mkdir -p Mira.app/Contents/MacOS
    mkdir -p Mira.app/Contents/Resources
    
    # 复制文件
    cp target/release/mira Mira.app/Contents/MacOS/
    cp dist/README.md Mira.app/Contents/Resources/
    cp dist/LICENSE Mira.app/Contents/Resources/
    
    # 创建 Info.plist
    cat > Mira.app/Contents/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>mira</string>
    <key>CFBundleIdentifier</key>
    <string>com.mira.desktop</string>
    <key>CFBundleName</key>
    <string>Mira</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>NSCameraUsageDescription</key>
    <string>Mira needs camera access to display video feed</string>
</dict>
</plist>
EOF
    
    # 签名应用包
    if [ "$SIGN" = true ]; then
        codesign --sign "Mira Development Certificate" --verbose Mira.app
    fi
    
    # 创建 DMG
    if command -v hdiutil &> /dev/null; then
        hdiutil create -volname "Mira" -srcfolder Mira.app -ov -format UDZO Mira.dmg
        echo "✓ DMG 创建完成: Mira.dmg"
    fi
fi

echo "构建完成！"
```

## 自动化部署

### GitHub Actions

创建 `.github/workflows/build.yml`：

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'
  pull_request:
    branches: [ main ]

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        profile: minimal
        override: true
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Build
      run: .\scripts\build_release.ps1 -Verbose
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: mira-windows-x64
        path: mira-windows-x64.zip

  build-macos:
    runs-on: macos-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        profile: minimal
        override: true
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Build
      run: |
        chmod +x scripts/build_release.sh
        ./scripts/build_release.sh --verbose
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: mira-macos-x64
        path: mira-macos-x64.tar.gz

  release:
    needs: [build-windows, build-macos]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')
    steps:
    - name: Download Windows artifact
      uses: actions/download-artifact@v3
      with:
        name: mira-windows-x64
    
    - name: Download macOS artifact
      uses: actions/download-artifact@v3
      with:
        name: mira-macos-x64
    
    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        files: |
          mira-windows-x64.zip
          mira-macos-x64.tar.gz
        draft: false
        prerelease: false
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## 故障排除

### 常见构建问题

#### Windows 问题

1. **`error: linker 'link.exe' not found`**
   ```powershell
   # 解决方案：安装 Visual Studio Build Tools
   # 或在 Visual Studio Developer Command Prompt 中运行
   ```

2. **PowerShell 执行策略错误**
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

#### macOS 问题

1. **`xcrun: error: invalid active developer path`**
   ```bash
   sudo xcode-select --reset
   xcode-select --install
   ```

2. **权限问题**
   ```bash
   chmod +x scripts/*.sh
   ```

### 性能优化

1. **并行编译**
   ```bash
   export CARGO_BUILD_JOBS=8
   ```

2. **使用更快的链接器**
   ```toml
   # .cargo/config.toml
   [target.x86_64-pc-windows-msvc]
   linker = "lld-link.exe"
   
   [target.x86_64-apple-darwin]
   rustflags = ["-C", "link-arg=-fuse-ld=lld"]
   ```

## 总结

本指南提供了完整的 Mira 部署流程：

1. **环境配置**: 本地环境是最推荐的方案
2. **构建脚本**: 使用提供的自动化脚本
3. **代码签名**: 提高用户信任度
4. **打包分发**: 创建专业的安装包
5. **自动化**: 使用 CI/CD 自动化构建和发布

选择适合你的方案，按照步骤操作即可成功部署 Mira 应用！