# GitHub Actions 工作流说明

## build.yml - 构建和发布工作流

### 触发条件

| 事件 | 分支/标签 | 说明 |
|------|----------|------|
| `push` | main, master | 推送代码到主分支 |
| `push` | v* | 推送标签（如 v1.0.0） |
| `pull_request` | main, master | 创建 PR |
| `workflow_dispatch` | 任意 | 手动触发 |

### Jobs 执行条件

#### 1. build-windows / build-macos / build-linux

**执行条件**：总是执行（所有触发事件）

**说明**：
- 构建所有平台的版本
- 上传构建产物
- 无论什么触发方式都会执行

#### 2. auto-tag

**执行条件**：满足以下任一条件

| 场景 | 条件 | 示例 |
|------|------|------|
| **提交消息包含 [release]** | `github.event_name == 'push'` <br> `contains(github.event.head_commit.message, '[release]')` <br> `!startsWith(github.ref, 'refs/tags/')` | `git commit -m "feat: new feature [release]"` <br> `git push origin main` |
| **手动触发并勾选创建发布** | `github.event_name == 'workflow_dispatch'` <br> `github.event.inputs.create_release == 'true'` <br> `!startsWith(github.ref, 'refs/tags/')` | 在 Actions 页面手动触发 <br> 勾选 "Create release after build" |

**不执行的情况**：
- ❌ 普通 push（提交消息不包含 `[release]`）
- ❌ Pull Request
- ❌ 标签推送（已有标签，不需要再创建）
- ❌ 手动触发但未勾选 "Create release after build"

**执行内容**：
1. 获取最新标签
2. 计算新版本号
3. 更新 Cargo.toml
4. 提交版本变更
5. 创建并推送新标签

#### 3. release

**执行条件**：满足以下所有条件

| 条件 | 说明 |
|------|------|
| `always()` | 即使前面的 job 被跳过也检查 |
| `startsWith(github.ref, 'refs/tags/v') OR needs.auto-tag.outputs.should_release == 'true'` | 是标签推送 或 auto-tag 成功 |
| `needs.build-windows.result == 'success'` | Windows 构建成功 |
| `needs.build-macos.result == 'success'` | macOS 构建成功 |
| `needs.build-linux.result == 'success'` | Linux 构建成功 |

**执行内容**：
1. 下载所有平台的构建产物
2. 创建压缩包
3. 生成校验和
4. 创建 GitHub Release
5. 上传所有文件

## 使用场景

### 场景 1：开发中的普通提交

```bash
git commit -m "fix: minor bug"
git push origin main
```

**结果**：
- ✅ build-windows: 执行
- ✅ build-macos: 执行
- ✅ build-linux: 执行
- ❌ auto-tag: 跳过（没有 [release]）
- ❌ release: 跳过（auto-tag 未执行）

### 场景 2：发布新版本（提交消息）

```bash
git commit -m "feat: add new feature [release]"
git push origin main
```

**结果**：
- ✅ build-windows: 执行
- ✅ build-macos: 执行
- ✅ build-linux: 执行
- ✅ auto-tag: 执行（检测到 [release]）
  - 创建新标签（如 v1.0.1）
- ✅ release: 执行（auto-tag 成功）
  - 创建 GitHub Release

### 场景 3：手动触发（不创建发布）

在 Actions 页面：
- 点击 "Run workflow"
- version_bump: patch
- create_release: ❌ 不勾选

**结果**：
- ✅ build-windows: 执行
- ✅ build-macos: 执行
- ✅ build-linux: 执行
- ❌ auto-tag: 跳过（未勾选 create_release）
- ❌ release: 跳过（auto-tag 未执行）

### 场景 4：手动触发（创建发布）

在 Actions 页面：
- 点击 "Run workflow"
- version_bump: minor
- create_release: ✅ 勾选

**结果**：
- ✅ build-windows: 执行
- ✅ build-macos: 执行
- ✅ build-linux: 执行
- ✅ auto-tag: 执行（勾选了 create_release）
  - 创建新标签（如 v1.1.0）
- ✅ release: 执行（auto-tag 成功）
  - 创建 GitHub Release

### 场景 5：手动推送标签

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

**结果**：
- ✅ build-windows: 执行
- ✅ build-macos: 执行
- ✅ build-linux: 执行
- ❌ auto-tag: 跳过（是标签推送）
- ✅ release: 执行（检测到标签推送）
  - 创建 GitHub Release

### 场景 6：Pull Request

```bash
git checkout -b feature/new-feature
git commit -m "feat: new feature"
git push origin feature/new-feature
# 创建 PR
```

**结果**：
- ✅ build-windows: 执行
- ✅ build-macos: 执行
- ✅ build-linux: 执行
- ❌ auto-tag: 跳过（是 PR）
- ❌ release: 跳过（auto-tag 未执行）

## 调试技巧

### 查看为什么 job 被跳过

1. 打开 Actions 运行页面
2. 点击被跳过的 job
3. 查看 "Set up job" 部分的条件评估

### 常见问题

#### Q: 为什么 auto-tag 被跳过？

**A**: 检查以下条件：

1. **提交消息是否包含 `[release]`？**
   ```bash
   # 正确
   git commit -m "feat: new feature [release]"
   
   # 错误（缺少 [release]）
   git commit -m "feat: new feature"
   ```

2. **是否是 push 事件？**
   - PR 不会触发 auto-tag
   - 标签推送不会触发 auto-tag

3. **手动触发时是否勾选了 "Create release after build"？**
   - 必须勾选才会执行 auto-tag

#### Q: 为什么 release 被跳过？

**A**: 检查以下条件：

1. **是否有标签推送或 auto-tag 成功？**
   - 必须满足其中之一

2. **所有构建是否成功？**
   - Windows、macOS、Linux 必须全部成功
   - 任何一个失败都会跳过 release

#### Q: 如何只构建不发布？

**A**: 两种方式：

1. **普通 push（不包含 [release]）**
   ```bash
   git commit -m "feat: new feature"
   git push origin main
   ```

2. **手动触发（不勾选 create_release）**
   - 在 Actions 页面手动触发
   - 不勾选 "Create release after build"

#### Q: 如何强制创建发布？

**A**: 三种方式：

1. **提交消息包含 [release]**
   ```bash
   git commit -m "chore: release [release]"
   git push origin main
   ```

2. **手动触发并勾选**
   - 在 Actions 页面手动触发
   - 勾选 "Create release after build"

3. **手动推送标签**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

## 工作流配置

### 手动触发参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `version_bump` | choice | patch | 版本提升类型（major/minor/patch） |
| `create_release` | boolean | false | 是否在构建后创建发布 |

### 环境变量

| 变量 | 值 | 说明 |
|------|-----|------|
| `CARGO_TERM_COLOR` | always | Cargo 输出彩色日志 |
| `MIRA_SHOW_CONSOLE` | false | Windows 构建时隐藏控制台 |

## 相关文档

- [RELEASE_PROCESS.md](../../RELEASE_PROCESS.md) - 发布流程详解
- [WORKFLOW_DIAGRAM.md](../../WORKFLOW_DIAGRAM.md) - 工作流程图
- [build.yml](build.yml) - 工作流配置文件
