# Mira 构建脚本

本目录包含用于构建和分发 Mira 应用程序的脚本。

## 脚本说明

### Windows 构建脚本 (`build_release.ps1`)

用于在 Windows 平台上构建优化的发布版本。

**使用方法:**
```powershell
# 基本构建
.\scripts\build_release.ps1

# 清理构建
.\scripts\build_release.ps1 -Clean

# 详细输出
.\scripts\build_release.ps1 -Verbose

# 清理并详细构建
.\scripts\build_release.ps1 -Clean -Verbose
```

**功能:**
- 构建优化的发布版本
- 验证二进制文件大小（目标 < 20MB）
- 检查依赖项
- 创建分发包
- 生成 ZIP 压缩包
- 运行基本验证测试

### macOS/Linux 构建脚本 (`build_release.sh`)

用于在 macOS 和 Linux 平台上构建优化的发布版本。

**使用方法:**
```bash
# 基本构建
./scripts/build_release.sh

# 清理构建
./scripts/build_release.sh --clean

# 详细输出
./scripts/build_release.sh --verbose

# 清理并详细构建
./scripts/build_release.sh --clean --verbose
```

**功能:**
- 构建优化的发布版本
- 验证二进制文件大小（目标 < 25MB）
- 检查依赖项
- 创建分发包
- 生成 tar.gz 压缩包
- 运行基本验证测试

## 构建优化配置

项目的 `Cargo.toml` 文件包含以下发布优化配置：

```toml
[profile.release]
opt-level = 3          # 最高优化级别
lto = true            # 链接时优化
codegen-units = 1     # 单个代码生成单元
debug = false         # 禁用调试信息
panic = "abort"       # 使用 abort 而不是 unwind
strip = true          # 去除符号表
```

## 大小要求

根据需求文档，应用程序应满足以下大小要求：

- **Windows**: 安装包 < 20MB
- **macOS**: 安装包 < 25MB
- **安装后**: 磁盘占用 < 50MB

## 输出文件

构建脚本会生成以下文件：

1. **二进制文件**: `target/release/mira[.exe]`
2. **分发目录**: `dist/` - 包含二进制文件和文档
3. **压缩包**: 
   - Windows: `mira-windows-x64.zip`
   - macOS: `mira-macos-x64.tar.gz`
   - Linux: `mira-linux-x64.tar.gz`

## 验证检查

构建脚本会自动验证：

- ✅ 二进制文件大小是否符合要求
- ✅ 分发包总大小是否合理
- ✅ 压缩包大小是否适合下载
- ✅ 应用程序是否可以正常启动
- ✅ 依赖项检查（如果工具可用）

## 故障排除

### Windows 构建问题

1. **缺少 Visual Studio Build Tools**:
   - 安装 Visual Studio 2019 或更高版本
   - 或安装 Build Tools for Visual Studio
   - 确保包含 C++ 工具链

2. **PowerShell 执行策略**:
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

### macOS/Linux 构建问题

1. **权限问题**:
   ```bash
   chmod +x scripts/build_release.sh
   ```

2. **缺少依赖**:
   - macOS: 安装 Xcode Command Line Tools
   - Linux: 安装 build-essential 包

3. **缺少 bc 计算器**:
   ```bash
   # macOS
   brew install bc
   
   # Ubuntu/Debian
   sudo apt-get install bc
   
   # CentOS/RHEL
   sudo yum install bc
   ```

## 持续集成

这些脚本可以在 CI/CD 管道中使用：

```yaml
# GitHub Actions 示例
- name: Build Release (Windows)
  run: .\scripts\build_release.ps1 -Verbose
  shell: powershell

- name: Build Release (macOS)
  run: ./scripts/build_release.sh --verbose
  shell: bash
```

## 性能基准测试

除了发布构建，还可以运行性能基准测试：

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench performance
cargo bench --bench memory_performance
```

基准测试结果会保存在 `target/criterion/` 目录中。