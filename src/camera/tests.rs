// 摄像头管理器集成测试

use super::*;
use crate::error::CameraError;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 测试摄像头管理器的完整工作流程
    /// 注意：这个测试需要系统有可用的摄像头设备
    #[test]
    fn test_camera_manager_workflow() {
        // 创建摄像头管理器
        let mut manager = match CameraManager::new() {
            Ok(manager) => manager,
            Err(CameraError::NoDeviceFound) => {
                println!("跳过测试：系统中没有摄像头设备");
                return;
            }
            Err(e) => panic!("创建摄像头管理器失败: {}", e),
        };

        // 验证设备列表不为空
        let devices = manager.devices();
        assert!(!devices.is_empty(), "应该至少有一个摄像头设备");

        // 验证设备信息的完整性
        for device in devices {
            assert!(!device.name.is_empty(), "设备名称不应为空");
            assert!(!device.description.is_empty(), "设备描述不应为空");
        }

        // 测试打开第一个设备
        let first_device_index = 0;
        assert!(manager.open_device(first_device_index).is_ok(), "应该能够打开第一个设备");

        // 验证当前设备信息
        let current_device = manager.current_device();
        assert!(current_device.is_some(), "应该有当前设备");
        assert_eq!(current_device.unwrap().index, first_device_index);

        // 验证设备状态
        assert_eq!(manager.current_device_index(), Some(first_device_index));
        assert!(!manager.is_capturing(), "初始状态不应该在捕获");

        // 测试捕获帧（这会自动开始视频流）
        match manager.capture_frame() {
            Ok(frame) => {
                assert!(frame.width > 0, "帧宽度应该大于0");
                assert!(frame.height > 0, "帧高度应该大于0");
                assert!(!frame.data.is_empty(), "帧数据不应为空");
                assert_eq!(frame.format, PixelFormat::RGB8, "应该是RGB8格式");
                
                // 验证数据大小是否合理（RGB8 格式应该是 width * height * 3）
                let expected_size = (frame.width * frame.height * 3) as usize;
                assert_eq!(frame.data.len(), expected_size, "帧数据大小应该匹配");
                
                // 验证现在应该在捕获状态
                assert!(manager.is_capturing(), "捕获帧后应该处于捕获状态");
            }
            Err(e) => {
                println!("警告：捕获帧失败，可能是权限问题: {}", e);
            }
        }

        // 测试关闭设备
        assert!(manager.close_device().is_ok(), "应该能够关闭设备");
        assert!(manager.current_device().is_none(), "关闭后不应该有当前设备");
        assert!(!manager.is_capturing(), "关闭后不应该在捕获状态");
    }

    #[test]
    fn test_device_switching() {
        let mut manager = match CameraManager::new() {
            Ok(manager) => manager,
            Err(CameraError::NoDeviceFound) => {
                println!("跳过测试：系统中没有摄像头设备");
                return;
            }
            Err(e) => panic!("创建摄像头管理器失败: {}", e),
        };

        let devices = manager.devices();
        if devices.len() < 2 {
            println!("跳过测试：需要至少2个摄像头设备进行切换测试");
            return;
        }

        // 打开第一个设备
        assert!(manager.open_device(0).is_ok());
        assert_eq!(manager.current_device_index(), Some(0));

        // 切换到第二个设备
        assert!(manager.open_device(1).is_ok());
        assert_eq!(manager.current_device_index(), Some(1));

        // 验证设备信息已更新
        let current_device = manager.current_device().unwrap();
        assert_eq!(current_device.index, 1);
    }

    #[test]
    fn test_error_handling() {
        let mut manager = match CameraManager::new() {
            Ok(manager) => manager,
            Err(CameraError::NoDeviceFound) => {
                // 如果没有设备，测试这种情况的处理
                println!("测试无设备情况的错误处理");
                return;
            }
            Err(e) => panic!("创建摄像头管理器失败: {}", e),
        };

        // 测试打开无效索引的设备
        let invalid_index = 9999;
        let result = manager.open_device(invalid_index);
        assert!(matches!(result, Err(CameraError::NoDeviceFound)));

        // 测试在没有打开设备时捕获帧
        let mut empty_manager = CameraManager {
            camera: None,
            devices: vec![],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        let result = empty_manager.capture_frame();
        assert!(matches!(result, Err(CameraError::CaptureError(_))));
    }

    #[test]
    fn test_multiple_enumerate_calls() {
        let mut manager = match CameraManager::new() {
            Ok(manager) => manager,
            Err(CameraError::NoDeviceFound) => {
                println!("跳过测试：系统中没有摄像头设备");
                return;
            }
            Err(e) => panic!("创建摄像头管理器失败: {}", e),
        };

        let devices1 = manager.enumerate_devices().unwrap();
        let devices2 = manager.enumerate_devices().unwrap();

        // 多次枚举应该返回相同的结果
        assert_eq!(devices1.len(), devices2.len());
        for (d1, d2) in devices1.iter().zip(devices2.iter()) {
            assert_eq!(d1.index, d2.index);
            assert_eq!(d1.name, d2.name);
            assert_eq!(d1.description, d2.description);
        }
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    /// 创建模拟的摄像头管理器用于测试
    fn create_mock_manager() -> CameraManager {
        CameraManager {
            camera: None,
            devices: vec![
                CameraInfo {
                    index: 0,
                    name: "Mock Camera 1".to_string(),
                    description: "Mock Description 1".to_string(),
                },
                CameraInfo {
                    index: 1,
                    name: "Mock Camera 2".to_string(),
                    description: "Mock Description 2".to_string(),
                },
            ],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        }
    }

    #[test]
    fn test_mock_device_info() {
        let manager = create_mock_manager();
        let devices = manager.devices();
        
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].name, "Mock Camera 1");
        assert_eq!(devices[1].name, "Mock Camera 2");
    }

    #[test]
    fn test_mock_current_device() {
        let mut manager = create_mock_manager();
        
        // 初始状态没有当前设备
        assert!(manager.current_device().is_none());
        assert_eq!(manager.current_device_index(), None);
        
        // 模拟设置当前设备
        manager.current_device_index = Some(0);
        let current = manager.current_device();
        assert!(current.is_some());
        assert_eq!(current.unwrap().index, 0);
    }

    #[test]
    fn test_mock_capturing_state() {
        let mut manager = create_mock_manager();
        
        // 初始状态不在捕获
        assert!(!manager.is_capturing());
        
        // 模拟设置捕获状态
        manager.is_capturing = true;
        assert!(manager.is_capturing());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_retry_mechanism_configuration() {
        let mut manager = CameraManager {
            camera: None,
            devices: vec![],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        // 测试初始状态
        assert_eq!(manager.retry_count(), 0);
        assert_eq!(manager.max_retries(), 3);
        
        // 测试设置最大重试次数
        manager.set_max_retries(5);
        assert_eq!(manager.max_retries(), 5);
        
        // 测试设置为0（禁用重试）
        manager.set_max_retries(0);
        assert_eq!(manager.max_retries(), 0);
    }

    #[test]
    fn test_nokhwa_error_mapping() {
        use nokhwa::NokhwaError;
        
        // 测试设备被占用错误
        let error = NokhwaError::UnsupportedOperationError(nokhwa::utils::ApiBackend::Auto);
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::DeviceInUse));
        
        // 测试权限拒绝错误
        let error = NokhwaError::GetPropertyError { 
            property: "test".to_string(), 
            error: "Permission denied".to_string() 
        };
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::PermissionDenied));
        
        // 测试设置属性权限错误
        let error = NokhwaError::SetPropertyError { 
            property: "test".to_string(), 
            value: "test".to_string(), 
            error: "Access denied".to_string() 
        };
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::PermissionDenied));
        
        // 测试打开设备错误 - 权限相关
        let error = NokhwaError::OpenDeviceError("Permission denied".to_string(), "test".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::PermissionDenied));
        
        // 测试打开设备错误 - 设备被占用
        let error = NokhwaError::OpenDeviceError("Device is busy".to_string(), "test".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::DeviceInUse));
        
        // 测试打开设备错误 - 其他错误
        let error = NokhwaError::OpenDeviceError("Unknown error".to_string(), "test".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::CaptureError(_)));
        
        // 测试结构化错误
        let error = NokhwaError::StructureError { 
            structure: "test".to_string(), 
            error: "Invalid structure".to_string() 
        };
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::CaptureError(_)));
        
        // 测试读取帧错误
        let error = NokhwaError::ReadFrameError("Read failed".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::CaptureError(_)));
        
        // 测试处理帧错误
        let error = NokhwaError::ProcessFrameError { 
            src: nokhwa::utils::FrameFormat::MJPEG, 
            destination: "RGB".to_string(), 
            error: "Process failed".to_string() 
        };
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::CaptureError(_)));
        
        // 测试通用错误
        let error = NokhwaError::GeneralError("General error".to_string());
        let mapped = CameraManager::map_nokhwa_error(error);
        assert!(matches!(mapped, CameraError::CaptureError(_)));
    }
}
        assert!(matches!(mapped, CameraError::CaptureError(_)));
    }

    #[test]
    fn test_permission_check() {
        // 注意：这个测试的结果取决于系统环境
        let manager = CameraManager {
            camera: None,
            devices: vec![],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        let result = manager.check_device_permissions();
        
        // 结果应该是 Ok(()) 或者是一个明确的错误类型
        match result {
            Ok(()) => {
                println!("权限检查通过");
            }
            Err(CameraError::NoDeviceFound) => {
                println!("权限检查：未找到设备");
            }
            Err(CameraError::PermissionDenied) => {
                println!("权限检查：权限被拒绝");
            }
            Err(e) => {
                println!("权限检查：其他错误 - {}", e);
            }
        }
    }

    #[test]
    fn test_device_in_use_detection() {
        let manager = CameraManager {
            camera: None,
            devices: vec![
                CameraInfo {
                    index: 0,
                    name: "Test Camera".to_string(),
                    description: "Test Description".to_string(),
                },
            ],
            current_device_index: None,
            is_capturing: false,
            retry_count: 0,
            max_retries: 3,
        };
        
        // 测试有效索引
        let result = manager.is_device_in_use(0);
        // 结果取决于系统状态，但不应该panic
        println!("设备 0 占用状态: {}", result);
        
        // 测试无效索引
        let result = manager.is_device_in_use(999);
        assert!(!result, "无效索引应该返回false");
    }

    #[test]
    fn test_error_display_messages() {
        // 测试所有错误类型的显示消息
        let errors = vec![
            CameraError::NoDeviceFound,
            CameraError::DeviceInUse,
            CameraError::PermissionDenied,
            CameraError::CaptureError("Test error".to_string()),
        ];
        
        for error in errors {
            let message = error.to_string();
            assert!(!message.is_empty(), "错误消息不应为空");
            println!("错误消息: {}", message);
        }
    }
}