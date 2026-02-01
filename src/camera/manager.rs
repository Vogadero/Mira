// 摄像头管理器实现

use crate::error::CameraError;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution},
    Camera, CallbackCamera,
};
use log::{debug, error, info, warn};

/// 摄像头设备信息
#[derive(Debug, Clone, PartialEq)]
pub struct CameraInfo {
    pub index: usize,
    pub name: String,
    pub description: String,
}

/// 视频帧数据
#[derive(Debug, Clone)]
pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

/// 像素格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    RGB8,
    RGBA8,
    YUV420,
}

/// 摄像头管理器
pub struct CameraManager {
    camera: Option<CallbackCamera>,
    devices: Vec<CameraInfo>,
    current_device_index: Option<usize>,
    is_capturing: bool,
    retry_count: u32,
    max_retries: u32,
}

impl CameraManager {
    /// 创建新的摄像头管理器
    pub fn new() -> Result<Self, CameraError> {
        info!("创建摄像头管理器");
        
        let mut manager = Self {
            camera: None,
            devices: Vec::new(),
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        // 初始化时枚举设备
        manager.enumerate_devices()?;
        
        Ok(manager)
    }

    /// 创建空的摄像头管理器（用于测试或无摄像头环境）
    pub fn new_empty() -> Self {
        Self {
            camera: None,
            devices: Vec::new(),
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// 枚举所有可用的摄像头设备
    pub fn enumerate_devices(&mut self) -> Result<Vec<CameraInfo>, CameraError> {
        debug!("开始枚举摄像头设备");
        
        // 使用 nokhwa 的 query 函数枚举设备
        let devices = nokhwa::query(nokhwa::utils::ApiBackend::Auto)
            .map_err(|e| {
                error!("枚举摄像头设备失败: {}", e);
                CameraError::NoDeviceFound
            })?;
        
        if devices.is_empty() {
            warn!("未找到任何摄像头设备");
            return Err(CameraError::NoDeviceFound);
        }
        
        // 转换为我们的 CameraInfo 格式
        self.devices = devices
            .into_iter()
            .enumerate()
            .map(|(index, device_info)| {
                let name = device_info.human_name().to_string();
                let description = device_info.description().to_string();
                
                debug!("发现摄像头设备 {}: {} ({})", index, name, description);
                
                CameraInfo {
                    index,
                    name,
                    description,
                }
            })
            .collect();
        
        info!("成功枚举到 {} 个摄像头设备", self.devices.len());
        Ok(self.devices.clone())
    }

    /// 打开指定索引的摄像头设备
    pub fn open_device(&mut self, index: usize) -> Result<(), CameraError> {
        info!("尝试打开摄像头设备 {}", index);
        
        // 检查索引是否有效
        if index >= self.devices.len() {
            error!("无效的摄像头设备索引: {}", index);
            return Err(CameraError::NoDeviceFound);
        }
        
        // 如果当前有设备打开，先关闭它
        if self.camera.is_some() {
            self.close_device()?;
        }
        
        // 创建摄像头索引
        let camera_index = CameraIndex::Index(index as u32);
        
        // 设置请求的格式 - 优先使用 640x480 @ 30 FPS
        let requested_format = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate
        );
        
        // 尝试打开摄像头
        let camera = CallbackCamera::new(camera_index, requested_format, |_| {})
            .map_err(|e| {
                error!("打开摄像头设备 {} 失败: {}", index, e);
                Self::map_nokhwa_error(e)
            })?;
        
        self.camera = Some(camera);
        self.current_device_index = Some(index);
        self.is_capturing = false;
        self.retry_count = 0; // 重置重试计数器
        
        info!("成功打开摄像头设备 {}: {}", index, self.devices[index].name);
        Ok(())
    }

    /// 关闭当前摄像头设备
    pub fn close_device(&mut self) -> Result<(), CameraError> {
        if let Some(mut camera) = self.camera.take() {
            info!("关闭摄像头设备");
            
            // 停止视频流
            if self.is_capturing {
                if let Err(e) = camera.stop_stream() {
                    warn!("停止视频流时出现警告: {}", e);
                }
                self.is_capturing = false;
            }
            
            // 摄像头会在 drop 时自动释放资源
            self.current_device_index = None;
            self.retry_count = 0; // 重置重试计数器
            info!("摄像头设备已关闭");
        }
        
        Ok(())
    }

    /// 捕获一帧视频
    pub fn capture_frame(&mut self) -> Result<Frame, CameraError> {
        let camera = self.camera.as_mut()
            .ok_or_else(|| {
                error!("尝试捕获帧时没有打开的摄像头设备");
                CameraError::CaptureError("没有打开的摄像头设备".to_string())
            })?;
        
        // 如果还没有开始捕获，先开始视频流
        if !self.is_capturing {
            camera.open_stream()
                .map_err(|e| {
                    error!("开始视频流失败: {}", e);
                    CameraError::CaptureError(format!("开始视频流失败: {}", e))
                })?;
            self.is_capturing = true;
            debug!("视频流已开始");
        }
        
        // 尝试捕获一帧，带重试机制
        self.capture_frame_with_retry()
    }
    
    /// 带重试机制的帧捕获
    fn capture_frame_with_retry(&mut self) -> Result<Frame, CameraError> {
        let camera = self.camera.as_mut().unwrap(); // 已在上层检查过
        
        for attempt in 0..=self.max_retries {
            match camera.frame() {
                Ok(frame) => {
                    // 捕获成功，重置重试计数器
                    if self.retry_count > 0 {
                        info!("摄像头捕获恢复正常，重试次数: {}", self.retry_count);
                        self.retry_count = 0;
                    }
                    
                    // 转换为我们的 Frame 格式
                    let width = frame.width();
                    let height = frame.height();
                    let data = frame.into_raw();
                    
                    debug!("成功捕获帧: {}x{}, {} 字节", width, height, data.len());
                    
                    return Ok(Frame {
                        data,
                        width,
                        height,
                        format: PixelFormat::RGB8, // nokhwa 默认返回 RGB8 格式
                    });
                }
                Err(e) => {
                    self.retry_count += 1;
                    
                    if attempt < self.max_retries {
                        warn!("捕获视频帧失败 (尝试 {}/{}): {}", attempt + 1, self.max_retries + 1, e);
                        
                        // 短暂等待后重试
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        
                        // 尝试重新启动视频流
                        if let Err(restart_err) = self.restart_video_stream() {
                            warn!("重启视频流失败: {}", restart_err);
                        }
                    } else {
                        error!("捕获视频帧失败，已达到最大重试次数 ({}): {}", self.max_retries, e);
                        return Err(CameraError::CaptureError(format!(
                            "捕获帧失败，已重试 {} 次: {}", 
                            self.max_retries, 
                            e
                        )));
                    }
                }
            }
        }
        
        // 理论上不会到达这里
        Err(CameraError::CaptureError("未知的捕获错误".to_string()))
    }
    
    /// 将 nokhwa 错误映射到 CameraError
    fn map_nokhwa_error(error: nokhwa::NokhwaError) -> CameraError {
        match error {
            // 设备被占用的情况
            nokhwa::NokhwaError::UnsupportedOperationError(_) => {
                warn!("摄像头设备不支持请求的操作，可能被其他应用占用");
                CameraError::DeviceInUse
            }
            nokhwa::NokhwaError::OpenDeviceError(msg) => {
                if msg.to_lowercase().contains("permission") || 
                   msg.to_lowercase().contains("access") ||
                   msg.to_lowercase().contains("denied") {
                    warn!("摄像头访问权限被拒绝: {}", msg);
                    CameraError::PermissionDenied
                } else if msg.to_lowercase().contains("busy") ||
                         msg.to_lowercase().contains("in use") ||
                         msg.to_lowercase().contains("occupied") {
                    warn!("摄像头设备被占用: {}", msg);
                    CameraError::DeviceInUse
                } else {
                    warn!("打开摄像头设备失败: {}", msg);
                    CameraError::CaptureError(format!("打开设备失败: {}", msg))
                }
            }
            // 权限相关错误
            nokhwa::NokhwaError::GetPropertyError(_) => {
                warn!("无法获取摄像头属性，可能是权限问题");
                CameraError::PermissionDenied
            }
            nokhwa::NokhwaError::SetPropertyError(_) => {
                warn!("无法设置摄像头属性，可能是权限问题");
                CameraError::PermissionDenied
            }
            // 结构化错误
            nokhwa::NokhwaError::StructureError(msg) => {
                error!("摄像头结构错误: {}", msg);
                CameraError::CaptureError(format!("设备结构错误: {}", msg))
            }
            // 读取错误
            nokhwa::NokhwaError::ReadFrameError(msg) => {
                warn!("读取帧错误: {}", msg);
                CameraError::CaptureError(format!("读取帧失败: {}", msg))
            }
            // 处理错误
            nokhwa::NokhwaError::ProcessFrameError(msg) => {
                warn!("处理帧错误: {}", msg);
                CameraError::CaptureError(format!("处理帧失败: {}", msg))
            }
            // 通用错误
            nokhwa::NokhwaError::GeneralError(msg) => {
                error!("摄像头通用错误: {}", msg);
                CameraError::CaptureError(format!("通用错误: {}", msg))
            }
            // 其他未知错误
            _ => {
                error!("未知的摄像头错误: {}", error);
                CameraError::CaptureError(format!("未知错误: {}", error))
            }
        }
    }

    /// 检查设备权限
    pub fn check_device_permissions(&self) -> Result<(), CameraError> {
        debug!("检查摄像头设备权限");
        
        // 尝试枚举设备来检查基本权限
        match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
            Ok(devices) => {
                if devices.is_empty() {
                    warn!("权限检查：未找到任何摄像头设备");
                    Err(CameraError::NoDeviceFound)
                } else {
                    debug!("权限检查通过，找到 {} 个设备", devices.len());
                    Ok(())
                }
            }
            Err(e) => {
                error!("权限检查失败: {}", e);
                Err(Self::map_nokhwa_error(e))
            }
        }
    }

    /// 检测设备是否被占用
    pub fn is_device_in_use(&self, index: usize) -> bool {
        if index >= self.devices.len() {
            return false;
        }
        
        debug!("检测设备 {} 是否被占用", index);
        
        // 尝试创建一个临时摄像头实例来检测占用状态
        let camera_index = CameraIndex::Index(index as u32);
        let requested_format = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate
        );
        
        match CallbackCamera::new(camera_index, requested_format, |_| {}) {
            Ok(_) => {
                debug!("设备 {} 可用", index);
                false
            }
            Err(e) => {
                debug!("设备 {} 检测结果: {}", index, e);
                matches!(Self::map_nokhwa_error(e), CameraError::DeviceInUse)
            }
        }
    }
    
    /// 重启视频流
    fn restart_video_stream(&mut self) -> Result<(), CameraError> {
        if let Some(camera) = &mut self.camera {
            debug!("尝试重启视频流");
            
            // 停止当前流
            if let Err(e) = camera.stop_stream() {
                warn!("停止视频流时出现警告: {}", e);
            }
            
            // 重新开始流
            camera.open_stream()
                .map_err(|e| {
                    error!("重启视频流失败: {}", e);
                    CameraError::CaptureError(format!("重启视频流失败: {}", e))
                })?;
            
            debug!("视频流重启成功");
            Ok(())
        } else {
            Err(CameraError::CaptureError("没有可用的摄像头设备".to_string()))
        }
    }

    /// 获取当前设备信息
    pub fn current_device(&self) -> Option<&CameraInfo> {
        self.current_device_index
            .and_then(|index| self.devices.get(index))
    }
    
    /// 获取当前设备索引
    pub fn current_device_index(&self) -> Option<usize> {
        self.current_device_index
    }
    
    /// 检查是否正在捕获
    pub fn is_capturing(&self) -> bool {
        self.is_capturing
    }
    
    /// 获取所有设备列表
    pub fn devices(&self) -> &[CameraInfo] {
        &self.devices
    }
    
    /// 获取当前重试次数
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }
    
    /// 获取最大重试次数
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
    
    /// 设置最大重试次数
    pub fn set_max_retries(&mut self, max_retries: u32) {
        self.max_retries = max_retries;
        info!("设置最大重试次数为: {}", max_retries);
    }
}

impl Drop for CameraManager {
    fn drop(&mut self) {
        if let Err(e) = self.close_device() {
            error!("清理摄像头管理器时出错: {}", e);
        }
        debug!("摄像头管理器已清理");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_manager_creation() {
        // 注意：这个测试可能会失败如果系统没有摄像头设备
        // 在 CI 环境中，我们可能需要使用模拟设备
        let result = CameraManager::new();
        
        // 如果系统有摄像头设备，应该成功创建
        // 如果没有设备，应该返回 NoDeviceFound 错误
        match result {
            Ok(manager) => {
                assert!(!manager.devices().is_empty());
            }
            Err(CameraError::NoDeviceFound) => {
                // 在没有摄像头的环境中这是预期的
                println!("警告: 系统中没有找到摄像头设备");
            }
            Err(e) => {
                panic!("创建摄像头管理器时出现意外错误: {}", e);
            }
        }
    }

    #[test]
    fn test_camera_info_structure() {
        let info = CameraInfo {
            index: 0,
            name: "Test Camera".to_string(),
            description: "Test Description".to_string(),
        };
        
        assert_eq!(info.index, 0);
        assert_eq!(info.name, "Test Camera");
        assert_eq!(info.description, "Test Description");
    }

    #[test]
    fn test_frame_structure() {
        let frame = Frame {
            data: vec![255, 0, 0, 0, 255, 0, 0, 0, 255], // 3x1 RGB 像素
            width: 3,
            height: 1,
            format: PixelFormat::RGB8,
        };
        
        assert_eq!(frame.width, 3);
        assert_eq!(frame.height, 1);
        assert_eq!(frame.format, PixelFormat::RGB8);
        assert_eq!(frame.data.len(), 9); // 3 像素 * 3 字节/像素
    }

    #[test]
    fn test_pixel_format_variants() {
        // 测试所有像素格式变体
        let formats = [PixelFormat::RGB8, PixelFormat::RGBA8, PixelFormat::YUV420];
        
        for format in formats {
            let frame = Frame {
                data: vec![0; 100],
                width: 10,
                height: 10,
                format,
            };
            assert_eq!(frame.format, format);
        }
    }

    #[test]
    fn test_invalid_device_index() {
        // 创建一个空的摄像头管理器用于测试
        let mut manager = CameraManager {
            camera: None,
            devices: vec![],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        // 尝试打开不存在的设备应该返回错误
        let result = manager.open_device(999);
        assert!(matches!(result, Err(CameraError::NoDeviceFound)));
    }

    #[test]
    fn test_capture_without_device() {
        // 创建一个没有打开设备的摄像头管理器
        let mut manager = CameraManager {
            camera: None,
            devices: vec![],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        // 尝试捕获帧应该返回错误
        let result = manager.capture_frame();
        assert!(matches!(result, Err(CameraError::CaptureError(_))));
    }

    #[test]
    fn test_close_device_without_open() {
        // 创建一个没有打开设备的摄像头管理器
        let mut manager = CameraManager {
            camera: None,
            devices: vec![],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        // 关闭不存在的设备应该成功（无操作）
        let result = manager.close_device();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_retry_mechanism() {
        // 测试重试机制的配置
        let mut manager = CameraManager {
            camera: None,
            devices: vec![],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        assert_eq!(manager.retry_count(), 0);
        assert_eq!(manager.max_retries(), 3);
        
        // 测试设置最大重试次数
        manager.set_max_retries(5);
        assert_eq!(manager.max_retries(), 5);
    }
    
    #[test]
    fn test_error_mapping() {
        // 测试 nokhwa 错误映射
        use nokhwa::NokhwaError;
        
        // 测试设备被占用错误
        let error = NokhwaError::UnsupportedOperationError("Device busy".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::DeviceInUse));
        
        // 测试权限拒绝错误
        let error = NokhwaError::GetPropertyError("Permission denied".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::PermissionDenied));
        
        // 测试通用错误
        let error = NokhwaError::GeneralError("General error".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::CaptureError(_)));
    }
}
