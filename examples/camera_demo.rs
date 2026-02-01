// 摄像头管理器演示程序

use mira::camera::CameraManager;
use mira::error::CameraError;
use log::{error, info};

fn main() {
    // 初始化日志系统
    env_logger::init();
    
    info!("摄像头管理器演示程序启动");
    
    // 创建摄像头管理器
    let mut manager = match CameraManager::new() {
        Ok(manager) => {
            info!("成功创建摄像头管理器");
            manager
        }
        Err(CameraError::NoDeviceFound) => {
            error!("未找到摄像头设备");
            return;
        }
        Err(e) => {
            error!("创建摄像头管理器失败: {}", e);
            return;
        }
    };
    
    // 显示所有可用设备
    let devices = manager.devices();
    info!("找到 {} 个摄像头设备:", devices.len());
    for device in devices {
        info!("  设备 {}: {} ({})", device.index, device.name, device.description);
    }
    
    if devices.is_empty() {
        error!("没有可用的摄像头设备");
        return;
    }
    
    // 打开第一个设备
    let first_device_index = 0;
    match manager.open_device(first_device_index) {
        Ok(()) => {
            info!("成功打开设备 {}", first_device_index);
            
            if let Some(current_device) = manager.current_device() {
                info!("当前设备: {} ({})", current_device.name, current_device.description);
            }
        }
        Err(e) => {
            error!("打开设备失败: {}", e);
            return;
        }
    }
    
    // 尝试捕获几帧
    info!("开始捕获视频帧...");
    for i in 1..=5 {
        match manager.capture_frame() {
            Ok(frame) => {
                info!(
                    "捕获第 {} 帧: {}x{}, {} 字节, 格式: {:?}",
                    i, frame.width, frame.height, frame.data.len(), frame.format
                );
                
                // 简单验证帧数据
                if frame.data.is_empty() {
                    error!("警告: 帧数据为空");
                } else {
                    let expected_size = (frame.width * frame.height * 3) as usize;
                    if frame.data.len() != expected_size {
                        error!(
                            "警告: 帧数据大小不匹配，期望 {} 字节，实际 {} 字节",
                            expected_size, frame.data.len()
                        );
                    }
                }
            }
            Err(e) => {
                error!("捕获第 {} 帧失败: {}", i, e);
                break;
            }
        }
        
        // 短暂延迟
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    // 关闭设备
    match manager.close_device() {
        Ok(()) => info!("成功关闭摄像头设备"),
        Err(e) => error!("关闭设备时出错: {}", e),
    }
    
    info!("演示程序结束");
}