# Mira 发布说明

## 如何获取构建版本

### 自动构建（推荐）

我们的 GitHub Actions 会自动为每次提交构建所有平台的版本：

1. **访问 Actions 页面**: https://github.com/Vogadero/Mira/actions
2. **选择最新的构建**: 点击最新的 "Build and Release" 工作流
3. **下载构建产物**: 在页面底部的 "Artifacts" 部分下载对应平台的文件

### 发布版本

当我们创建 Git 标签时，会自动创建正式发布版本：

1. **访问发布页面**: https://github.com/Vogadero/Mira/releases
2. **下载最新版本**: 选择适合你平台的文件下载

## 支持的平台

| 平台 | 文件名 | 说明 |
|------|--------|------|
| Windows | `mira-windows-x64.zip` | Windows 10+ (64位) |
| macOS | `mira-macos-x64.tar.gz` | macOS 11+ (Intel/Apple Silicon) |
| Linux | `mira-linux-x64.tar.gz` | Ubuntu 20.04+ 或同等版本 |

## 安装说明

### Windows
1. 下载 `mira-windows-x64.zip`
2. 解压到任意目录
3. 双击运行 `mira.exe`
4. 首次运行时授予摄像头权限

### macOS
1. 下载 `mira-macos-x64.tar.gz`
2. 解压：`tar -xzf mira-macos-x64.tar.gz`
3. 运行：`./mira`
4. 如果遇到安全提示，在系统偏好设置中允许运行

### Linux
1. 下载 `mira-linux-x64.tar.gz`
2. 解压：`tar -xzf mira-linux-x64.tar.gz`
3. 添加执行权限：`chmod +x mira`
4. 运行：`./mira`

## 构建状态

当前构建状态：[![Build Status](https://github.com/Vogadero/Mira/workflows/Build%20and%20Release/badge.svg)](https://github.com/Vogadero/Mira/actions)

## 创建发布版本

如果你是项目维护者，可以通过以下步骤创建新的发布版本：

```bash
# 创建并推送标签
git tag v1.0.0
git push origin v1.0.0
```

这会自动触发 GitHub Actions 构建并创建发布版本。

## 本地构建

如果你想本地构建，请参考：
- [SETUP_GUIDE.md](SETUP_GUIDE.md) - 环境配置
- [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) - 详细构建指南

## 故障排除

### 构建失败
1. 检查 [Actions 页面](https://github.com/Vogadero/Mira/actions) 的错误日志
2. 确保所有依赖项都正确配置
3. 检查 Rust 版本是否符合要求

### 运行问题
1. 确保系统满足最低要求
2. 检查摄像头权限设置
3. 查看应用日志文件

## 反馈和支持

- 🐛 报告问题: [GitHub Issues](https://github.com/Vogadero/Mira/issues)
- 💬 讨论交流: [GitHub Discussions](https://github.com/Vogadero/Mira/discussions)
- 📧 联系邮箱: 15732651140@163.com