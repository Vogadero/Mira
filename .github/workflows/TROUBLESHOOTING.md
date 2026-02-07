# GitHub Actions 故障排除

## 常见错误及解决方案

### 错误 1: "GitHub Releases requires a tag"

**错误信息**：
```
Error: ⚠️ GitHub Releases requires a tag
```

**原因**：
`softprops/action-gh-release` 需要一个 Git 标签才能创建 release，但是：
1. 当 auto-tag 创建新标签后，release job 的 `github.ref` 还不是新标签
2. 需要明确指定 `tag_name` 参数

**解决方案**：
已在工作流中修复：
```yaml
- name: Create Release
  uses: softprops/action-gh-release@v1
  with:
    tag_name: ${{ steps.get_version.outputs.tag_name }}  # 明确指定标签
    files: ...
```

**相关修改**：
- 添加了 `tag_name` 参数
- checkout 时使用 `ref: ${{ needs.auto-tag.outputs.new_tag || github.ref }}`
- 添加了 `Fetch latest tags` 步骤确保获取最新标签

### 错误 2: auto-tag job 被跳过

**症状**：
在 Actions 页面看到 "This job was skipped"

**原因**：
auto-tag 有严格的执行条件，只在以下情况运行：
1. push 事件 + 提交消息包含 `[release]`
2. 手动触发 + 勾选 "Create release after build"

**解决方案**：

#### 方法 1：在提交消息中添加 [release]
```bash
git commit -m "feat: new feature [release]"
git push origin main
```

#### 方法 2：手动触发并勾选选项
1. 访问 Actions 页面
2. 选择 "Build and Release"
3. 点击 "Run workflow"
4. ✅ 勾选 "Create release after build"
5. 点击 "Run workflow"

#### 方法 3：直接推送标签（跳过 auto-tag）
```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

### 错误 3: release job 被跳过

**症状**：
构建成功，auto-tag 成功，但 release 被跳过

**原因**：
release job 需要满足所有条件：
- 是标签推送 或 auto-tag 成功
- 所有构建任务（Windows、macOS、Linux）都成功

**检查清单**：
- [ ] Windows 构建成功？
- [ ] macOS 构建成功？
- [ ] Linux 构建成功？
- [ ] auto-tag 执行成功？或者是标签推送？

**解决方案**：
1. 检查失败的构建任务
2. 修复构建问题
3. 重新触发工作流

### 错误 4: 构建失败

**常见原因**：

#### Windows 构建失败
- 缺少 Visual Studio Build Tools
- winres 编译失败
- 链接器错误

**解决方案**：
```yaml
# 确保在 CI 环境中有正确的工具链
- name: Install Rust
  uses: dtolnay/rust-toolchain@stable
```

#### macOS 构建失败
- 缺少系统依赖
- 权限问题

**解决方案**：
```bash
brew install bc
```

#### Linux 构建失败
- 缺少系统库

**解决方案**：
```bash
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  libudev-dev \
  libv4l-dev \
  libgtk-3-dev \
  libglib2.0-dev \
  libappindicator3-dev \
  libxdo-dev
```

### 错误 5: 标签已存在

**错误信息**：
```
fatal: tag 'v0.1.0' already exists
```

**原因**：
尝试创建已存在的标签

**解决方案**：

#### 删除本地和远程标签
```bash
# 删除本地标签
git tag -d v0.1.0

# 删除远程标签
git push origin :refs/tags/v0.1.0
```

#### 或者创建新版本
```bash
# 手动触发工作流，选择更高的版本号
# 或者在提交消息中使用 [release]
```

### 错误 6: 权限错误

**错误信息**：
```
Error: Resource not accessible by integration
```

**原因**：
GitHub Actions 没有足够的权限

**解决方案**：
确保工作流有正确的权限：
```yaml
permissions:
  contents: write  # 需要写权限来创建 release 和推送标签
```

**检查仓库设置**：
1. Settings → Actions → General
2. Workflow permissions
3. 选择 "Read and write permissions"

### 错误 7: 构建产物未找到

**错误信息**：
```
Error: Unable to find any artifacts for the associated workflow
```

**原因**：
- 构建任务失败
- artifact 名称不匹配
- artifact 已过期

**解决方案**：
1. 确保构建任务成功
2. 检查 artifact 名称一致：
   ```yaml
   # 上传
   - name: Upload artifact
     uses: actions/upload-artifact@v4
     with:
       name: mira-windows-x64  # 名称必须匹配
   
   # 下载
   - name: Download artifact
     uses: actions/download-artifact@v4
     with:
       name: mira-windows-x64  # 名称必须匹配
   ```

## 调试技巧

### 1. 查看详细日志

在工作流中添加调试输出：
```yaml
- name: Debug info
  run: |
    echo "Event name: ${{ github.event_name }}"
    echo "Ref: ${{ github.ref }}"
    echo "Commit message: ${{ github.event.head_commit.message }}"
    echo "Auto-tag new_tag: ${{ needs.auto-tag.outputs.new_tag }}"
    echo "Auto-tag should_release: ${{ needs.auto-tag.outputs.should_release }}"
```

### 2. 检查条件评估

在 Actions 页面：
1. 点击被跳过的 job
2. 查看 "Set up job" 部分
3. 查看条件评估结果

### 3. 本地测试

```bash
# 测试构建
cargo build --release

# 测试脚本
./scripts/build_release.sh  # macOS/Linux
.\scripts\build_release.ps1  # Windows
```

### 4. 使用 act 本地运行 Actions

```bash
# 安装 act
brew install act  # macOS
# 或从 https://github.com/nektos/act 下载

# 运行工作流
act push
```

## 工作流执行流程

### 正常流程（带 [release]）

```
1. Push 代码 (包含 [release])
   ↓
2. build-windows ✅
   build-macos ✅
   build-linux ✅
   ↓
3. auto-tag ✅
   - 创建标签 v0.1.0
   - 输出: new_tag=v0.1.0, should_release=true
   ↓
4. release ✅
   - Checkout 代码（ref=v0.1.0）
   - Fetch 最新标签
   - 下载构建产物
   - 创建 release（tag_name=v0.1.0）
   ↓
5. 完成 ✅
```

### 失败流程（构建失败）

```
1. Push 代码 (包含 [release])
   ↓
2. build-windows ✅
   build-macos ❌ (失败)
   build-linux ✅
   ↓
3. auto-tag ⏭️ (跳过 - 构建失败)
   ↓
4. release ⏭️ (跳过 - auto-tag 未执行)
   ↓
5. 停止 ❌
```

### 手动标签流程

```
1. 手动推送标签 v0.1.0
   ↓
2. build-windows ✅
   build-macos ✅
   build-linux ✅
   ↓
3. auto-tag ⏭️ (跳过 - 是标签推送)
   ↓
4. release ✅
   - Checkout 代码（ref=refs/tags/v0.1.0）
   - 下载构建产物
   - 创建 release（tag_name=v0.1.0）
   ↓
5. 完成 ✅
```

## 快速检查清单

发布前检查：
- [ ] 代码已提交并推送
- [ ] 提交消息包含 `[release]`（如果使用自动标签）
- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] 版本号正确

发布失败时检查：
- [ ] 查看 Actions 页面的错误日志
- [ ] 检查哪个 job 失败或被跳过
- [ ] 验证触发条件是否满足
- [ ] 检查权限设置
- [ ] 验证标签是否已存在

## 相关文档

- [README.md](README.md) - 工作流说明
- [../../RELEASE_PROCESS.md](../../RELEASE_PROCESS.md) - 发布流程
- [../../WORKFLOW_DIAGRAM.md](../../WORKFLOW_DIAGRAM.md) - 流程图
- [build.yml](build.yml) - 工作流配置

## 获取帮助

如果问题仍未解决：
1. 查看 [GitHub Actions 文档](https://docs.github.com/en/actions)
2. 创建 [Issue](https://github.com/Vogadero/Mira/issues)
3. 在 [Discussions](https://github.com/Vogadero/Mira/discussions) 提问
