// 摄像头错误处理演示

use mira::camera::CameraManager;
use mira::error::CameraError;
use log::{error, info, warn};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    env_logger::init();
    
    info!("开始摄像头错误处理演示");
    
    // 演示1: 创建摄像头管理器和权限检查
    println!("=== 演示1: 创建摄像头管理器 ===");
    let mut manager = match CameraManager::new() {
        Ok(manager) => {
            info!("摄像头管理器创建成功");
            manager
        }
        Err(CameraError::NoDeviceFound) => {
            error!("未找到摄像头设备");
            println!("错误: {}", CameraError::NoDeviceFound);
            return Ok(());
        }
        Err(e) => {
            error!("创建摄像头管理器失败: {}", e);
            return Err(e.into());
        }
    };
    
    // 演示2: 权限检查
    println!("\n=== 演示2: 权限检查 ===");
    match manager.check_device_permissions() {
        Ok(()) => {
            info!("摄像头权限检查通过");
            println!("✓ 摄像头权限检查通过");
        }
        Err(e) => {
            warn!("摄像头权限检查失败: {}", e);
            println!("✗ 摄像头权限检查失败: {}", e);
        }
    }
    
    // 演示3: 设备枚举和占用检测
    println!("\n=== 演示3: 设备枚举和占用检测 ===");
    let devices = manager.devices();
    println!("找到 {} 个摄像头设备:", devices.len());
    
    for device in devices {
        println!("  设备 {}: {} ({})", device.index, device.name, device.description);
        
        // 检测设备是否被占用
        let in_use = manager.is_device_in_use(device.index);
        println!("    占用状态: {}", if in_use { "被占用" } else { "可用" });
    }
    
    // 演示4: 错误处理 - 尝试打开无效设备
    println!("\n=== 演示4: 错误处理 - 无效设备索引 ===");
    match manager.open_device(9999) {
        Ok(()) => {
            println!("意外成功打开了无效设备");
        }
        Err(e) => {
            println!("预期的错误: {}", e);
            info!("正确处理了无效设备索引错误");
        }
    }
    
    // 演示5: 重试机制配置
    println!("\n=== 演示5: 重试机制配置 ===");
    println!("当前重试次数: {}", manager.retry_count());
    println!("最大重试次数: {}", manager.max_retries());
    
    // 设置新的重试次数
    manager.set_max_retries(5);
    println!("设置最大重试次数为: {}", manager.max_retries());
    
    // 演示6: 尝试打开第一个设备（如果存在）
    if !devices.is_empty() {
        println!("\n=== 演示6: 打开设备和捕获测试 ===");
        let first_device_index = 0;
        
        match manager.open_device(first_device_index) {
            Ok(()) => {
                info!("成功打开设备 {}", first_device_index);
                println!("✓ 成功打开设备 {}: {}", first_device_index, devices[first_device_index].name);
                
                // 尝试捕获几帧来测试重试机制
                println!("尝试捕获 3 帧...");
                for i in 1..=3 {
                    match manager.capture_frame() {
                        Ok(frame) => {
                            println!("  帧 {}: {}x{} ({} 字节)", 
                                   i, frame.width, frame.height, frame.data.len());
                        }
                        Err(e) => {
                            warn!("捕获帧 {} 失败: {}", i, e);
                            println!("  帧 {} 失败: {}", i, e);
                        }
                    }
                }
                
                // 关闭设备
                if let Err(e) = manager.close_device() {
                    error!("关闭设备失败: {}", e);
                } else {
                    info!("设备已关闭");
                    println!("✓ 设备已关闭");
                }
            }
            Err(CameraError::DeviceInUse) => {
                warn!("设备被占用");
                println!("✗ 设备被其他应用占用: {}", CameraError::DeviceInUse);
            }
            Err(CameraError::PermissionDenied) => {
                warn!("权限被拒绝");
                println!("✗ 摄像头权限被拒绝: {}", CameraError::PermissionDenied);
            }
            Err(e) => {
                error!("打开设备失败: {}", e);
                println!("✗ 打开设备失败: {}", e);
            }
        }
    }
    
    // 演示7: 错误类型展示
    println!("\n=== 演示7: 所有错误类型的友好消息 ===");
    let error_examples = vec![
        CameraError::NoDeviceFound,
        CameraError::DeviceInUse,
        CameraError::PermissionDenied,
        CameraError::CaptureError("示例捕获错误".to_string()),
    ];
    
    for (i, error) in error_examples.iter().enumerate() {
        println!("  错误类型 {}: {}", i + 1, error);
    }
    
    info!("摄像头错误处理演示完成");
    println!("\n=== 演示完成 ===");
    
    Ok(())
}