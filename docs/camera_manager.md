# CameraManager 实现文档

## 概述

CameraManager 是 Mira 应用程序的核心组件之一，负责管理摄像头设备的枚举、打开、关闭和视频帧捕获。它使用 `nokhwa` 库提供跨平台的摄像头访问能力。

## 功能特性

### 1. 设备枚举
- 自动检测系统中所有可用的摄像头设备
- 提供设备名称、描述和索引信息
- 支持多次枚举，结果保持一致

### 2. 设备管理
- 支持按索引打开指定的摄像头设备
- 自动处理设备切换（关闭当前设备再打开新设备）
- 安全的设备关闭和资源释放

### 3. 视频捕获
- 支持实时视频帧捕获
- 自动管理视频流的开始和停止
- 提供 RGB8 格式的帧数据

### 4. 状态管理
- 跟踪当前打开的设备索引
- 监控视频流捕获状态
- 提供设备和状态查询接口

### 5. 错误处理
- 完善的错误类型定义和映射
- 智能的 nokhwa 错误转换
- 友好的用户错误消息
- 详细的错误日志记录

### 6. 重试机制
- 捕获失败时自动重试（默认最多 3 次）
- 重试间隔为 100 毫秒
- 每次重试前尝试重启视频流
- 可配置的最大重试次数

### 7. 权限和占用检测
- 主动检查摄像头访问权限
- 检测设备是否被其他应用占用
- 提供设备可用性状态查询
- 友好的中文错误消息
- 详细的日志记录

## API 文档

### 结构体

#### `CameraManager`
摄像头管理器主结构体。

```rust
pub struct CameraManager {
    camera: Option<CallbackCamera>,
    devices: Vec<CameraInfo>,
    current_device_index: Option<usize>,
    is_capturing: bool,
}
```

#### `CameraInfo`
摄像头设备信息。

```rust
pub struct CameraInfo {
    pub index: usize,        // 设备索引
    pub name: String,        // 设备名称
    pub description: String, // 设备描述
}
```

#### `Frame`
视频帧数据。

```rust
pub struct Frame {
    pub data: Vec<u8>,       // 像素数据
    pub width: u32,          // 帧宽度
    pub height: u32,         // 帧高度
    pub format: PixelFormat, // 像素格式
}
```

#### `PixelFormat`
支持的像素格式。

```rust
pub enum PixelFormat {
    RGB8,    // 8位RGB格式
    RGBA8,   // 8位RGBA格式
    YUV420,  // YUV420格式
}
```

### 方法

#### `new() -> Result<Self, CameraError>`
创建新的摄像头管理器实例。

- **返回**: 成功时返回 `CameraManager` 实例，失败时返回 `CameraError`
- **错误**: 如果系统中没有摄像头设备，返回 `CameraError::NoDeviceFound`

#### `enumerate_devices(&mut self) -> Result<Vec<CameraInfo>, CameraError>`
枚举所有可用的摄像头设备。

- **返回**: 成功时返回设备信息列表，失败时返回 `CameraError`
- **错误**: 如果无法访问摄像头系统，返回 `CameraError::NoDeviceFound`

#### `open_device(&mut self, index: usize) -> Result<(), CameraError>`
打开指定索引的摄像头设备。

- **参数**: `index` - 要打开的设备索引
- **返回**: 成功时返回 `()`，失败时返回 `CameraError`
- **错误**: 
  - `CameraError::NoDeviceFound` - 索引无效
  - `CameraError::DeviceInUse` - 设备被占用
  - `CameraError::PermissionDenied` - 权限被拒绝

#### `close_device(&mut self) -> Result<(), CameraError>`
关闭当前打开的摄像头设备。

- **返回**: 成功时返回 `()`，失败时返回 `CameraError`
- **注意**: 如果没有打开的设备，此方法不执行任何操作

#### `capture_frame(&mut self) -> Result<Frame, CameraError>`
捕获一帧视频数据。

- **返回**: 成功时返回 `Frame` 实例，失败时返回 `CameraError`
- **错误**: 
  - `CameraError::CaptureError` - 没有打开的设备或捕获失败
- **注意**: 首次调用时会自动开始视频流

#### `current_device(&self) -> Option<&CameraInfo>`
获取当前打开设备的信息。

- **返回**: 如果有设备打开，返回 `Some(&CameraInfo)`，否则返回 `None`

#### `current_device_index(&self) -> Option<usize>`
获取当前打开设备的索引。

- **返回**: 如果有设备打开，返回 `Some(usize)`，否则返回 `None`

#### `is_capturing(&self) -> bool`
检查是否正在捕获视频流。

- **返回**: 如果正在捕获，返回 `true`，否则返回 `false`

#### `devices(&self) -> &[CameraInfo]`
获取所有设备的列表。

- **返回**: 设备信息数组的引用

## 使用示例

### 基本使用流程

```rust
use mira::camera::CameraManager;
use mira::error::CameraError;

fn main() -> Result<(), CameraError> {
    // 1. 创建摄像头管理器
    let mut manager = CameraManager::new()?;
    
    // 2. 获取设备列表
    let devices = manager.devices();
    println!("找到 {} 个摄像头设备", devices.len());
    
    // 3. 打开第一个设备
    if !devices.is_empty() {
        manager.open_device(0)?;
        println!("已打开设备: {}", devices[0].name);
    }
    
    // 4. 捕获视频帧
    for i in 0..10 {
        match manager.capture_frame() {
            Ok(frame) => {
                println!("捕获第 {} 帧: {}x{}", i + 1, frame.width, frame.height);
            }
            Err(e) => {
                eprintln!("捕获失败: {}", e);
                break;
            }
        }
    }
    
    // 5. 关闭设备
    manager.close_device()?;
    
    Ok(())
}
```

### 设备切换

```rust
// 枚举设备并选择
let devices = manager.enumerate_devices()?;
for (i, device) in devices.iter().enumerate() {
    println!("{}: {} ({})", i, device.name, device.description);
}

// 切换到不同设备
manager.open_device(0)?;  // 打开第一个设备
// ... 使用设备 ...
manager.open_device(1)?;  // 切换到第二个设备（自动关闭第一个）
```

### 错误处理

```rust
match manager.open_device(index) {
    Ok(()) => println!("设备打开成功"),
    Err(CameraError::NoDeviceFound) => {
        eprintln!("设备不存在或索引无效");
    }
    Err(CameraError::DeviceInUse) => {
        eprintln!("设备正被其他应用使用");
    }
    Err(CameraError::PermissionDenied) => {
        eprintln!("摄像头访问权限被拒绝");
    }
    Err(e) => eprintln!("其他错误: {}", e),
}
```

## 技术实现细节

### nokhwa 集成
- 使用 `nokhwa::query()` 函数枚举设备
- 使用 `CallbackCamera` 进行视频捕获
- 默认请求最高帧率的格式
- 支持 RGB8 像素格式输出

### 内存管理
- 使用 RAII 模式确保资源自动释放
- 实现 `Drop` trait 进行清理
- 避免内存泄漏和资源占用

### 线程安全
- 当前实现不是线程安全的
- 如需多线程使用，需要外部同步机制

### 平台支持
- Windows: 使用 DirectShow 后端
- macOS: 使用 AVFoundation 后端
- Linux: 使用 V4L2 后端（通过 nokhwa）

## 性能考虑

### 帧率
- 目标帧率: 30 FPS 或更高
- 实际帧率取决于设备能力和系统性能

### 内存使用
- 每帧内存使用: width × height × 3 字节（RGB8）
- 建议及时处理帧数据，避免积累

### CPU 使用
- 帧捕获和格式转换会消耗 CPU 资源
- 建议在专用线程中进行视频处理

## 已知限制

1. **单设备限制**: 同时只能打开一个摄像头设备
2. **格式限制**: 目前只支持 RGB8 输出格式
3. **同步 API**: 所有操作都是同步的，可能阻塞调用线程
4. **权限依赖**: 需要系统摄像头访问权限

## 未来改进

1. **异步支持**: 添加异步 API 支持
2. **多格式支持**: 支持更多像素格式
3. **多设备支持**: 同时管理多个摄像头设备
4. **性能优化**: 减少内存分配和拷贝
5. **配置选项**: 支持分辨率、帧率等参数配置

## 测试

项目包含完整的测试套件：

- **单元测试**: 测试各个方法的基本功能
- **集成测试**: 测试完整的工作流程
- **模拟测试**: 使用模拟数据进行测试
- **错误测试**: 验证错误处理逻辑

运行测试：
```bash
cargo test camera
```

运行示例：
```bash
cargo run --example camera_demo
```

## 错误处理详解

### 错误类型

CameraManager 使用 `CameraError` 枚举来表示不同类型的错误：

#### NoDeviceFound
- **描述**: 系统中没有找到任何摄像头设备
- **触发场景**: 
  - 系统没有连接摄像头硬件
  - 摄像头驱动程序未安装或损坏
  - 摄像头被系统禁用
- **用户消息**: "未检测到摄像头设备，请连接摄像头后重试"
- **处理建议**: 检查硬件连接和驱动程序

#### DeviceInUse
- **描述**: 摄像头设备正被其他应用程序占用
- **触发场景**:
  - 其他应用程序正在使用摄像头
  - 设备资源未正确释放
  - 系统级摄像头服务占用设备
- **用户消息**: "摄像头正被其他应用使用，请关闭占用摄像头的应用"
- **处理建议**: 关闭其他摄像头应用程序

#### PermissionDenied
- **描述**: 应用程序没有访问摄像头的权限
- **触发场景**:
  - 用户拒绝了摄像头权限请求
  - 系统隐私设置禁止应用访问摄像头
  - 企业策略限制摄像头访问
- **用户消息**: "摄像头访问权限被拒绝，请在系统设置中允许 Mira 访问摄像头"
- **处理建议**: 在系统设置中授权摄像头访问

#### CaptureError
- **描述**: 视频帧捕获过程中发生的各种错误
- **触发场景**:
  - 摄像头硬件故障
  - 视频流中断
  - 内存不足
  - 格式转换失败
- **用户消息**: "视频捕获失败: [具体错误信息]"
- **处理建议**: 重启应用程序或检查硬件

### nokhwa 错误映射

CameraManager 智能地将 nokhwa 库的错误映射到用户友好的 CameraError：

```rust
// 设备占用相关
UnsupportedOperationError -> DeviceInUse
OpenDeviceError("busy"|"in use") -> DeviceInUse

// 权限相关  
GetPropertyError -> PermissionDenied
SetPropertyError -> PermissionDenied
OpenDeviceError("permission"|"access"|"denied") -> PermissionDenied

// 捕获相关
ReadFrameError -> CaptureError
ProcessFrameError -> CaptureError
StructureError -> CaptureError
GeneralError -> CaptureError
```

### 重试机制

当视频帧捕获失败时，CameraManager 会自动执行重试：

1. **检测失败**: 捕获帧时发生错误
2. **记录重试**: 增加重试计数器
3. **等待间隔**: 等待 100 毫秒
4. **重启流**: 尝试重新启动视频流
5. **重新捕获**: 再次尝试捕获帧
6. **成功恢复**: 重置重试计数器并记录恢复日志
7. **达到上限**: 返回最终错误

重试配置：
- 默认最大重试次数：3 次
- 重试间隔：100 毫秒
- 可通过 `set_max_retries()` 调整
- 设置为 0 可禁用重试

### 权限检查

`check_device_permissions()` 方法主动检查摄像头权限：

```rust
match manager.check_device_permissions() {
    Ok(()) => println!("权限检查通过"),
    Err(CameraError::NoDeviceFound) => println!("未找到设备"),
    Err(CameraError::PermissionDenied) => println!("权限被拒绝"),
    Err(e) => println!("其他错误: {}", e),
}
```

### 占用检测

`is_device_in_use()` 方法检测特定设备是否被占用：

```rust
for (index, device) in manager.devices().iter().enumerate() {
    let in_use = manager.is_device_in_use(index);
    println!("设备 {}: {} - {}", 
             index, 
             device.name, 
             if in_use { "被占用" } else { "可用" });
}
```

## 最佳实践

### 1. 错误处理
```rust
match manager.open_device(0) {
    Ok(()) => {
        // 设备打开成功，继续操作
    }
    Err(CameraError::NoDeviceFound) => {
        // 显示"未找到设备"的用户界面
    }
    Err(CameraError::DeviceInUse) => {
        // 提示用户关闭其他摄像头应用
    }
    Err(CameraError::PermissionDenied) => {
        // 引导用户到系统设置授权
    }
    Err(e) => {
        // 记录错误日志，显示通用错误消息
        log::error!("打开摄像头失败: {}", e);
    }
}
```

### 2. 重试配置
```rust
// 对于关键应用，增加重试次数
manager.set_max_retries(5);

// 对于实时应用，禁用重试以减少延迟
manager.set_max_retries(0);
```

### 3. 权限预检
```rust
// 在尝试打开设备前先检查权限
if let Err(e) = manager.check_device_permissions() {
    // 处理权限问题
    return Err(e);
}

// 权限检查通过，继续打开设备
manager.open_device(0)?;
```

### 4. 设备选择
```rust
// 选择第一个可用的设备
let mut selected_device = None;
for device in manager.devices() {
    if !manager.is_device_in_use(device.index) {
        selected_device = Some(device.index);
        break;
    }
}

if let Some(index) = selected_device {
    manager.open_device(index)?;
} else {
    return Err(CameraError::DeviceInUse);
}
```