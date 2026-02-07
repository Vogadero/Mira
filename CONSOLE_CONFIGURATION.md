# Mira 控制台配置指南

本文档说明如何控制 Mira 应用的控制台窗口显示/隐藏。

## 概述

Mira 在不同模式下有不同的控制台行为：

| 模式 | 默认行为 | 说明 |
|------|----------|------|
| **Release（发布）** | 隐藏控制台 | 用户不会看到黑色的 CMD 窗口 |
| **Debug（调试）** | 显示控制台 | 开发者可以看到日志输出 |

## 为什么要隐藏控制台？

### 用户体验
- ✅ 更专业的外观
- ✅ 不会有黑色 CMD 窗口干扰
- ✅ 更像原生桌面应用
- ✅ 减少用户困惑

### 开发调试
- ✅ Debug 模式下仍可看到日志
- ✅ 可以通过环境变量强制显示
- ✅ 日志文件始终可用

## 控制台行为

### Release 模式（发布版本）

**默认行为**：控制台窗口隐藏

```bash
# 构建发布版本（控制台隐藏）
cargo build --release

# 运行发布版本
.\target\release\mira.exe  # 不会显示控制台窗口
```

**特点**：
- 应用启动时不显示黑色 CMD 窗口
- 日志仍然写入日志文件
- 托盘图标的"显示信息"功能不会在控制台显示（仅写入日志）

### Debug 模式（开发版本）

**默认行为**：控制台窗口显示

```bash
# 构建调试版本（控制台显示）
cargo build

# 运行调试版本
cargo run  # 会显示控制台窗口和日志输出
```

**特点**：
- 应用启动时显示控制台窗口
- 实时查看日志输出
- 方便调试和开发
- 托盘图标的"显示信息"会在控制台显示

## 强制显示/隐藏控制台

### 方法一：环境变量（推荐）

#### Windows PowerShell

```powershell
# 强制显示控制台（即使在 release 模式）
$env:MIRA_SHOW_CONSOLE="true"
cargo build --release
.\target\release\mira.exe

# 或者直接运行
$env:MIRA_SHOW_CONSOLE="true"; .\target\release\mira.exe

# 恢复默认（隐藏控制台）
$env:MIRA_SHOW_CONSOLE="false"
```

#### Windows CMD

```cmd
REM 强制显示控制台
set MIRA_SHOW_CONSOLE=true
cargo build --release
.\target\release\mira.exe

REM 恢复默认
set MIRA_SHOW_CONSOLE=false
```

#### Linux/macOS

```bash
# 强制显示控制台
export MIRA_SHOW_CONSOLE=true
cargo build --release
./target/release/mira

# 恢复默认
export MIRA_SHOW_CONSOLE=false
```

### 方法二：修改构建脚本

如果你想永久改变行为，可以修改 `build.rs`：

```rust
// 在 build.rs 中找到这一行：
if is_release && show_console != "true" && show_console != "1" {

// 改为（始终显示控制台）：
if false {

// 或改为（始终隐藏控制台）：
if true {
```

## 日志查看

即使控制台隐藏，你仍然可以查看日志：

### 日志文件位置

| 平台 | 路径 |
|------|------|
| **Windows** | `%APPDATA%\Mira\logs\mira.log` |
| **macOS** | `~/Library/Application Support/Mira/logs/mira.log` |
| **Linux** | `~/.local/share/Mira/logs/mira.log` |

### 实时查看日志

#### Windows PowerShell

```powershell
# 实时查看日志
Get-Content "$env:APPDATA\Mira\logs\mira.log" -Wait -Tail 20
```

#### Linux/macOS

```bash
# 实时查看日志
tail -f ~/Library/Application\ Support/Mira/logs/mira.log  # macOS
tail -f ~/.local/share/Mira/logs/mira.log                  # Linux
```

## 构建配置

### 发布版本（用户使用）

```bash
# Windows
.\scripts\build_release.ps1

# macOS/Linux
./scripts/build_release.sh
```

**结果**：
- ✅ 控制台隐藏
- ✅ 优化的二进制文件
- ✅ 适合分发给用户

### 调试版本（开发使用）

```bash
# 直接运行（显示控制台）
cargo run

# 或构建后运行
cargo build
.\target\debug\mira.exe  # Windows
./target/debug/mira      # macOS/Linux
```

**结果**：
- ✅ 控制台显示
- ✅ 实时日志输出
- ✅ 方便调试

### 带控制台的发布版本（调试用）

```bash
# Windows PowerShell
$env:MIRA_SHOW_CONSOLE="true"
cargo build --release

# Linux/macOS
export MIRA_SHOW_CONSOLE=true
cargo build --release
```

**结果**：
- ✅ 控制台显示
- ✅ 优化的二进制文件
- ✅ 适合调试发布版本的问题

## CI/CD 配置

在 GitHub Actions 中，发布版本会自动隐藏控制台：

```yaml
# .github/workflows/build.yml
- name: Build release
  run: |
    # 默认隐藏控制台
    cargo build --release --verbose --bin mira
```

如果需要在 CI 中显示控制台：

```yaml
- name: Build release with console
  env:
    MIRA_SHOW_CONSOLE: true
  run: |
    cargo build --release --verbose --bin mira
```

## 常见问题

### Q: 为什么我的发布版本还显示控制台？

**A**: 可能的原因：
1. 设置了 `MIRA_SHOW_CONSOLE=true` 环境变量
2. 使用了 `cargo run --release`（这会显示控制台）
3. 构建脚本被修改了

**解决方案**：
```bash
# 确保环境变量未设置
$env:MIRA_SHOW_CONSOLE=""  # PowerShell
set MIRA_SHOW_CONSOLE=     # CMD

# 重新构建
cargo clean
cargo build --release
```

### Q: 如何在发布版本中查看日志？

**A**: 有三种方法：
1. 查看日志文件（推荐）
2. 使用 `MIRA_SHOW_CONSOLE=true` 重新构建
3. 使用调试版本

### Q: 控制台隐藏后，"显示信息"功能还有用吗？

**A**: 有用！信息会写入日志文件。你可以：
1. 打开日志文件查看
2. 使用实时日志查看命令
3. 使用带控制台的版本

### Q: macOS/Linux 也会隐藏控制台吗？

**A**: 不会。控制台隐藏功能仅在 Windows 上有效，因为：
- Windows 的 GUI 应用会显示黑色 CMD 窗口
- macOS/Linux 的终端应用不会有这个问题
- 在 macOS/Linux 上，用户通常从终端启动应用

### Q: 如何为测试用户提供带控制台的版本？

**A**: 构建时设置环境变量：

```bash
# Windows
$env:MIRA_SHOW_CONSOLE="true"
.\scripts\build_release.ps1

# 生成的 mira.exe 会显示控制台
```

## 技术细节

### Windows 子系统

Windows 应用有两种子系统：

1. **CONSOLE** - 控制台应用
   - 启动时显示 CMD 窗口
   - 适合命令行工具
   - 可以使用 `println!` 输出

2. **WINDOWS** - GUI 应用
   - 启动时不显示 CMD 窗口
   - 适合桌面应用
   - `println!` 输出会被忽略

Mira 在 release 模式下使用 WINDOWS 子系统。

### 链接器参数

```rust
// build.rs 中设置
println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
```

这告诉链接器：
- 使用 WINDOWS 子系统（隐藏控制台）
- 使用标准的 main 函数入口点

### 日志系统

即使控制台隐藏，日志系统仍然工作：

```rust
// src/logging.rs
// 日志同时写入文件和控制台（如果可见）
fern::Dispatch::new()
    .chain(std::io::stdout())  // 控制台输出
    .chain(log_file)           // 文件输出
```

## 最佳实践

### 开发阶段
- ✅ 使用 `cargo run` 查看实时日志
- ✅ 使用 debug 模式进行调试
- ✅ 频繁查看控制台输出

### 测试阶段
- ✅ 使用 release 模式测试用户体验
- ✅ 验证控制台已隐藏
- ✅ 测试日志文件是否正常写入

### 发布阶段
- ✅ 使用 release 模式构建
- ✅ 确保控制台隐藏
- ✅ 提供日志文件位置说明

### 调试发布版本
- ✅ 使用 `MIRA_SHOW_CONSOLE=true` 构建
- ✅ 或查看日志文件
- ✅ 使用日志级别控制输出详细程度

## 相关文档

- [SETUP_GUIDE.md](SETUP_GUIDE.md) - 环境配置
- [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) - 部署指南
- [README.md](README.md) - 项目主页

## 反馈

如有问题或建议，请：
- 创建 [Issue](https://github.com/Vogadero/Mira/issues)
- 参与 [Discussions](https://github.com/Vogadero/Mira/discussions)
- 发送邮件至 15732651140@163.com
