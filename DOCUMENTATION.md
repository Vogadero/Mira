# Mira 文档索引

本文档提供 Mira 项目所有文档的索引和说明。

## 📚 文档分类

### 用户文档

#### 基础文档
- **[README.md](README.md)** - 项目主页（中文）
  - 项目概述和功能特性
  - 快速开始指南
  - 基本使用说明
  - 常见问题解答

- **[README_EN.md](README_EN.md)** - 项目主页（英文）
  - Project overview and features
  - Quick start guide
  - Basic usage instructions
  - FAQ

#### 安装和配置
- **[SETUP_GUIDE.md](SETUP_GUIDE.md)** - 环境配置指南
  - Windows 环境配置
  - macOS 环境配置
  - Linux 环境配置
  - 依赖安装说明
  - 故障排除

- **[DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)** - 部署指南
  - 本地构建步骤
  - Docker 构建方法
  - 代码签名流程
  - 分发打包说明
  - 自动化部署

- **[CONSOLE_CONFIGURATION.md](CONSOLE_CONFIGURATION.md)** - 控制台配置指南
  - 控制台显示/隐藏配置
  - Release 和 Debug 模式行为
  - 环境变量控制
  - 日志查看方法
  - 常见问题解答

#### 功能说明
- **[TRAY_ICON_GUIDE.md](TRAY_ICON_GUIDE.md)** - 系统托盘功能指南
  - 托盘图标说明
  - 右键菜单功能
  - 快捷键参考
  - 常见问题

### 开发文档

#### 发布流程
- **[RELEASE_PROCESS.md](RELEASE_PROCESS.md)** - 发布流程说明
  - 自动发布流程
  - 版本号规范
  - 发布检查清单
  - 工作流详解
  - 故障排除

- **[WORKFLOW_DIAGRAM.md](WORKFLOW_DIAGRAM.md)** - 工作流程图
  - 完整发布流程图
  - 三种触发方式对比
  - 失败处理流程
  - 工作流架构说明

- **[RELEASE_NOTES.md](RELEASE_NOTES.md)** - 发布说明
  - 如何获取构建版本
  - 支持的平台
  - 安装说明
  - 构建状态

#### 技术文档
- **[docs/camera_manager.md](docs/camera_manager.md)** - 摄像头管理模块
  - 模块架构
  - API 文档
  - 使用示例

- **[docs/window_manager.md](docs/window_manager.md)** - 窗口管理模块
  - 窗口交互实现
  - 拖拽和缩放
  - 旋转功能

- **[docs/shape_generation.md](docs/shape_generation.md)** - 形状生成算法
  - 形状遮罩实现
  - 算法说明
  - 性能优化

- **[docs/scaling_implementation.md](docs/scaling_implementation.md)** - 缩放实现
  - 缩放算法
  - 性能考虑
  - 边界处理

#### 构建脚本
- **[scripts/README.md](scripts/README.md)** - 构建脚本说明
  - Windows 构建脚本
  - macOS/Linux 构建脚本
  - 使用方法

### 规范文档

- **[.kiro/specs/mira/requirements.md](.kiro/specs/mira/requirements.md)** - 需求文档
  - 功能需求
  - 性能需求
  - 平台需求

- **[.kiro/specs/mira/design.md](.kiro/specs/mira/design.md)** - 设计文档
  - 架构设计
  - 模块设计
  - 接口设计

- **[.kiro/specs/mira/tasks.md](.kiro/specs/mira/tasks.md)** - 任务列表
  - 已完成任务
  - 进行中任务
  - 计划任务

## 📖 文档使用指南

### 新用户

如果你是第一次使用 Mira，建议按以下顺序阅读：

1. **[README.md](README.md)** - 了解项目概况
2. **[SETUP_GUIDE.md](SETUP_GUIDE.md)** - 配置开发环境（如果需要从源码构建）
3. **[TRAY_ICON_GUIDE.md](TRAY_ICON_GUIDE.md)** - 学习如何使用托盘功能

### 开发者

如果你想参与开发，建议阅读：

1. **[README.md](README.md)** - 项目概述和技术栈
2. **[SETUP_GUIDE.md](SETUP_GUIDE.md)** - 配置开发环境
3. **[docs/](docs/)** - 技术文档
4. **[.kiro/specs/mira/](kiro/specs/mira/)** - 规范文档
5. **[RELEASE_PROCESS.md](RELEASE_PROCESS.md)** - 发布流程

### 维护者

如果你是项目维护者，需要了解：

1. **[RELEASE_PROCESS.md](RELEASE_PROCESS.md)** - 发布新版本
2. **[DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)** - 部署和打包
3. **[.github/workflows/](github/workflows/)** - CI/CD 配置

## 🔍 快速查找

### 按主题查找

#### 安装和配置
- 环境配置 → [SETUP_GUIDE.md](SETUP_GUIDE.md)
- 部署打包 → [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)
- 系统要求 → [README.md#系统要求](README.md#系统要求)

#### 功能使用
- 基本操作 → [README.md#使用说明](README.md#使用说明)
- 托盘功能 → [TRAY_ICON_GUIDE.md](TRAY_ICON_GUIDE.md)
- 快捷键 → [README.md#快捷键参考](README.md#快捷键参考)

#### 开发相关
- 项目结构 → [README.md#项目结构](README.md#项目结构)
- 技术栈 → [README.md#技术栈](README.md#技术栈)
- API 文档 → [docs/](docs/)

#### 发布和部署
- 发布流程 → [RELEASE_PROCESS.md](RELEASE_PROCESS.md)
- 构建脚本 → [scripts/README.md](scripts/README.md)
- CI/CD → [.github/workflows/](github/workflows/)

### 按文件类型查找

#### Markdown 文档
```
根目录:
├── README.md                    # 项目主页（中文）
├── README_EN.md                 # 项目主页（英文）
├── SETUP_GUIDE.md               # 环境配置指南
├── DEPLOYMENT_GUIDE.md          # 部署指南
├── RELEASE_PROCESS.md           # 发布流程
├── RELEASE_NOTES.md             # 发布说明
├── TRAY_ICON_GUIDE.md           # 托盘功能指南
└── DOCUMENTATION.md             # 本文档

docs/:
├── camera_manager.md            # 摄像头管理
├── window_manager.md            # 窗口管理
├── shape_generation.md          # 形状生成
└── scaling_implementation.md    # 缩放实现

.kiro/specs/mira/:
├── requirements.md              # 需求文档
├── design.md                    # 设计文档
└── tasks.md                     # 任务列表

scripts/:
└── README.md                    # 构建脚本说明
```

## 📝 文档维护

### 更新文档

当你修改代码时，请同时更新相关文档：

- **新功能** → 更新 README.md 和相关技术文档
- **Bug 修复** → 更新 RELEASE_NOTES.md
- **API 变更** → 更新 docs/ 中的相应文档
- **配置变更** → 更新 SETUP_GUIDE.md 或 DEPLOYMENT_GUIDE.md

### 文档规范

- 使用 Markdown 格式
- 保持简洁清晰
- 提供代码示例
- 包含截图（如果适用）
- 及时更新过时内容

### 文档审查

在提交 PR 时，请确保：

- [ ] 文档与代码同步
- [ ] 没有拼写错误
- [ ] 链接正确有效
- [ ] 格式统一规范

## 🌐 多语言支持

目前支持的语言：

- **简体中文** - 主要文档语言
- **English** - README_EN.md

计划支持：

- 日本语
- 한국어

## 📧 反馈和建议

如果你发现文档有问题或有改进建议：

1. 创建 [Issue](https://github.com/Vogadero/Mira/issues)
2. 提交 Pull Request
3. 在 [Discussions](https://github.com/Vogadero/Mira/discussions) 讨论
4. 发送邮件至 15732651140@163.com

## 📜 许可证

所有文档采用 [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) 许可证。

---

<div align="center">

**感谢阅读 Mira 文档！**

如有疑问，欢迎随时联系我们。

</div>
