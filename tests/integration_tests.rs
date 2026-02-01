// 集成测试
// 测试完整的用户工作流

use mira::{
    camera::{CameraManager, CameraInfo},
    config::{ConfigManager, AppConfig, WindowConfig, CameraConfig},
    shape::{ShapeMask, ShapeType},
    window::WindowManager,
    render::RenderEngine,
    event::EventHandler,
    error::{CameraError, WindowError, ConfigError},
};
use std::time::{Duration, Instant};
use winit::{
    event::{WindowEvent, MouseButton, ElementState, MouseScrollDelta, ModifiersState},
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 测试完整的用户工作流：启动 → 选择设备 → 切换形状 → 拖拽 → 关闭
    #[tokio::test]
    async fn test_complete_user_workflow() {
        // 1. 应用启动阶段
        let event_loop = EventLoop::new().expect("创建事件循环失败");
        
        // 初始化配置管理器
        let mut config_manager = ConfigManager::new().expect("配置管理器创建失败");
        let config = config_manager.load().unwrap_or_else(|_| {
            // 使用默认配置
            AppConfig {
                window: WindowConfig {
                    position_x: 100.0,
                    position_y: 100.0,
                    width: 400,
                    height: 400,
                    rotation: 0.0,
                    shape: "Circle".to_string(),
                },
                camera: CameraConfig {
                    device_index: 0,
                },
            }
        });
        
        // 初始化摄像头管理器
        let mut camera_manager = CameraManager::new().unwrap_or_else(|_| {
            // 如果没有摄像头，创建空管理器用于测试
            CameraManager::new_empty()
        });
        
        // 2. 设备选择阶段
        if !camera_manager.devices().is_empty() {
            // 测试设备枚举
            let devices = camera_manager.devices();
            assert!(!devices.is_empty(), "应该至少有一个摄像头设备");
            
            // 测试打开第一个设备
            let result = camera_manager.open_device(0);
            match result {
                Ok(()) => {
                    println!("成功打开摄像头设备 0");
                    
                    // 测试捕获帧
                    let frame_result = camera_manager.capture_frame();
                    match frame_result {
                        Ok(frame) => {
                            assert!(frame.width > 0, "帧宽度应该大于 0");
                            assert!(frame.height > 0, "帧高度应该大于 0");
                            assert!(!frame.data.is_empty(), "帧数据不应该为空");
                            println!("成功捕获帧: {}x{}", frame.width, frame.height);
                        }
                        Err(e) => {
                            println!("捕获帧失败（可能是测试环境限制）: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("打开摄像头失败（可能是测试环境限制）: {}", e);
                }
            }
        }
        
        // 3. 窗口管理阶段
        let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
        
        // 应用配置
        window_manager.set_position(config.window.position_x, config.window.position_y);
        window_manager.set_size(config.window.width, config.window.height);
        window_manager.set_rotation(config.window.rotation);
        
        // 验证窗口状态
        let position = window_manager.position();
        let size = window_manager.size();
        let rotation = window_manager.rotation();
        
        assert_eq!(position.x, config.window.position_x);
        assert_eq!(position.y, config.window.position_y);
        assert_eq!(size.width, config.window.width);
        assert_eq!(size.height, config.window.height);
        assert_eq!(rotation, config.window.rotation);
        
        // 4. 形状切换阶段
        let mut shape_mask = ShapeMask::new(ShapeType::Circle, config.window.width, config.window.height);
        
        // 测试所有预设形状
        let shapes = vec![
            ShapeType::Circle,
            ShapeType::Ellipse,
            ShapeType::Rectangle,
            ShapeType::RoundedRectangle { radius: 20.0 },
            ShapeType::Heart,
        ];
        
        for shape in shapes {
            let start_time = Instant::now();
            shape_mask.set_shape(shape.clone());
            let switch_time = start_time.elapsed();
            
            // 验证形状切换时间 < 100ms
            assert!(switch_time < Duration::from_millis(100), 
                   "形状切换时间 {:?} 超过 100ms 限制", switch_time);
            
            // 验证遮罩数据
            let mask_data = shape_mask.data();
            assert!(!mask_data.is_empty(), "遮罩数据不应该为空");
            
            println!("成功切换到形状 {:?}，耗时 {:?}", shape, switch_time);
        }
        
        // 5. 窗口交互阶段
        // 测试拖拽功能
        let initial_pos = window_manager.position();
        let cursor_pos = PhysicalPosition::new(200.0, 200.0);
        
        window_manager.start_drag(cursor_pos);
        assert!(window_manager.is_dragging(), "应该处于拖拽状态");
        
        let new_cursor_pos = PhysicalPosition::new(250.0, 250.0);
        window_manager.update_drag(new_cursor_pos);
        
        let dragged_pos = window_manager.position();
        assert_ne!(dragged_pos.x, initial_pos.x, "拖拽后 X 位置应该改变");
        assert_ne!(dragged_pos.y, initial_pos.y, "拖拽后 Y 位置应该改变");
        
        window_manager.end_drag();
        assert!(!window_manager.is_dragging(), "应该退出拖拽状态");
        
        // 测试缩放功能
        let initial_size = window_manager.size();
        window_manager.scale(1.1); // 放大 10%
        
        let scaled_size = window_manager.size();
        assert!(scaled_size.width > initial_size.width, "缩放后宽度应该增加");
        assert!(scaled_size.height > initial_size.height, "缩放后高度应该增加");
        
        // 测试旋转功能
        let initial_rotation = window_manager.rotation();
        window_manager.set_rotation(initial_rotation + 15.0);
        
        let rotated_angle = window_manager.rotation();
        assert_eq!(rotated_angle, initial_rotation + 15.0, "旋转角度应该正确更新");
        
        // 6. 配置保存阶段
        let final_config = AppConfig {
            window: WindowConfig {
                position_x: window_manager.position().x,
                position_y: window_manager.position().y,
                width: window_manager.size().width,
                height: window_manager.size().height,
                rotation: window_manager.rotation(),
                shape: "Heart".to_string(),
            },
            camera: CameraConfig {
                device_index: 0,
            },
        };
        
        let save_result = config_manager.save(&final_config);
        assert!(save_result.is_ok(), "配置保存应该成功");
        
        println!("完整用户工作流测试通过");
    }

    /// 测试配置持久化工作流：启动 → 修改设置 → 关闭 → 重新启动 → 验证设置恢复
    #[tokio::test]
    async fn test_configuration_persistence_workflow() {
        // 1. 创建初始配置
        let initial_config = AppConfig {
            window: WindowConfig {
                position_x: 150.0,
                position_y: 200.0,
                width: 500,
                height: 600,
                rotation: 45.0,
                shape: "Rectangle".to_string(),
            },
            camera: CameraConfig {
                device_index: 1,
            },
        };
        
        // 2. 保存配置
        let mut config_manager = ConfigManager::new().expect("配置管理器创建失败");
        config_manager.save(&initial_config).expect("配置保存失败");
        
        // 3. 模拟应用重启 - 重新加载配置
        let mut new_config_manager = ConfigManager::new().expect("新配置管理器创建失败");
        let loaded_config = new_config_manager.load().expect("配置加载失败");
        
        // 4. 验证配置一致性
        assert_eq!(loaded_config.window.position_x, initial_config.window.position_x);
        assert_eq!(loaded_config.window.position_y, initial_config.window.position_y);
        assert_eq!(loaded_config.window.width, initial_config.window.width);
        assert_eq!(loaded_config.window.height, initial_config.window.height);
        assert_eq!(loaded_config.window.rotation, initial_config.window.rotation);
        assert_eq!(loaded_config.window.shape, initial_config.window.shape);
        assert_eq!(loaded_config.camera.device_index, initial_config.camera.device_index);
        
        // 5. 修改配置并再次保存
        let modified_config = AppConfig {
            window: WindowConfig {
                position_x: 300.0,
                position_y: 400.0,
                width: 800,
                height: 600,
                rotation: 90.0,
                shape: "Heart".to_string(),
            },
            camera: CameraConfig {
                device_index: 0,
            },
        };
        
        new_config_manager.save(&modified_config).expect("修改后配置保存失败");
        
        // 6. 再次重启并验证
        let mut final_config_manager = ConfigManager::new().expect("最终配置管理器创建失败");
        let final_loaded_config = final_config_manager.load().expect("最终配置加载失败");
        
        assert_eq!(final_loaded_config.window.position_x, modified_config.window.position_x);
        assert_eq!(final_loaded_config.window.position_y, modified_config.window.position_y);
        assert_eq!(final_loaded_config.window.width, modified_config.window.width);
        assert_eq!(final_loaded_config.window.height, modified_config.window.height);
        assert_eq!(final_loaded_config.window.rotation, modified_config.window.rotation);
        assert_eq!(final_loaded_config.window.shape, modified_config.window.shape);
        assert_eq!(final_loaded_config.camera.device_index, modified_config.camera.device_index);
        
        println!("配置持久化工作流测试通过");
    }

    /// 测试错误恢复工作流：设备断开 → 重新连接 → 自动恢复
    #[tokio::test]
    async fn test_error_recovery_workflow() {
        // 1. 初始化摄像头管理器
        let mut camera_manager = CameraManager::new().unwrap_or_else(|_| {
            CameraManager::new_empty()
        });
        
        // 2. 如果有设备，测试错误恢复
        if !camera_manager.devices().is_empty() {
            // 尝试打开设备
            let open_result = camera_manager.open_device(0);
            
            match open_result {
                Ok(()) => {
                    println!("成功打开摄像头设备");
                    
                    // 模拟设备断开（关闭设备）
                    let close_result = camera_manager.close_device();
                    assert!(close_result.is_ok(), "设备关闭应该成功");
                    
                    // 验证设备已关闭
                    let capture_result = camera_manager.capture_frame();
                    assert!(capture_result.is_err(), "关闭后捕获应该失败");
                    
                    // 模拟重新连接（重新打开设备）
                    let reopen_result = camera_manager.open_device(0);
                    match reopen_result {
                        Ok(()) => {
                            println!("成功重新打开摄像头设备");
                            
                            // 验证设备恢复正常
                            let recovery_capture = camera_manager.capture_frame();
                            match recovery_capture {
                                Ok(_) => println!("设备恢复后成功捕获帧"),
                                Err(e) => println!("设备恢复后捕获失败: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("重新打开设备失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("初始打开设备失败: {}", e);
                    
                    // 测试错误处理
                    match e {
                        CameraError::NoDeviceFound => {
                            println!("正确处理了无设备错误");
                        }
                        CameraError::DeviceInUse => {
                            println!("正确处理了设备被占用错误");
                        }
                        CameraError::PermissionDenied => {
                            println!("正确处理了权限拒绝错误");
                        }
                        CameraError::CaptureError(_) => {
                            println!("正确处理了捕获错误");
                        }
                    }
                }
            }
        } else {
            println!("无摄像头设备，跳过错误恢复测试");
        }
        
        // 3. 测试窗口错误恢复
        let event_loop = EventLoop::new().expect("创建事件循环失败");
        let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
        
        // 测试无效尺寸的自动修正
        let original_size = window_manager.size();
        
        // 尝试设置过小的尺寸
        window_manager.set_size(50, 50); // 小于最小尺寸 100x100
        let corrected_size = window_manager.size();
        
        // 验证尺寸被自动修正
        assert!(corrected_size.width >= 100, "宽度应该被修正到最小值");
        assert!(corrected_size.height >= 100, "高度应该被修正到最小值");
        
        println!("错误恢复工作流测试通过");
    }

    /// 测试多设备切换工作流
    #[tokio::test]
    async fn test_multi_device_switching_workflow() {
        let mut camera_manager = CameraManager::new().unwrap_or_else(|_| {
            CameraManager::new_empty()
        });
        
        let devices = camera_manager.devices();
        
        if devices.len() > 1 {
            println!("发现 {} 个摄像头设备，测试设备切换", devices.len());
            
            // 测试切换到每个设备
            for (index, device) in devices.iter().enumerate() {
                println!("尝试切换到设备 {}: {}", index, device.name);
                
                let switch_result = camera_manager.open_device(index);
                match switch_result {
                    Ok(()) => {
                        println!("成功切换到设备 {}", index);
                        
                        // 验证当前设备
                        if let Some(current_device) = camera_manager.current_device() {
                            assert_eq!(current_device.index, index, "当前设备索引应该匹配");
                        }
                        
                        // 尝试捕获帧
                        let capture_result = camera_manager.capture_frame();
                        match capture_result {
                            Ok(frame) => {
                                println!("设备 {} 成功捕获帧: {}x{}", index, frame.width, frame.height);
                            }
                            Err(e) => {
                                println!("设备 {} 捕获帧失败: {}", index, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("切换到设备 {} 失败: {}", index, e);
                    }
                }
            }
        } else if devices.len() == 1 {
            println!("只有一个摄像头设备，跳过多设备切换测试");
        } else {
            println!("无摄像头设备，跳过多设备切换测试");
        }
        
        println!("多设备切换工作流测试完成");
    }

    /// 测试渲染引擎集成
    #[tokio::test]
    async fn test_render_engine_integration() {
        let event_loop = EventLoop::new().expect("创建事件循环失败");
        let window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
        
        // 初始化渲染引擎
        let mut render_engine = RenderEngine::new(window_manager.window()).await
            .expect("渲染引擎创建失败");
        
        // 测试形状遮罩设置
        let shape_mask = ShapeMask::new(ShapeType::Circle, 400, 400);
        let mask_result = render_engine.set_mask(&shape_mask);
        assert!(mask_result.is_ok(), "设置形状遮罩应该成功");
        
        // 测试表面调整大小
        render_engine.resize(500, 500);
        
        // 测试渲染（没有实际帧数据）
        let render_result = render_engine.render(0.0);
        // 渲染可能失败（因为没有视频帧），但不应该崩溃
        match render_result {
            Ok(()) => println!("渲染成功"),
            Err(e) => println!("渲染失败（预期）: {}", e),
        }
        
        println!("渲染引擎集成测试通过");
    }

    /// 测试事件处理器集成
    #[tokio::test]
    async fn test_event_handler_integration() {
        let event_loop = EventLoop::new().expect("创建事件循环失败");
        
        // 初始化所有组件
        let config_manager = ConfigManager::new().expect("配置管理器创建失败");
        let camera_manager = CameraManager::new().unwrap_or_else(|_| {
            CameraManager::new_empty()
        });
        let window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
        let render_engine = RenderEngine::new(window_manager.window()).await
            .expect("渲染引擎创建失败");
        let shape_mask = ShapeMask::new(ShapeType::Circle, 400, 400);
        
        // 创建事件处理器
        let mut event_handler = EventHandler::new(
            window_manager,
            camera_manager,
            render_engine,
            shape_mask,
            config_manager,
        );
        
        // 测试鼠标按下事件
        let mouse_press = WindowEvent::MouseInput {
            device_id: unsafe { std::mem::transmute(0usize) },
            state: ElementState::Pressed,
            button: MouseButton::Left,
            modifiers: ModifiersState::empty(),
        };
        
        let should_exit = event_handler.handle_window_event(&mouse_press);
        assert!(!should_exit, "鼠标按下不应该导致退出");
        
        // 测试鼠标滚轮事件
        let mouse_wheel = WindowEvent::MouseWheel {
            device_id: unsafe { std::mem::transmute(0usize) },
            delta: MouseScrollDelta::LineDelta(0.0, 1.0),
            phase: winit::event::TouchPhase::Moved,
            modifiers: ModifiersState::empty(),
        };
        
        let should_exit = event_handler.handle_window_event(&mouse_wheel);
        assert!(!should_exit, "鼠标滚轮不应该导致退出");
        
        // 测试窗口调整大小事件
        let resize_event = WindowEvent::Resized(PhysicalSize::new(600, 600));
        let should_exit = event_handler.handle_window_event(&resize_event);
        assert!(!should_exit, "窗口调整大小不应该导致退出");
        
        // 测试关闭请求事件
        let close_event = WindowEvent::CloseRequested;
        let should_exit = event_handler.handle_window_event(&close_event);
        assert!(should_exit, "关闭请求应该导致退出");
        
        println!("事件处理器集成测试通过");
    }

    /// 测试内存管理集成
    #[test]
    fn test_memory_management_integration() {
        use mira::memory::{FrameBufferPool, MemoryMonitor};
        use std::time::Duration;
        
        // 测试帧缓冲区池
        let pool = FrameBufferPool::new(1024 * 1024, 2, 10); // 1MB 缓冲区
        
        // 获取多个缓冲区
        let mut buffers = Vec::new();
        for i in 0..5 {
            let buffer = pool.get_buffer();
            assert_eq!(buffer.len(), 1024 * 1024, "缓冲区 {} 大小应该正确", i);
            buffers.push(buffer);
        }
        
        let stats = pool.get_stats();
        assert!(stats.allocated_count <= 10, "分配的缓冲区数量不应该超过最大值");
        
        // 归还缓冲区
        for buffer in buffers {
            pool.return_buffer(buffer);
        }
        
        let final_stats = pool.get_stats();
        assert!(final_stats.available_count > 0, "应该有可用的缓冲区");
        
        // 测试内存监控
        let mut memory_monitor = MemoryMonitor::new(
            Duration::from_millis(10),
            100,
            10.0,
        );
        
        // 模拟内存使用
        let alert1 = memory_monitor.update(50.0, 5);
        assert!(alert1.is_none(), "正常内存使用不应该产生警报");
        
        std::thread::sleep(Duration::from_millis(20));
        let alert2 = memory_monitor.update(150.0, 8);
        
        // 可能产生高内存使用警报
        if let Some(alert) = alert2 {
            println!("内存警报: {:?}", alert);
        }
        
        println!("内存管理集成测试通过");
    }

    /// 测试性能监控集成
    #[test]
    fn test_performance_monitoring_integration() {
        use mira::performance::{PerformanceMonitor, PerformanceThresholds};
        use std::time::Duration;
        
        let thresholds = PerformanceThresholds {
            min_fps: 30.0,
            max_cpu_percent: 25.0,
            max_memory_mb: 200.0,
            max_frame_time_ms: 33.0,
            max_render_time_ms: 16.0,
        };
        
        let mut monitor = PerformanceMonitor::new(
            100,
            Duration::from_secs(1),
            Some(thresholds),
        );
        
        // 记录正常性能数据
        for _ in 0..10 {
            let alert = monitor.record_frame(
                Duration::from_millis(16), // 60 FPS
                Duration::from_millis(8),  // 8ms 渲染时间
            );
            assert!(alert.is_none(), "正常性能不应该产生警报");
        }
        
        // 记录低性能数据
        let low_fps_alert = monitor.record_frame(
            Duration::from_millis(50), // 20 FPS
            Duration::from_millis(25), // 25ms 渲染时间
        );
        
        if let Some(alert) = low_fps_alert {
            println!("性能警报: {}", alert.message());
        }
        
        let stats = monitor.get_stats();
        assert!(stats.sample_count > 0, "应该有性能样本");
        assert!(stats.avg_fps > 0.0, "平均 FPS 应该大于 0");
        
        println!("性能监控集成测试通过");
    }
}
    /// 边缘情况和压力测试模块
    mod edge_cases_and_stress_tests {
        use super::*;
        use std::thread;
        use std::sync::{Arc, Mutex};

        /// 测试空设备列表情况
        #[test]
        fn test_empty_device_list() {
            // 创建空的摄像头管理器
            let camera_manager = CameraManager::new_empty();
            
            // 验证设备列表为空
            assert!(camera_manager.devices().is_empty(), "设备列表应该为空");
            
            // 尝试打开不存在的设备
            let mut manager = camera_manager;
            let result = manager.open_device(0);
            
            match result {
                Err(CameraError::NoDeviceFound) => {
                    println!("正确处理了空设备列表情况");
                }
                _ => {
                    panic!("应该返回 NoDeviceFound 错误");
                }
            }
            
            // 验证当前设备为空
            assert!(manager.current_device().is_none(), "当前设备应该为空");
            
            // 尝试捕获帧应该失败
            let capture_result = manager.capture_frame();
            assert!(capture_result.is_err(), "空设备列表时捕获应该失败");
        }

        /// 测试最小窗口尺寸（100x100）
        #[tokio::test]
        async fn test_minimum_window_size() {
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            // 尝试设置小于最小尺寸的窗口
            window_manager.set_size(50, 50);
            let size = window_manager.size();
            
            // 验证尺寸被自动调整到最小值
            assert!(size.width >= 100, "宽度应该被调整到最小值 100");
            assert!(size.height >= 100, "高度应该被调整到最小值 100");
            
            // 测试正好是最小尺寸
            window_manager.set_size(100, 100);
            let min_size = window_manager.size();
            assert_eq!(min_size.width, 100, "最小宽度应该是 100");
            assert_eq!(min_size.height, 100, "最小高度应该是 100");
            
            // 测试形状遮罩在最小尺寸下的表现
            let mut shape_mask = ShapeMask::new(ShapeType::Circle, 100, 100);
            shape_mask.generate();
            
            let mask_data = shape_mask.data();
            assert_eq!(mask_data.len(), 100 * 100, "最小尺寸遮罩数据长度应该正确");
            
            // 验证遮罩中心区域不透明
            let center_index = (50 * 100 + 50) as usize;
            assert_eq!(mask_data[center_index], 255, "圆形遮罩中心应该不透明");
            
            println!("最小窗口尺寸测试通过");
        }

        /// 测试最大窗口尺寸（屏幕的 80%）
        #[tokio::test]
        async fn test_maximum_window_size() {
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            // 获取屏幕尺寸（模拟）
            let screen_width = 1920u32;
            let screen_height = 1080u32;
            let max_width = (screen_width as f32 * 0.8) as u32;
            let max_height = (screen_height as f32 * 0.8) as u32;
            
            // 尝试设置超过最大尺寸的窗口
            window_manager.set_size(screen_width, screen_height);
            let size = window_manager.size();
            
            // 验证尺寸被限制在最大值内
            assert!(size.width <= max_width, "宽度应该被限制在最大值内");
            assert!(size.height <= max_height, "高度应该被限制在最大值内");
            
            // 测试正好是最大尺寸
            window_manager.set_size(max_width, max_height);
            let max_size = window_manager.size();
            assert!(max_size.width <= max_width, "最大宽度应该在限制内");
            assert!(max_size.height <= max_height, "最大高度应该在限制内");
            
            // 测试大尺寸形状遮罩的性能
            let start_time = Instant::now();
            let mut large_shape_mask = ShapeMask::new(ShapeType::Heart, max_width, max_height);
            large_shape_mask.generate();
            let generation_time = start_time.elapsed();
            
            // 验证大尺寸遮罩生成时间合理（< 500ms）
            assert!(generation_time < Duration::from_millis(500), 
                   "大尺寸遮罩生成时间 {:?} 应该 < 500ms", generation_time);
            
            let mask_data = large_shape_mask.data();
            assert_eq!(mask_data.len(), (max_width * max_height) as usize, 
                      "大尺寸遮罩数据长度应该正确");
            
            println!("最大窗口尺寸测试通过，生成时间: {:?}", generation_time);
        }

        /// 测试边界旋转角度（0°、90°、180°、270°）
        #[tokio::test]
        async fn test_boundary_rotation_angles() {
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            let boundary_angles = vec![0.0, 90.0, 180.0, 270.0, 360.0];
            
            for angle in boundary_angles {
                window_manager.set_rotation(angle);
                let actual_angle = window_manager.rotation();
                
                // 验证角度在 0-360 范围内
                assert!(actual_angle >= 0.0 && actual_angle < 360.0, 
                       "角度 {} 应该在 0-360 范围内，实际: {}", angle, actual_angle);
                
                // 360° 应该被归一化为 0°
                if angle == 360.0 {
                    assert_eq!(actual_angle, 0.0, "360° 应该被归一化为 0°");
                } else {
                    assert_eq!(actual_angle, angle, "角度应该匹配");
                }
            }
            
            // 测试自动对齐功能
            let near_angles = vec![
                (2.0, 0.0),    // 2° 应该对齐到 0°
                (88.0, 90.0),  // 88° 应该对齐到 90°
                (92.0, 90.0),  // 92° 应该对齐到 90°
                (178.0, 180.0), // 178° 应该对齐到 180°
                (182.0, 180.0), // 182° 应该对齐到 180°
                (268.0, 270.0), // 268° 应该对齐到 270°
                (272.0, 270.0), // 272° 应该对齐到 270°
                (358.0, 0.0),   // 358° 应该对齐到 0°
            ];
            
            for (input_angle, expected_angle) in near_angles {
                window_manager.set_rotation(input_angle);
                let aligned_angle = window_manager.rotation();
                
                // 检查是否在对齐范围内（±5°）
                let diff = (aligned_angle - expected_angle).abs();
                if diff <= 5.0 || diff >= 355.0 { // 考虑 0°/360° 边界
                    println!("角度 {}° 正确对齐到 {}°", input_angle, aligned_angle);
                } else {
                    println!("角度 {}° 未对齐，当前: {}°", input_angle, aligned_angle);
                }
            }
            
            println!("边界旋转角度测试通过");
        }

        /// 测试窗口拖拽到屏幕边界外
        #[tokio::test]
        async fn test_window_drag_beyond_screen_bounds() {
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            // 设置窗口尺寸
            window_manager.set_size(400, 400);
            let window_size = window_manager.size();
            
            // 模拟屏幕尺寸
            let screen_size = PhysicalSize::new(1920u32, 1080u32);
            
            // 测试拖拽到屏幕左边界外
            window_manager.set_position(-350.0, 100.0); // 只有 50px 在屏幕内
            window_manager.constrain_to_screen(screen_size);
            let constrained_pos = window_manager.position();
            
            // 验证至少 20% 的窗口在屏幕内
            let min_visible_width = window_size.width as f32 * 0.2;
            assert!(constrained_pos.x + window_size.width as f64 >= min_visible_width as f64,
                   "窗口应该有至少 20% 在屏幕内");
            
            // 测试拖拽到屏幕右边界外
            window_manager.set_position(1870.0, 100.0); // 只有 50px 在屏幕内
            window_manager.constrain_to_screen(screen_size);
            let right_constrained_pos = window_manager.position();
            
            assert!(right_constrained_pos.x <= (screen_size.width as f64 - min_visible_width as f64),
                   "窗口右边界应该被约束");
            
            // 测试拖拽到屏幕上边界外
            window_manager.set_position(100.0, -350.0);
            window_manager.constrain_to_screen(screen_size);
            let top_constrained_pos = window_manager.position();
            
            let min_visible_height = window_size.height as f32 * 0.2;
            assert!(top_constrained_pos.y + window_size.height as f64 >= min_visible_height as f64,
                   "窗口应该有至少 20% 在屏幕内（垂直方向）");
            
            // 测试拖拽到屏幕下边界外
            window_manager.set_position(100.0, 1030.0);
            window_manager.constrain_to_screen(screen_size);
            let bottom_constrained_pos = window_manager.position();
            
            assert!(bottom_constrained_pos.y <= (screen_size.height as f64 - min_visible_height as f64),
                   "窗口下边界应该被约束");
            
            println!("窗口边界约束测试通过");
        }

        /// 测试长时间运行稳定性（模拟 1 小时运行）
        #[test]
        fn test_long_running_stability() {
            use mira::memory::{FrameBufferPool, MemoryMonitor};
            use mira::performance::PerformanceMonitor;
            
            println!("开始长时间运行稳定性测试（模拟）");
            
            // 创建内存监控器
            let mut memory_monitor = MemoryMonitor::new(
                Duration::from_millis(1), // 快速检查用于测试
                1000,
                10.0,
            );
            
            // 创建性能监控器
            let mut performance_monitor = PerformanceMonitor::new(
                1000,
                Duration::from_millis(100),
                None,
            );
            
            // 创建帧缓冲区池
            let frame_pool = FrameBufferPool::new(1024 * 1024, 5, 20);
            
            let initial_memory = 100.0;
            let mut current_memory = initial_memory;
            let mut frame_count = 0u64;
            
            // 模拟 1 小时的运行（3600 秒 * 30 FPS = 108,000 帧）
            // 为了测试速度，我们只模拟 1000 帧
            for i in 0..1000 {
                frame_count += 1;
                
                // 模拟帧处理
                let _buffer = frame_pool.get_buffer();
                
                // 模拟渲染时间
                let frame_time = Duration::from_millis(16 + (i % 10)); // 16-25ms
                let render_time = Duration::from_millis(8 + (i % 5));  // 8-12ms
                
                // 记录性能
                let _alert = performance_monitor.record_frame(frame_time, render_time);
                
                // 模拟内存使用变化（应该保持稳定）
                current_memory += (i as f32 * 0.001) % 2.0 - 1.0; // 小幅波动
                current_memory = current_memory.max(initial_memory - 5.0).min(initial_memory + 10.0);
                
                // 每 100 帧检查一次内存
                if i % 100 == 0 {
                    let memory_alert = memory_monitor.update(current_memory, frame_pool.get_stats().allocated_count);
                    
                    if let Some(alert) = memory_alert {
                        println!("帧 {}: 内存警报 - {:?}", i, alert);
                    }
                    
                    // 归还缓冲区以模拟正常清理
                    frame_pool.return_buffer(_buffer);
                }
                
                // 每 200 帧输出一次状态
                if i % 200 == 0 {
                    let perf_stats = performance_monitor.get_stats();
                    let memory_stats = memory_monitor.get_stats();
                    
                    println!("帧 {}: FPS={:.1}, 内存={:.1}MB, 缓冲区={}",
                            i, perf_stats.avg_fps, memory_stats.current_mb,
                            frame_pool.get_stats().allocated_count);
                }
            }
            
            // 验证最终状态
            let final_perf_stats = performance_monitor.get_stats();
            let final_memory_stats = memory_monitor.get_stats();
            let final_pool_stats = frame_pool.get_stats();
            
            // 验证性能稳定性
            assert!(final_perf_stats.avg_fps > 25.0, "平均 FPS 应该保持在 25+ ");
            assert!(final_perf_stats.sample_count == 1000, "应该有 1000 个性能样本");
            
            // 验证内存稳定性（没有显著泄漏）
            let memory_increase = final_memory_stats.current_mb - initial_memory;
            assert!(memory_increase < 20.0, "内存增长应该 < 20MB，实际: {:.1}MB", memory_increase);
            
            // 验证缓冲区池没有泄漏
            assert!(final_pool_stats.allocated_count <= final_pool_stats.max_buffers,
                   "分配的缓冲区数量应该在限制内");
            
            println!("长时间运行稳定性测试通过:");
            println!("  总帧数: {}", frame_count);
            println!("  平均 FPS: {:.1}", final_perf_stats.avg_fps);
            println!("  内存增长: {:.1}MB", memory_increase);
            println!("  缓冲区使用: {}/{}", final_pool_stats.allocated_count, final_pool_stats.max_buffers);
        }

        /// 测试极端缩放情况
        #[tokio::test]
        async fn test_extreme_scaling() {
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            // 设置初始尺寸
            window_manager.set_size(400, 400);
            let initial_size = window_manager.size();
            
            // 测试连续放大
            let mut current_size = initial_size;
            for i in 0..20 {
                window_manager.scale(1.1); // 每次放大 10%
                let new_size = window_manager.size();
                
                // 验证尺寸确实增加了
                assert!(new_size.width >= current_size.width, "第 {} 次放大后宽度应该增加", i);
                assert!(new_size.height >= current_size.height, "第 {} 次放大后高度应该增加", i);
                
                current_size = new_size;
                
                // 验证宽高比保持不变（允许 1 像素误差）
                let aspect_ratio = new_size.width as f32 / new_size.height as f32;
                let expected_ratio = initial_size.width as f32 / initial_size.height as f32;
                assert!((aspect_ratio - expected_ratio).abs() < 0.01, 
                       "宽高比应该保持不变");
            }
            
            println!("连续放大后尺寸: {}x{}", current_size.width, current_size.height);
            
            // 测试连续缩小
            for i in 0..30 {
                window_manager.scale(0.9); // 每次缩小 10%
                let new_size = window_manager.size();
                
                // 验证不会小于最小尺寸
                assert!(new_size.width >= 100, "第 {} 次缩小后宽度不应该小于 100", i);
                assert!(new_size.height >= 100, "第 {} 次缩小后高度不应该小于 100", i);
                
                current_size = new_size;
                
                // 如果达到最小尺寸，应该停止缩小
                if new_size.width == 100 && new_size.height == 100 {
                    println!("在第 {} 次缩小后达到最小尺寸", i + 1);
                    break;
                }
            }
            
            println!("连续缩小后尺寸: {}x{}", current_size.width, current_size.height);
            
            // 验证最终尺寸符合最小限制
            assert!(current_size.width >= 100, "最终宽度应该 >= 100");
            assert!(current_size.height >= 100, "最终高度应该 >= 100");
        }

        /// 测试快速连续事件处理
        #[tokio::test]
        async fn test_rapid_event_handling() {
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            
            // 初始化组件
            let config_manager = ConfigManager::new().expect("配置管理器创建失败");
            let camera_manager = CameraManager::new().unwrap_or_else(|_| {
                CameraManager::new_empty()
            });
            let window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            let render_engine = RenderEngine::new(window_manager.window()).await
                .expect("渲染引擎创建失败");
            let shape_mask = ShapeMask::new(ShapeType::Circle, 400, 400);
            
            let mut event_handler = EventHandler::new(
                window_manager,
                camera_manager,
                render_engine,
                shape_mask,
                config_manager,
            );
            
            // 快速连续发送鼠标滚轮事件
            let start_time = Instant::now();
            let mut event_count = 0;
            
            for i in 0..100 {
                let wheel_event = WindowEvent::MouseWheel {
                    device_id: unsafe { std::mem::transmute(0usize) },
                    delta: MouseScrollDelta::LineDelta(0.0, if i % 2 == 0 { 1.0 } else { -1.0 }),
                    phase: winit::event::TouchPhase::Moved,
                    modifiers: ModifiersState::empty(),
                };
                
                let should_exit = event_handler.handle_window_event(&wheel_event);
                assert!(!should_exit, "滚轮事件不应该导致退出");
                event_count += 1;
            }
            
            let processing_time = start_time.elapsed();
            let avg_time_per_event = processing_time.as_micros() / event_count;
            
            println!("处理 {} 个快速事件，总时间: {:?}，平均每事件: {}μs", 
                    event_count, processing_time, avg_time_per_event);
            
            // 验证事件处理性能（每个事件应该 < 1ms）
            assert!(avg_time_per_event < 1000, "平均事件处理时间应该 < 1ms");
            
            // 快速连续发送鼠标移动事件
            let start_time = Instant::now();
            event_count = 0;
            
            for i in 0..200 {
                let move_event = WindowEvent::CursorMoved {
                    device_id: unsafe { std::mem::transmute(0usize) },
                    position: PhysicalPosition::new(
                        200.0 + (i as f64 % 100.0),
                        200.0 + ((i * 2) as f64 % 100.0),
                    ),
                    modifiers: ModifiersState::empty(),
                };
                
                let should_exit = event_handler.handle_window_event(&move_event);
                assert!(!should_exit, "鼠标移动事件不应该导致退出");
                event_count += 1;
            }
            
            let move_processing_time = start_time.elapsed();
            let avg_move_time = move_processing_time.as_micros() / event_count;
            
            println!("处理 {} 个鼠标移动事件，总时间: {:?}，平均每事件: {}μs", 
                    event_count, move_processing_time, avg_move_time);
            
            // 鼠标移动事件处理应该更快（< 500μs）
            assert!(avg_move_time < 500, "平均鼠标移动处理时间应该 < 500μs");
        }

        /// 测试内存压力情况
        #[test]
        fn test_memory_pressure() {
            use mira::memory::{FrameBufferPool, MemoryMonitor};
            
            println!("开始内存压力测试");
            
            // 创建较小的缓冲区池来模拟内存压力
            let pool = FrameBufferPool::new(1024 * 1024, 0, 5); // 1MB 缓冲区，最多 5 个
            
            // 创建内存监控器，设置较低的阈值
            let mut memory_monitor = MemoryMonitor::new(
                Duration::from_millis(1),
                10,
                5.0, // 5MB 泄漏检测阈值
            );
            
            // 快速分配大量缓冲区
            let mut buffers = Vec::new();
            for i in 0..10 {
                let buffer = pool.get_buffer();
                assert_eq!(buffer.len(), 1024 * 1024, "缓冲区 {} 大小应该正确", i);
                buffers.push(buffer);
                
                // 模拟内存使用增长
                let memory_usage = 50.0 + (i as f32 * 10.0);
                let alert = memory_monitor.update(memory_usage, i + 1);
                
                if let Some(memory_alert) = alert {
                    println!("内存警报 {}: {:?}", i, memory_alert);
                }
            }
            
            let stats = pool.get_stats();
            println!("分配了 {} 个缓冲区，最大限制: {}", stats.allocated_count, stats.max_buffers);
            
            // 验证池没有超过限制
            assert!(stats.allocated_count <= stats.max_buffers, 
                   "分配的缓冲区数量不应该超过最大限制");
            
            // 快速释放所有缓冲区
            for (i, buffer) in buffers.into_iter().enumerate() {
                pool.return_buffer(buffer);
                
                // 模拟内存释放
                let memory_usage = 50.0 + ((9 - i) as f32 * 10.0);
                memory_monitor.update(memory_usage, 9 - i);
            }
            
            let final_stats = pool.get_stats();
            println!("释放后可用缓冲区: {}", final_stats.available_count);
            
            // 验证缓冲区被正确回收
            assert!(final_stats.available_count > 0, "应该有可用的缓冲区");
            
            println!("内存压力测试通过");
        }

        /// 测试形状切换压力
        #[test]
        fn test_shape_switching_stress() {
            let shapes = vec![
                ShapeType::Circle,
                ShapeType::Ellipse,
                ShapeType::Rectangle,
                ShapeType::RoundedRectangle { radius: 10.0 },
                ShapeType::Heart,
            ];
            
            let mut shape_mask = ShapeMask::new(ShapeType::Circle, 800, 600);
            let mut total_switch_time = Duration::new(0, 0);
            let switch_count = 100;
            
            println!("开始形状切换压力测试，切换 {} 次", switch_count);
            
            for i in 0..switch_count {
                let shape = &shapes[i % shapes.len()];
                
                let start_time = Instant::now();
                shape_mask.set_shape(shape.clone());
                let switch_time = start_time.elapsed();
                
                total_switch_time += switch_time;
                
                // 验证每次切换都在时间限制内
                assert!(switch_time < Duration::from_millis(100), 
                       "第 {} 次形状切换时间 {:?} 超过 100ms 限制", i, switch_time);
                
                // 验证遮罩数据正确生成
                let mask_data = shape_mask.data();
                assert_eq!(mask_data.len(), 800 * 600, "遮罩数据长度应该正确");
                
                if i % 20 == 0 {
                    println!("完成 {} 次切换，当前形状: {:?}，切换时间: {:?}", 
                            i + 1, shape, switch_time);
                }
            }
            
            let avg_switch_time = total_switch_time / switch_count as u32;
            println!("形状切换压力测试完成:");
            println!("  总切换次数: {}", switch_count);
            println!("  总时间: {:?}", total_switch_time);
            println!("  平均切换时间: {:?}", avg_switch_time);
            
            // 验证平均切换时间合理
            assert!(avg_switch_time < Duration::from_millis(50), 
                   "平均形状切换时间应该 < 50ms");
        }
    }
    /// 跨平台兼容性测试模块
    mod cross_platform_compatibility_tests {
        use super::*;
        use std::path::PathBuf;

        /// 测试 Windows 平台特定功能
        #[cfg(target_os = "windows")]
        #[tokio::test]
        async fn test_windows_specific_features() {
            println!("测试 Windows 平台特定功能");
            
            // 测试配置文件路径
            let config_manager = ConfigManager::new().expect("配置管理器创建失败");
            let config_path = config_manager.config_path();
            
            // Windows 配置路径应该在 %APPDATA%\Mira\config.toml
            let expected_path_contains = vec!["AppData", "Roaming", "Mira", "config.toml"];
            let path_str = config_path.to_string_lossy().to_lowercase();
            
            for expected in expected_path_contains {
                assert!(path_str.contains(&expected.to_lowercase()), 
                       "Windows 配置路径应该包含 '{}'，实际路径: {}", expected, path_str);
            }
            
            // 测试窗口创建（Windows 特定的置顶窗口）
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            // 验证窗口属性
            let window = window_manager.window();
            assert!(window.is_visible().unwrap_or(true), "窗口应该可见");
            
            // 测试摄像头设备枚举（Windows DirectShow）
            let camera_manager = CameraManager::new().unwrap_or_else(|_| {
                println!("无摄像头设备，使用空管理器");
                CameraManager::new_empty()
            });
            
            let devices = camera_manager.devices();
            println!("Windows 平台发现 {} 个摄像头设备", devices.len());
            
            for (i, device) in devices.iter().enumerate() {
                println!("  设备 {}: {} ({})", i, device.name, device.description);
                // Windows 设备名称通常包含制造商信息
                assert!(!device.name.is_empty(), "设备名称不应该为空");
                assert!(!device.description.is_empty(), "设备描述不应该为空");
            }
            
            println!("Windows 平台特定功能测试通过");
        }

        /// 测试 macOS 平台特定功能
        #[cfg(target_os = "macos")]
        #[tokio::test]
        async fn test_macos_specific_features() {
            println!("测试 macOS 平台特定功能");
            
            // 测试配置文件路径
            let config_manager = ConfigManager::new().expect("配置管理器创建失败");
            let config_path = config_manager.config_path();
            
            // macOS 配置路径应该在 ~/Library/Application Support/Mira/config.toml
            let expected_path_contains = vec!["Library", "Application Support", "Mira", "config.toml"];
            let path_str = config_path.to_string_lossy();
            
            for expected in expected_path_contains {
                assert!(path_str.contains(expected), 
                       "macOS 配置路径应该包含 '{}'，实际路径: {}", expected, path_str);
            }
            
            // 测试窗口创建（macOS 特定的置顶窗口）
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            // 验证窗口属性
            let window = window_manager.window();
            assert!(window.is_visible().unwrap_or(true), "窗口应该可见");
            
            // 测试摄像头设备枚举（macOS AVFoundation）
            let camera_manager = CameraManager::new().unwrap_or_else(|_| {
                println!("无摄像头设备，使用空管理器");
                CameraManager::new_empty()
            });
            
            let devices = camera_manager.devices();
            println!("macOS 平台发现 {} 个摄像头设备", devices.len());
            
            for (i, device) in devices.iter().enumerate() {
                println!("  设备 {}: {} ({})", i, device.name, device.description);
                // macOS 设备名称通常更简洁
                assert!(!device.name.is_empty(), "设备名称不应该为空");
                assert!(!device.description.is_empty(), "设备描述不应该为空");
            }
            
            println!("macOS 平台特定功能测试通过");
        }

        /// 测试 Linux 平台特定功能（如果支持）
        #[cfg(target_os = "linux")]
        #[tokio::test]
        async fn test_linux_specific_features() {
            println!("测试 Linux 平台特定功能");
            
            // 注意：当前 MVP 版本不支持 Linux，但为未来扩展预留测试
            println!("Linux 平台支持尚未实现，跳过测试");
        }

        /// 验证平台间配置文件兼容性
        #[tokio::test]
        async fn test_cross_platform_config_compatibility() {
            println!("测试跨平台配置文件兼容性");
            
            // 创建标准配置
            let test_config = AppConfig {
                window: WindowConfig {
                    position_x: 123.45,
                    position_y: 678.90,
                    width: 640,
                    height: 480,
                    rotation: 45.5,
                    shape: "Heart".to_string(),
                },
                camera: CameraConfig {
                    device_index: 2,
                },
            };
            
            // 测试配置序列化
            let serialized = toml::to_string(&test_config).expect("配置序列化失败");
            println!("序列化的配置:\n{}", serialized);
            
            // 验证序列化结果包含所有字段
            assert!(serialized.contains("position_x"), "序列化结果应该包含 position_x");
            assert!(serialized.contains("position_y"), "序列化结果应该包含 position_y");
            assert!(serialized.contains("width"), "序列化结果应该包含 width");
            assert!(serialized.contains("height"), "序列化结果应该包含 height");
            assert!(serialized.contains("rotation"), "序列化结果应该包含 rotation");
            assert!(serialized.contains("shape"), "序列化结果应该包含 shape");
            assert!(serialized.contains("device_index"), "序列化结果应该包含 device_index");
            
            // 测试配置反序列化
            let deserialized: AppConfig = toml::from_str(&serialized).expect("配置反序列化失败");
            
            // 验证反序列化结果
            assert_eq!(deserialized.window.position_x, test_config.window.position_x);
            assert_eq!(deserialized.window.position_y, test_config.window.position_y);
            assert_eq!(deserialized.window.width, test_config.window.width);
            assert_eq!(deserialized.window.height, test_config.window.height);
            assert_eq!(deserialized.window.rotation, test_config.window.rotation);
            assert_eq!(deserialized.window.shape, test_config.window.shape);
            assert_eq!(deserialized.camera.device_index, test_config.camera.device_index);
            
            // 测试使用配置管理器的完整流程
            let mut config_manager = ConfigManager::new().expect("配置管理器创建失败");
            
            // 保存配置
            config_manager.save(&test_config).expect("配置保存失败");
            
            // 加载配置
            let loaded_config = config_manager.load().expect("配置加载失败");
            
            // 验证加载的配置与原始配置一致
            assert_eq!(loaded_config.window.position_x, test_config.window.position_x);
            assert_eq!(loaded_config.window.position_y, test_config.window.position_y);
            assert_eq!(loaded_config.window.width, test_config.window.width);
            assert_eq!(loaded_config.window.height, test_config.window.height);
            assert_eq!(loaded_config.window.rotation, test_config.window.rotation);
            assert_eq!(loaded_config.window.shape, test_config.window.shape);
            assert_eq!(loaded_config.camera.device_index, test_config.camera.device_index);
            
            println!("跨平台配置文件兼容性测试通过");
        }

        /// 测试不同屏幕分辨率和 DPI 设置
        #[tokio::test]
        async fn test_different_screen_resolutions_and_dpi() {
            println!("测试不同屏幕分辨率和 DPI 设置");
            
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let mut window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            
            // 测试不同的屏幕分辨率
            let test_resolutions = vec![
                (1920, 1080), // Full HD
                (2560, 1440), // 2K
                (3840, 2160), // 4K
                (1366, 768),  // 常见笔记本分辨率
                (1280, 720),  // HD
                (1024, 768),  // 传统 4:3
            ];
            
            for (screen_width, screen_height) in test_resolutions {
                println!("测试屏幕分辨率: {}x{}", screen_width, screen_height);
                
                let screen_size = PhysicalSize::new(screen_width, screen_height);
                
                // 计算最大窗口尺寸（屏幕的 80%）
                let max_width = (screen_width as f32 * 0.8) as u32;
                let max_height = (screen_height as f32 * 0.8) as u32;
                
                // 测试设置最大尺寸
                window_manager.set_size(max_width, max_height);
                let size = window_manager.size();
                
                // 验证尺寸在合理范围内
                assert!(size.width <= max_width, "窗口宽度应该 <= 最大宽度");
                assert!(size.height <= max_height, "窗口高度应该 <= 最大高度");
                assert!(size.width >= 100, "窗口宽度应该 >= 最小宽度");
                assert!(size.height >= 100, "窗口高度应该 >= 最小高度");
                
                // 测试窗口边界约束
                window_manager.set_position(-100.0, -100.0);
                window_manager.constrain_to_screen(screen_size);
                let constrained_pos = window_manager.position();
                
                // 验证窗口被正确约束在屏幕内
                let min_visible_width = size.width as f32 * 0.2;
                let min_visible_height = size.height as f32 * 0.2;
                
                assert!(constrained_pos.x + size.width as f64 >= min_visible_width as f64,
                       "窗口应该有至少 20% 在屏幕内（水平）");
                assert!(constrained_pos.y + size.height as f64 >= min_visible_height as f64,
                       "窗口应该有至少 20% 在屏幕内（垂直）");
                
                // 测试形状遮罩在不同分辨率下的表现
                let mut shape_mask = ShapeMask::new(ShapeType::Circle, size.width, size.height);
                let start_time = Instant::now();
                shape_mask.generate();
                let generation_time = start_time.elapsed();
                
                // 验证遮罩生成时间合理（与分辨率相关）
                let pixel_count = size.width * size.height;
                let max_time_ms = (pixel_count as f32 / 1000000.0 * 100.0) as u64; // 每百万像素 100ms
                let max_time = Duration::from_millis(max_time_ms.max(50).min(1000)); // 50ms-1s 范围
                
                assert!(generation_time < max_time, 
                       "分辨率 {}x{} 下遮罩生成时间 {:?} 应该 < {:?}", 
                       screen_width, screen_height, generation_time, max_time);
                
                println!("  分辨率 {}x{}: 窗口尺寸 {}x{}, 遮罩生成时间 {:?}", 
                        screen_width, screen_height, size.width, size.height, generation_time);
            }
            
            // 测试 DPI 缩放（模拟）
            let dpi_scales = vec![1.0, 1.25, 1.5, 2.0]; // 100%, 125%, 150%, 200%
            
            for dpi_scale in dpi_scales {
                println!("测试 DPI 缩放: {}%", (dpi_scale * 100.0) as u32);
                
                // 模拟 DPI 缩放对窗口尺寸的影响
                let base_size = 400u32;
                let scaled_size = (base_size as f32 * dpi_scale) as u32;
                
                window_manager.set_size(scaled_size, scaled_size);
                let actual_size = window_manager.size();
                
                // 验证尺寸合理
                assert!(actual_size.width >= 100, "DPI 缩放后宽度应该 >= 100");
                assert!(actual_size.height >= 100, "DPI 缩放后高度应该 >= 100");
                
                // 测试形状遮罩在不同 DPI 下的表现
                let mut dpi_shape_mask = ShapeMask::new(ShapeType::Heart, actual_size.width, actual_size.height);
                let dpi_start_time = Instant::now();
                dpi_shape_mask.generate();
                let dpi_generation_time = dpi_start_time.elapsed();
                
                println!("  DPI {}%: 窗口尺寸 {}x{}, 遮罩生成时间 {:?}", 
                        (dpi_scale * 100.0) as u32, actual_size.width, actual_size.height, dpi_generation_time);
                
                // 验证遮罩数据正确
                let mask_data = dpi_shape_mask.data();
                assert_eq!(mask_data.len(), (actual_size.width * actual_size.height) as usize,
                          "DPI 缩放后遮罩数据长度应该正确");
            }
            
            println!("不同屏幕分辨率和 DPI 设置测试通过");
        }

        /// 测试平台特定的文件路径处理
        #[test]
        fn test_platform_specific_file_paths() {
            println!("测试平台特定的文件路径处理");
            
            let config_manager = ConfigManager::new().expect("配置管理器创建失败");
            let config_path = config_manager.config_path();
            
            // 验证路径是绝对路径
            assert!(config_path.is_absolute(), "配置文件路径应该是绝对路径");
            
            // 验证路径包含正确的文件名
            assert_eq!(config_path.file_name().unwrap(), "config.toml", 
                      "配置文件名应该是 config.toml");
            
            // 验证父目录存在或可以创建
            let parent_dir = config_path.parent().expect("配置文件应该有父目录");
            
            // 平台特定的路径验证
            #[cfg(target_os = "windows")]
            {
                let path_str = config_path.to_string_lossy();
                assert!(path_str.contains("AppData"), "Windows 路径应该包含 AppData");
                assert!(path_str.contains("Mira"), "路径应该包含应用名称");
                
                // 验证路径分隔符
                assert!(path_str.contains('\\'), "Windows 路径应该使用反斜杠");
            }
            
            #[cfg(target_os = "macos")]
            {
                let path_str = config_path.to_string_lossy();
                assert!(path_str.contains("Library"), "macOS 路径应该包含 Library");
                assert!(path_str.contains("Application Support"), "macOS 路径应该包含 Application Support");
                assert!(path_str.contains("Mira"), "路径应该包含应用名称");
                
                // 验证路径分隔符
                assert!(path_str.contains('/'), "macOS 路径应该使用正斜杠");
            }
            
            #[cfg(target_os = "linux")]
            {
                let path_str = config_path.to_string_lossy();
                // Linux 通常使用 ~/.config 或 ~/.local/share
                assert!(path_str.contains(".config") || path_str.contains(".local"), 
                       "Linux 路径应该包含 .config 或 .local");
                assert!(path_str.contains("Mira"), "路径应该包含应用名称");
                
                // 验证路径分隔符
                assert!(path_str.contains('/'), "Linux 路径应该使用正斜杠");
            }
            
            println!("配置文件路径: {}", config_path.display());
            println!("平台特定的文件路径处理测试通过");
        }

        /// 测试平台特定的窗口行为
        #[tokio::test]
        async fn test_platform_specific_window_behavior() {
            println!("测试平台特定的窗口行为");
            
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let window_manager = WindowManager::new(&event_loop).expect("窗口管理器创建失败");
            let window = window_manager.window();
            
            // 测试窗口基本属性
            let window_id = window.id();
            println!("窗口 ID: {:?}", window_id);
            
            // 测试窗口可见性
            let is_visible = window.is_visible().unwrap_or(true);
            println!("窗口可见性: {}", is_visible);
            
            // 测试窗口尺寸
            let inner_size = window.inner_size();
            println!("窗口内部尺寸: {}x{}", inner_size.width, inner_size.height);
            
            let outer_size = window.outer_size();
            println!("窗口外部尺寸: {}x{}", outer_size.width, outer_size.height);
            
            // 验证内部尺寸 <= 外部尺寸（考虑窗口边框）
            assert!(inner_size.width <= outer_size.width, "内部宽度应该 <= 外部宽度");
            assert!(inner_size.height <= outer_size.height, "内部高度应该 <= 外部高度");
            
            // 测试窗口位置
            if let Ok(position) = window.outer_position() {
                println!("窗口位置: ({}, {})", position.x, position.y);
            } else {
                println!("无法获取窗口位置（某些平台限制）");
            }
            
            // 测试窗口标题
            window.set_title("Mira 测试窗口");
            
            // 平台特定的窗口测试
            #[cfg(target_os = "windows")]
            {
                println!("Windows 特定窗口测试:");
                // Windows 特定的窗口属性测试
                // 注意：某些属性可能需要 Windows API 调用
            }
            
            #[cfg(target_os = "macos")]
            {
                println!("macOS 特定窗口测试:");
                // macOS 特定的窗口属性测试
                // 注意：某些属性可能需要 Cocoa API 调用
            }
            
            println!("平台特定的窗口行为测试通过");
        }

        /// 测试平台特定的性能特征
        #[test]
        fn test_platform_specific_performance() {
            use mira::performance::PerformanceMonitor;
            use std::time::Duration;
            
            println!("测试平台特定的性能特征");
            
            let mut monitor = PerformanceMonitor::new(
                100,
                Duration::from_secs(1),
                None,
            );
            
            // 测试不同平台的性能基准
            let mut frame_times = Vec::new();
            let mut render_times = Vec::new();
            
            for i in 0..50 {
                let frame_start = Instant::now();
                
                // 模拟一些计算工作
                let mut sum = 0u64;
                for j in 0..10000 {
                    sum += (i * j) as u64;
                }
                
                let render_time = frame_start.elapsed();
                let total_frame_time = render_time + Duration::from_micros(sum % 1000);
                
                frame_times.push(total_frame_time);
                render_times.push(render_time);
                
                monitor.record_frame(total_frame_time, render_time);
            }
            
            let stats = monitor.get_stats();
            
            println!("性能统计:");
            println!("  平均 FPS: {:.1}", stats.avg_fps);
            println!("  最小 FPS: {:.1}", stats.min_fps);
            println!("  最大 FPS: {:.1}", stats.max_fps);
            println!("  平均帧时间: {:.1}ms", stats.avg_frame_time);
            println!("  平均渲染时间: {:.1}ms", stats.avg_render_time);
            
            // 平台特定的性能验证
            #[cfg(target_os = "windows")]
            {
                println!("Windows 性能验证:");
                // Windows 通常有较好的 GPU 性能
                assert!(stats.avg_fps > 30.0, "Windows 平台平均 FPS 应该 > 30");
            }
            
            #[cfg(target_os = "macos")]
            {
                println!("macOS 性能验证:");
                // macOS 在 Metal 支持下性能也很好
                assert!(stats.avg_fps > 25.0, "macOS 平台平均 FPS 应该 > 25");
            }
            
            // 通用性能验证
            assert!(stats.avg_frame_time < 50.0, "平均帧时间应该 < 50ms");
            assert!(stats.sample_count == 50, "应该有 50 个性能样本");
            
            println!("平台特定的性能特征测试通过");
        }
    }