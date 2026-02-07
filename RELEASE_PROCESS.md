# Mira 发布流程

本文档说明如何发布 Mira 的新版本。

## 自动发布流程

Mira 使用 GitHub Actions 实现完全自动化的构建和发布流程。

### 工作流程

```
提交代码 → 自动构建（所有平台）→ 构建成功 → 创建标签 → 自动发布
```

**关键特性**：
- ✅ 构建成功后才会打标签
- ✅ 一个工作流完成所有步骤
- ✅ 失败时不会创建标签或发布

## 发布新版本

### 方法一：通过提交消息（推荐）

在提交消息中包含 `[release]` 标记：

```bash
# 修改代码
git add .
git commit -m "feat: add new feature [release]"
git push origin main
```

这将自动：
1. ✅ 构建所有平台的版本（Windows、macOS、Linux）
2. ✅ 等待所有构建成功
3. ✅ 创建新的版本标签（自动递增 patch 版本）
4. ✅ 创建 GitHub Release
5. ✅ 上传所有构建产物

**注意**：只有当所有平台构建成功后，才会创建标签和发布。

### 方法二：手动触发

1. 访问 [Actions 页面](https://github.com/Vogadero/Mira/actions)
2. 选择 "Build and Release" 工作流
3. 点击 "Run workflow"
4. 配置选项：
   - **version_bump**: 选择版本提升类型
     - **patch**: 修复版本 (1.0.0 → 1.0.1)
     - **minor**: 次要版本 (1.0.0 → 1.1.0)
     - **major**: 主要版本 (1.0.0 → 2.0.0)
   - **create_release**: 勾选以创建发布
5. 点击 "Run workflow" 确认

### 方法三：手动创建标签

```bash
# 创建标签
git tag -a v1.0.0 -m "Release v1.0.0"

# 推送标签
git push origin v1.0.0
```

这将触发构建和发布流程（跳过自动打标签步骤）。

## 版本号规范

Mira 遵循 [语义化版本](https://semver.org/lang/zh-CN/) 规范：

```
v主版本号.次版本号.修订号

例如: v1.2.3
```

### 版本提升规则

- **主版本号 (Major)**: 不兼容的 API 修改
  - 例如：重大架构变更、删除功能
  - 1.0.0 → 2.0.0

- **次版本号 (Minor)**: 向下兼容的功能性新增
  - 例如：新增形状、新增功能
  - 1.0.0 → 1.1.0

- **修订号 (Patch)**: 向下兼容的问题修正
  - 例如：Bug 修复、性能优化
  - 1.0.0 → 1.0.1

## 发布检查清单

在发布新版本前，请确保：

### 代码质量
- [ ] 所有测试通过 (`cargo test`)
- [ ] 代码格式正确 (`cargo fmt`)
- [ ] 没有 Clippy 警告 (`cargo clippy`)
- [ ] 文档已更新

### 功能验证
- [ ] 在 Windows 上测试
- [ ] 在 macOS 上测试
- [ ] 在 Linux 上测试（如果可能）
- [ ] 所有核心功能正常工作
- [ ] 没有已知的严重 Bug

### 文档更新
- [ ] README.md 已更新
- [ ] CHANGELOG.md 已更新（如果有）
- [ ] 版本号已更新
- [ ] 发布说明已准备

## 发布后

### 验证发布

1. 检查 [Releases 页面](https://github.com/Vogadero/Mira/releases)
2. 验证所有平台的构建产物都已上传
3. 下载并测试每个平台的版本
4. 验证校验和文件

### 通知用户

1. 在 GitHub Discussions 发布公告
2. 更新项目主页（如果有）
3. 在社交媒体分享（如果适用）

## 工作流详解

### Build and Release (`build.yml`)

这是唯一的工作流文件，包含所有构建和发布逻辑。

**触发条件**：
- 推送到 main/master 分支
- 创建标签（v*）
- Pull Request
- 手动触发

**工作流程**：

#### 1. 构建阶段（并行执行）
- **build-windows**: 构建 Windows x64 版本
- **build-macos**: 构建 macOS x64/ARM64 版本
- **build-linux**: 构建 Linux x64 版本

每个构建任务：
- 安装 Rust 工具链
- 缓存依赖项
- 编译 release 版本
- 创建分发包
- 上传构建产物

#### 2. 自动打标签阶段（条件执行）
- **auto-tag**: 仅在以下条件下运行：
  - 是 push 事件（不是 PR）
  - 不是标签推送
  - 提交消息包含 `[release]` 或手动触发时勾选创建发布
  - **所有构建任务成功**

执行步骤：
1. 获取最新标签
2. 计算新版本号
3. 更新 Cargo.toml
4. 提交版本变更
5. 创建并推送新标签

#### 3. 发布阶段（条件执行）
- **release**: 在以下条件下运行：
  - 是标签推送 或 auto-tag 成功
  - **所有构建任务成功**

执行步骤：
1. 下载所有平台的构建产物
2. 创建压缩包
3. 生成校验和
4. 创建 GitHub Release
5. 上传所有文件

### 工作流优势

1. **原子性**：构建失败不会创建标签
2. **可靠性**：只有所有平台构建成功才发布
3. **简洁性**：一个文件管理所有流程
4. **灵活性**：支持多种触发方式

## 构建产物

每次发布会生成以下文件：

| 文件 | 平台 | 格式 | 说明 |
|------|------|------|------|
| `mira-windows-x64.zip` | Windows | ZIP | Windows 10/11 x64 |
| `mira-macos-x64.tar.gz` | macOS | TAR.GZ | macOS 11+ (Intel/ARM) |
| `mira-linux-x64.tar.gz` | Linux | TAR.GZ | Ubuntu 20.04+ |
| `checksums.txt` | 所有 | TXT | SHA256 校验和 |

## 回滚版本

如果发布的版本有问题，可以：

### 1. 删除标签和发布

```bash
# 删除本地标签
git tag -d v1.0.0

# 删除远程标签
git push origin :refs/tags/v1.0.0
```

然后在 GitHub 上手动删除 Release。

### 2. 发布修复版本

```bash
# 修复问题
git add .
git commit -m "fix: critical bug [release]"
git push origin main
```

这将自动创建新的修复版本（例如 v1.0.1）。

## 故障排除

### 构建失败

1. 检查 [Actions 页面](https://github.com/Vogadero/Mira/actions) 的错误日志
2. 确保所有依赖项都正确配置
3. 验证 Rust 版本符合要求
4. 检查平台特定的依赖

### 标签创建失败

1. 确保有 `contents: write` 权限
2. 检查标签是否已存在
3. 验证版本号格式正确

### 发布未创建

1. 确保标签以 `v` 开头（例如 `v1.0.0`）
2. 检查工作流是否成功完成
3. 验证 `GITHUB_TOKEN` 权限

## 最佳实践

1. **频繁发布小版本**：比大版本更容易管理和回滚
2. **保持 CHANGELOG**：记录每个版本的变更
3. **测试后再发布**：确保质量
4. **语义化版本**：遵循版本号规范
5. **清晰的发布说明**：帮助用户了解变更

## 示例发布流程

### 场景：修复 Bug

```bash
# 1. 修复 Bug
git checkout -b fix/camera-crash
# ... 修复代码 ...
git add .
git commit -m "fix: resolve camera crash on startup"

# 2. 合并到主分支
git checkout main
git merge fix/camera-crash

# 3. 发布修复版本
git commit --allow-empty -m "chore: release bug fix [release]"
git push origin main

# 4. 自动流程会：
#    ✅ 构建 Windows、macOS、Linux 版本
#    ✅ 等待所有构建成功
#    ✅ 创建 v1.0.1 标签
#    ✅ 创建 GitHub Release
#    ✅ 上传所有构建产物
```

**如果构建失败**：
- ❌ 不会创建标签
- ❌ 不会创建发布
- ✅ 可以修复后重新推送

### 场景：新功能

```bash
# 1. 开发新功能
git checkout -b feature/new-shape
# ... 开发代码 ...
git add .
git commit -m "feat: add star shape mask"

# 2. 合并到主分支
git checkout main
git merge feature/new-shape

# 3. 手动触发发布
# 访问 Actions 页面
# 选择 "Build and Release" 工作流
# 点击 "Run workflow"
# 选择 version_bump: "minor"
# 勾选 create_release
# 点击 "Run workflow"

# 4. 自动流程会：
#    ✅ 构建所有平台
#    ✅ 等待构建成功
#    ✅ 创建 v1.1.0 标签
#    ✅ 创建 Release
```

### 场景：紧急修复（跳过自动标签）

```bash
# 1. 修复严重问题
git add .
git commit -m "fix: critical security issue"
git push origin main

# 2. 手动创建标签（立即发布）
git tag -a v1.0.2 -m "Emergency security fix"
git push origin v1.0.2

# 3. 自动流程会：
#    ✅ 构建所有平台
#    ✅ 直接创建 Release（跳过 auto-tag）
```

## 相关文档

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [SETUP_GUIDE.md](SETUP_GUIDE.md) - 环境配置
- [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) - 部署指南

## 联系方式

如有问题，请：
- 创建 [Issue](https://github.com/Vogadero/Mira/issues)
- 参与 [Discussions](https://github.com/Vogadero/Mira/discussions)
- 发送邮件至 15732651140@163.com
