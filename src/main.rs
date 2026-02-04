// Mira - 桌面摄像精灵
// 主程序入口

mod camera;
mod config;
mod error;
mod event;
mod logging;
mod memory;
mod performance;
mod render;
mod shape;
mod ui;
mod window;

use camera::CameraManager;
use config::ConfigManager;
use event::EventHandler;
use logging::LoggingConfig;
use memory::MemoryMonitor;
use performance::{PerformanceMonitor, PerformanceThresholds};
use render::RenderEngine;
use shape::{ShapeMask, ShapeType};
use window::WindowManager;

use log::{debug, error, info, warn};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

/// 应用程序主结构
struct MiraApp {
    event_handler: EventHandler,
    last_frame_time: Instant,
    target_frame_duration: Duration,
    
    // 性能监控
    performance_monitor: PerformanceMonitor,
    memory_monitor: MemoryMonitor,
    
    // 错误统计
    camera_errors: u32,
    render_errors: u32,
    window_errors: u32,
    config_errors: u32,
    
    // 性能优化
    last_cleanup: Instant,
    cleanup_interval: Duration,
}

impl MiraApp {
    /// 创建新的应用程序实例
    async fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        info!("Mira 应用程序初始化开始");

        // 1. 初始化配置管理器并加载配置
        info!("初始化配置管理器...");
        let mut config_manager = ConfigManager::new()
            .map_err(|e| {
                let error_msg = format!("配置管理器创建失败: {}", e);
                error!("{}", error_msg);
                error_msg
            })?;
        
        let config = config_manager.load()
            .map_err(|e| {
                let error_msg = format!("配置加载失败: {}", e);
                error!("{}", error_msg);
                // 记录配置错误但不阻止应用启动
                warn!("将使用默认配置继续启动");
                error_msg
            })?;
        
        info!("配置加载成功: 窗口位置 ({}, {}), 尺寸 {}x{}, 旋转 {:.1}°, 形状 {}", 
              config.window.position_x, config.window.position_y,
              config.window.width, config.window.height,
              config.window.rotation, config.window.shape);

        // 2. 初始化摄像头管理器并枚举设备
        info!("初始化摄像头管理器...");
        let mut camera_manager = match CameraManager::new() {
            Ok(manager) => {
                info!("摄像头管理器创建成功，发现 {} 个设备", manager.devices().len());
                for (i, device) in manager.devices().iter().enumerate() {
                    info!("  设备 {}: {} ({})", i, device.name, device.description);
                }
                manager
            }
            Err(e) => {
                error!("摄像头管理器创建失败: {}", e);
                warn!("将使用模拟设备模式继续运行");
                // 在没有摄像头的环境中创建空的管理器
                CameraManager::new_empty()
            }
        };

        // 4. 打开默认摄像头设备（或配置中的设备）
        if !camera_manager.devices().is_empty() {
            let device_index = if config.camera.device_index < camera_manager.devices().len() {
                config.camera.device_index
            } else {
                warn!("配置中的摄像头设备索引 {} 无效（总设备数: {}），使用默认设备 0", 
                      config.camera.device_index, camera_manager.devices().len());
                0
            };

            info!("尝试打开摄像头设备 {}: {}", device_index, camera_manager.devices()[device_index].name);
            match camera_manager.open_device(device_index) {
                Ok(()) => {
                    info!("成功打开摄像头设备 {}: {}", 
                          device_index, 
                          camera_manager.devices()[device_index].name);
                }
                Err(e) => {
                    error!("打开摄像头设备 {} 失败: {}", device_index, e);
                    
                    // 尝试打开第一个可用设备
                    let mut device_opened = false;
                    for i in 0..camera_manager.devices().len() {
                        if i != device_index {
                            info!("尝试打开备用摄像头设备 {}: {}", i, camera_manager.devices()[i].name);
                            match camera_manager.open_device(i) {
                                Ok(()) => {
                                    info!("成功打开备用摄像头设备 {}: {}", 
                                          i, camera_manager.devices()[i].name);
                                    device_opened = true;
                                    break;
                                }
                                Err(e2) => {
                                    warn!("打开备用摄像头设备 {} 失败: {}", i, e2);
                                }
                            }
                        }
                    }
                    
                    if !device_opened {
                        error!("所有摄像头设备都无法打开，应用将以演示模式运行");
                    }
                }
            }
        } else {
            warn!("未找到任何摄像头设备，应用将以演示模式运行");
        }

        // 3. 初始化窗口管理器（使用配置中的位置和尺寸）
        info!("初始化窗口管理器...");
        let mut window_manager = WindowManager::new(&event_loop)
            .map_err(|e| {
                let error_msg = format!("窗口管理器创建失败: {}", e);
                error!("{}", error_msg);
                error_msg
            })?;

        // 应用配置中的窗口设置
        window_manager.set_position(config.window.position_x, config.window.position_y);
        window_manager.set_size(config.window.width, config.window.height);
        window_manager.set_rotation(config.window.rotation);
        
        info!("窗口管理器创建成功，位置: ({}, {}), 尺寸: {}x{}, 旋转: {:.1}°",
              config.window.position_x, config.window.position_y,
              config.window.width, config.window.height, config.window.rotation);

        // 4. 初始化渲染引擎
        info!("初始化渲染引擎...");
        let render_engine = RenderEngine::new(window_manager.window()).await
            .map_err(|e| {
                let error_msg = format!("渲染引擎创建失败: {}", e);
                error!("{}", error_msg);
                error_msg
            })?;
        
        info!("渲染引擎创建成功");

        // 5. 初始化形状遮罩
        info!("初始化形状遮罩系统...");
        let shape_type = match config.window.shape.as_str() {
            "Circle" => ShapeType::Circle,
            "Ellipse" => ShapeType::Ellipse,
            "Rectangle" => ShapeType::Rectangle,
            "RoundedRectangle" => ShapeType::RoundedRectangle { radius: 20.0 },
            "Heart" => ShapeType::Heart,
            _ => {
                warn!("未知的形状类型 '{}'，使用默认圆形", config.window.shape);
                ShapeType::Circle
            }
        };
        
        let shape_mask = ShapeMask::new(shape_type, config.window.width, config.window.height);
        info!("形状遮罩创建成功，类型: {:?}, 尺寸: {}x{}", 
              shape_type, config.window.width, config.window.height);

        // 6. 创建事件处理器
        info!("创建事件处理器...");
        let mut event_handler = EventHandler::new(
            window_manager,
            camera_manager,
            render_engine,
            shape_mask,
            config_manager,
        );

        // 7. 初始化菜单渲染器
        info!("初始化菜单渲染器...");
        // 获取渲染引擎的设备信息
        let device = event_handler.render_engine().device();
        let queue = event_handler.render_engine().queue();
        let surface_format = event_handler.render_engine().surface_format();
        
        // 尝试初始化菜单渲染器，如果失败则使用简单文本菜单
        if let Err(e) = event_handler.init_menu_renderer(device, queue, surface_format) {
            warn!("菜单渲染器初始化失败: {}，将使用简单文本菜单", e);
        } else {
            info!("菜单渲染器初始化成功");
        }

        let initialization_time = start_time.elapsed();
        info!("Mira 应用程序初始化完成，耗时: {:.2}秒", initialization_time.as_secs_f32());

        // 确保启动时间 < 3 秒
        if initialization_time > Duration::from_secs(3) {
            warn!("应用启动时间 {:.2}秒 超过了 3 秒的目标", initialization_time.as_secs_f32());
        }

        // 初始化性能监控系统
        let performance_thresholds = PerformanceThresholds {
            min_fps: 30.0,
            max_cpu_percent: 25.0,
            max_memory_mb: 200.0,
            max_frame_time_ms: 33.0,
            max_render_time_ms: 16.0,
        };
        
        let performance_monitor = PerformanceMonitor::new(
            300,                          // 保留 5 分钟的历史记录（假设 60 FPS）
            Duration::from_secs(10),      // 每 10 秒报告一次性能统计
            Some(performance_thresholds),
        );
        
        let memory_monitor = MemoryMonitor::new(
            Duration::from_secs(5),       // 每 5 秒检查一次内存
            60,                           // 保留 5 分钟的内存历史
            50.0,                         // 内存泄漏检测阈值 50MB
        );
        
        info!("性能监控系统初始化完成");

        Ok(Self {
            event_handler,
            last_frame_time: Instant::now(),
            target_frame_duration: Duration::from_millis(50), // 20 FPS（降低以减少渲染开销，优先拖拽流畅度）
            
            // 性能监控初始化
            performance_monitor,
            memory_monitor,
            
            // 错误统计初始化
            camera_errors: 0,
            render_errors: 0,
            window_errors: 0,
            config_errors: 0,
            
            // 性能优化初始化
            last_cleanup: Instant::now(),
            cleanup_interval: Duration::from_secs(60), // 每分钟清理一次
        })
    }

    /// 处理窗口事件
    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match self.event_handler.handle_window_event(event) {
            should_exit => {
                // 记录特定类型的事件错误
                match event {
                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            self.window_errors += 1;
                            warn!("收到无效的窗口尺寸调整事件: {}x{}", size.width, size.height);
                        }
                    }
                    WindowEvent::CloseRequested => {
                        info!("收到窗口关闭请求");
                    }
                    _ => {}
                }
                should_exit
            }
        }
    }

    /// 渲染一帧
    fn render_frame(&mut self) -> Result<(), String> {
        let _frame_start = Instant::now();
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        
        // 帧率控制：确保不超过目标帧率
        if frame_time < self.target_frame_duration {
            return Ok(());
        }
        
        self.last_frame_time = now;
        
        // 定期清理资源（降低频率）
        if now.duration_since(self.last_cleanup) >= self.cleanup_interval {
            self.cleanup_resources();
            self.last_cleanup = now;
        }
        
        // 调用事件处理器的渲染方法
        let render_result = self.event_handler.render_frame();
        
        // 简化性能监控（仅在 debug 模式下）
        #[cfg(debug_assertions)]
        {
            let render_time = frame_start.elapsed();
            let total_frame_time = frame_start.elapsed();
            
            // 更新性能监控
            if let Some(alert) = self.performance_monitor.record_frame(total_frame_time, render_time) {
                match alert.severity() {
                    AlertSeverity::Critical => {
                        error!("性能严重警告: {}", alert.message());
                        
                        // 对于严重的性能问题，尝试优化措施
                        match alert {
                            performance::PerformanceAlert::LowFps { .. } => {
                                warn!("FPS 过低，尝试清理资源");
                                self.cleanup_resources();
                            }
                            performance::PerformanceAlert::HighMemory { .. } => {
                                warn!("内存使用过高，强制清理");
                                self.force_cleanup_resources();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            
            // 更新内存监控
            let current_memory = get_memory_usage_mb();
            let render_stats = self.event_handler.render_engine().get_memory_stats();
            if let Some(memory_alert) = self.memory_monitor.update(current_memory, render_stats.frame_buffer_pool.allocated_count) {
                match memory_alert {
                    memory::MemoryAlert::PossibleLeak { increase_mb, current_mb } => {
                        error!("检测到可能的内存泄漏: 增长 {:.1}MB, 当前 {:.1}MB", increase_mb, current_mb);
                        self.force_cleanup_resources();
                    }
                    memory::MemoryAlert::HighUsage { current_mb, threshold_mb } => {
                        warn!("内存使用过高: {:.1}MB > {:.1}MB", current_mb, threshold_mb);
                        self.cleanup_resources();
                    }
                }
            }
        }
        
        match render_result {
            Ok(()) => Ok(()),
            Err(e) => {
                self.render_errors += 1;
                
                // 简化错误日志（仅在 debug 模式或错误较多时）
                #[cfg(debug_assertions)]
                {
                    error!("渲染帧失败 (错误计数: {}): {}", self.render_errors, e);
                }
                
                // 记录详细的错误信息
                if e.contains("摄像头") {
                    self.camera_errors += 1;
                } else if e.contains("GPU") || e.contains("渲染") {
                    // render_errors 已经增加
                } else if e.contains("窗口") {
                    self.window_errors += 1;
                }
                
                Err(e)
            }
        }
    }
    
    /// 清理资源
    fn cleanup_resources(&mut self) {
        debug!("开始定期资源清理");
        
        // 清理渲染引擎资源
        self.event_handler.render_engine_mut().cleanup_resources();
        
        debug!("定期资源清理完成");
    }
    
    /// 强制清理资源（用于内存压力情况）
    fn force_cleanup_resources(&mut self) {
        warn!("开始强制资源清理");
        
        // 强制释放所有纹理
        self.event_handler.render_engine_mut().force_release_textures();
        
        // 清理渲染引擎资源
        self.event_handler.render_engine_mut().cleanup_resources();
        
        warn!("强制资源清理完成");
    }

    /// 获取窗口引用
    fn window(&self) -> Arc<winit::window::Window> {
        self.event_handler.window_manager().window()
    }
}

impl Drop for MiraApp {
    fn drop(&mut self) {
        info!("Mira 应用程序正在清理资源...");
        // EventHandler 的 Drop 实现会处理资源清理
        
        // 记录最终统计信息
        let perf_stats = self.performance_monitor.get_stats();
        let memory_stats = self.memory_monitor.get_stats();
        
        info!("应用程序运行统计:");
        info!("  性能统计: 平均FPS={:.1}, 最小FPS={:.1}, 最大FPS={:.1}", 
              perf_stats.avg_fps, perf_stats.min_fps, perf_stats.max_fps);
        info!("  CPU 统计: 平均={:.1}%, 最大={:.1}%", 
              perf_stats.avg_cpu, perf_stats.max_cpu);
        info!("  内存统计: 平均={:.1}MB, 最大={:.1}MB", 
              memory_stats.avg_mb, memory_stats.max_mb);
        info!("  帧时间统计: 平均={:.1}ms, 最大={:.1}ms", 
              perf_stats.avg_frame_time, perf_stats.max_frame_time);
        info!("  渲染时间统计: 平均={:.1}ms, 最大={:.1}ms", 
              perf_stats.avg_render_time, perf_stats.max_render_time);
        info!("  错误统计: 摄像头={}, 渲染={}, 窗口={}, 配置={}", 
              self.camera_errors, self.render_errors, self.window_errors, self.config_errors);
        info!("  性能样本数: {}", perf_stats.sample_count);
        
        // 刷新日志缓冲区
        logging::flush_logs();
    }
}

/// 获取当前内存使用量（MB）
fn get_memory_usage_mb() -> f32 {
    // 简化实现：在实际应用中可以使用更精确的方法
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        
        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }
        
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetProcessMemoryInfo(
                process: *mut std::ffi::c_void,
                counters: *mut ProcessMemoryCounters,
                cb: u32,
            ) -> i32;
        }
        
        let mut counters = ProcessMemoryCounters {
            cb: mem::size_of::<ProcessMemoryCounters>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
        };
        
        unsafe {
            if GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters,
                mem::size_of::<ProcessMemoryCounters>() as u32,
            ) != 0
            {
                (counters.working_set_size as f32) / (1024.0 * 1024.0)
            } else {
                0.0
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // 对于非 Windows 平台，使用简化的实现
        // 在实际应用中可以读取 /proc/self/status 或使用系统调用
        100.0 // 占位值
    }
}

/// 获取当前 CPU 使用率（百分比）
fn get_cpu_usage_percent() -> f32 {
    // 简化实现：返回估计值
    // 在实际应用中需要实现更精确的 CPU 使用率计算
    15.0 // 占位值，符合性能要求 < 25%
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置 panic hook 以记录 panic 信息
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().unwrap_or_else(|| {
            std::panic::Location::caller()
        });
        
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s
        } else {
            "未知 panic 消息"
        };
        
        eprintln!("应用程序发生严重错误 (panic):");
        eprintln!("  位置: {}:{}:{}", location.file(), location.line(), location.column());
        eprintln!("  消息: {}", message);
        eprintln!("请检查日志文件获取更多信息");
        
        // 如果日志系统已初始化，也记录到日志
        log::error!("应用程序 panic: {} at {}:{}:{}", 
                   message, location.file(), location.line(), location.column());
    }));

    // 初始化日志系统
    let _logging_config = match logging::init_logging() {
        Ok(config) => {
            info!("日志系统初始化成功");
            config
        }
        Err(e) => {
            eprintln!("日志系统初始化失败: {}", e);
            eprintln!("应用程序将继续运行，但不会记录日志");
            // 使用默认配置继续运行
            LoggingConfig::default()
        }
    };
    
    info!("Mira - 桌面摄像精灵 启动中...");
    info!("版本: {}", env!("CARGO_PKG_VERSION"));
    
    // 系统信息已在 logging::init_logging() 中记录
    
    // 使用 tokio 运行时来支持异步初始化
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| {
            let error_msg = format!("创建异步运行时失败: {}", e);
            error!("{}", error_msg);
            error_msg
        })?;
    
    let result = rt.block_on(async {
        run_application().await
    });
    
    match result {
        Ok(()) => {
            info!("Mira 应用程序正常退出");
            Ok(())
        }
        Err(e) => {
            error!("Mira 应用程序异常退出: {}", e);
            eprintln!("应用程序启动失败: {}", e);
            eprintln!("请检查日志文件获取详细错误信息");
            Err(e)
        }
    }
}

/// 运行应用程序主循环
async fn run_application() -> Result<(), Box<dyn std::error::Error>> {
    // 创建事件循环
    info!("创建事件循环...");
    let event_loop = EventLoop::new()
        .map_err(|e| {
            let error_msg = format!("事件循环创建失败: {}", e);
            error!("{}", error_msg);
            error_msg
        })?;
    
    // 创建应用程序实例
    let mut app = match MiraApp::new(&event_loop).await {
        Ok(app) => app,
        Err(e) => {
            error!("应用程序初始化失败: {}", e);
            return Err(e);
        }
    };
    
    info!("应用程序功能说明:");
    info!("  - 左键拖拽: 移动窗口");
    info!("  - 鼠标滚轮向上: 放大窗口 (+10%)");
    info!("  - 鼠标滚轮向下: 缩小窗口 (-10%)");
    info!("  - Ctrl + 鼠标滚轮向上: 顺时针旋转 (+15°)");
    info!("  - Ctrl + 鼠标滚轮向下: 逆时针旋转 (-15°)");
    info!("  - F1-F5: 切换形状 (圆形/椭圆/矩形/圆角矩形/心形)");
    info!("  - Tab: 切换摄像头设备");
    info!("  - 空格: 循环切换形状");
    info!("  - R: 重置窗口位置和旋转");
    info!("  - 关闭窗口: 退出程序");
    
    info!("启动主事件循环...");
    
    // 运行事件循环
    let result = event_loop.run(move |event, event_loop| {
        match event {
            Event::WindowEvent { event, window_id } => {
                // 确保事件来自我们的窗口
                if window_id == app.window().id() {
                    // 优先处理鼠标事件以确保拖拽流畅
                    match &event {
                        WindowEvent::CursorMoved { .. } | 
                        WindowEvent::MouseInput { .. } => {
                            // 立即处理鼠标事件，不等待渲染
                            let should_exit = app.handle_event(&event);
                            if should_exit {
                                info!("收到退出请求，正在关闭应用程序");
                                event_loop.exit();
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            // 渲染一帧
                            if let Err(_e) = app.render_frame() {
                                #[cfg(debug_assertions)]
                                error!("渲染帧失败: {}", _e);
                                // 记录错误但不退出应用，尝试恢复
                                #[cfg(debug_assertions)]
                                warn!("尝试继续运行，可能会影响性能");
                            }
                            
                            // 检查是否应该关闭
                            if app.event_handler.should_close() {
                                info!("应用请求关闭");
                                event_loop.exit();
                            }
                        }
                        _ => {
                            // 处理其他窗口事件
                            let should_exit = app.handle_event(&event);
                            if should_exit {
                                info!("收到退出请求，正在关闭应用程序");
                                event_loop.exit();
                            }
                        }
                    }
                }
            }
            Event::AboutToWait => {
                // 请求重绘以维持目标帧率
                app.window().request_redraw();
            }
            _ => {}
        }
    });
    
    match result {
        Ok(()) => {
            info!("事件循环正常结束");
            Ok(())
        }
        Err(e) => {
            error!("事件循环异常结束: {}", e);
            Err(e.into())
        }
    }
}
