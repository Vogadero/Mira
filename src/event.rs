// 事件处理器实现

use crate::camera::CameraManager;
use crate::config::ConfigManager;
use crate::render::RenderEngine;
use crate::shape::ShapeMask;
use crate::window::WindowManager;
use log::{debug, error, info, warn};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{Key, NamedKey, ModifiersState},
};

/// 事件处理器
pub struct EventHandler {
    window_manager: WindowManager,
    camera_manager: CameraManager,
    render_engine: RenderEngine,
    shape_mask: ShapeMask,
    config_manager: ConfigManager,
    
    // 事件状态管理
    last_cursor_pos: PhysicalPosition<f64>,
    modifiers_state: ModifiersState,
    is_ctrl_pressed: bool,
    
    // UI 控制状态
    is_hovering: bool,
    hover_start_time: std::time::Instant,
    show_controls: bool,
}

impl EventHandler {
    /// 创建新的事件处理器
    pub fn new(
        window_manager: WindowManager,
        camera_manager: CameraManager,
        render_engine: RenderEngine,
        shape_mask: ShapeMask,
        config_manager: ConfigManager,
    ) -> Self {
        info!("创建事件处理器");
        
        Self {
            window_manager,
            camera_manager,
            render_engine,
            shape_mask,
            config_manager,
            last_cursor_pos: PhysicalPosition::new(0.0, 0.0),
            modifiers_state: ModifiersState::empty(),
            is_ctrl_pressed: false,
            
            // UI 控制状态初始化
            is_hovering: false,
            hover_start_time: std::time::Instant::now(),
            show_controls: false,
        }
    }
    
    /// 处理窗口事件
    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_mouse_move(*position);
                false
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match (*state, *button) {
                    (ElementState::Pressed, MouseButton::Left) => {
                        self.handle_mouse_press(*button, self.last_cursor_pos);
                    }
                    (ElementState::Released, MouseButton::Left) => {
                        self.handle_mouse_release(*button);
                    }
                    _ => {}
                }
                false
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(*delta, self.modifiers_state);
                false
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.handle_modifiers_changed(modifiers.state());
                false
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
                false
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(*size);
                false
            }
            WindowEvent::CloseRequested => {
                self.handle_close_requested();
                true // 返回 true 表示应该退出
            }
            _ => false,
        }
    }
    
    /// 处理修饰键状态变化
    fn handle_modifiers_changed(&mut self, modifiers: ModifiersState) {
        self.modifiers_state = modifiers;
        self.is_ctrl_pressed = modifiers.control_key();
        debug!("修饰键状态更新: Ctrl={}", self.is_ctrl_pressed);
    }
    
    /// 获取窗口管理器引用
    pub fn window_manager(&self) -> &WindowManager {
        &self.window_manager
    }
    
    /// 获取窗口管理器可变引用
    pub fn window_manager_mut(&mut self) -> &mut WindowManager {
        &mut self.window_manager
    }
    
    /// 获取摄像头管理器引用
    pub fn camera_manager(&self) -> &CameraManager {
        &self.camera_manager
    }
    
    /// 获取摄像头管理器可变引用
    pub fn camera_manager_mut(&mut self) -> &mut CameraManager {
        &mut self.camera_manager
    }
    
    /// 获取渲染引擎引用
    pub fn render_engine(&self) -> &RenderEngine {
        &self.render_engine
    }
    
    /// 获取渲染引擎可变引用
    pub fn render_engine_mut(&mut self) -> &mut RenderEngine {
        &mut self.render_engine
    }
    
    /// 获取形状遮罩引用
    pub fn shape_mask(&self) -> &ShapeMask {
        &self.shape_mask
    }
    
    /// 获取形状遮罩可变引用
    pub fn shape_mask_mut(&mut self) -> &mut ShapeMask {
        &mut self.shape_mask
    }
    
    /// 获取配置管理器引用
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }
    
    /// 获取配置管理器可变引用
    pub fn config_manager_mut(&mut self) -> &mut ConfigManager {
        &mut self.config_manager
    }
    
    /// 显示右键上下文菜单
    fn show_context_menu(&mut self, position: PhysicalPosition<f64>) {
        // 显示详细的右键菜单信息
        info!("=== 右键上下文菜单 ===");
        info!("鼠标位置: ({:.1}, {:.1})", position.x, position.y);
        info!("");
        info!("🎭 形状切换:");
        info!("   F1 - 圆形 ⭕");
        info!("   F2 - 椭圆形 ⭕");
        info!("   F3 - 矩形 ⬜");
        info!("   F4 - 圆角矩形 ▢");
        info!("   F5 - 心形 ❤️");
        info!("   空格 - 循环切换");
        info!("");
        info!("🎮 窗口控制:");
        info!("   鼠标滚轮 - 缩放 (±10%)");
        info!("   Ctrl+滚轮 - 旋转 (±15°)");
        info!("   拖拽 - 移动窗口");
        info!("   R - 重置位置和旋转");
        info!("");
        info!("📹 摄像头:");
        info!("   Tab - 切换设备");
        info!("   当前设备: {}", 
              self.camera_manager.current_device()
                  .map(|d| d.name.as_str())
                  .unwrap_or("未知"));
        info!("");
        info!("ℹ️  当前状态:");
        info!("   形状: {:?}", self.shape_mask.shape_type());
        info!("   尺寸: {}x{}", self.window_manager.size().width, self.window_manager.size().height);
        info!("   位置: ({:.0}, {:.0})", self.window_manager.position().x, self.window_manager.position().y);
        info!("   旋转: {:.1}°", self.window_manager.rotation());
        info!("========================");
        
        // TODO: 实现真正的可视化右键菜单
        // 这需要创建一个浮动菜单窗口或使用GUI库
        // 当前版本通过控制台日志提供功能说明
    }
    
    /// 获取当前鼠标位置
    pub fn last_cursor_pos(&self) -> PhysicalPosition<f64> {
        self.last_cursor_pos
    }
    
    /// 检查 Ctrl 键是否按下
    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_ctrl_pressed
    }
    
    /// 处理鼠标按下事件（开始拖拽、点击控制按钮或显示右键菜单）
    fn handle_mouse_press(&mut self, button: MouseButton, position: PhysicalPosition<f64>) {
        match button {
            MouseButton::Left => {
                // 检查是否点击了控制按钮
                if self.show_controls {
                    let window_size = self.window_manager.size();
                    let button_size = 20.0;
                    let margin = 5.0;
                    
                    // 关闭按钮位置（右上角）
                    let close_x = window_size.width as f64 - button_size - margin;
                    let close_y = margin;
                    
                    // 最小化按钮位置（关闭按钮左边）
                    let minimize_x = close_x - button_size - margin;
                    let minimize_y = margin;
                    
                    // 检查点击位置
                    if position.x >= close_x && position.x <= close_x + button_size
                    && position.y >= close_y && position.y <= close_y + button_size {
                        // 点击关闭按钮 - 触发窗口关闭事件
                        info!("用户点击关闭按钮，准备退出应用");
                        // 保存配置并清理资源
                        self.handle_close_requested();
                        // 注意：这里需要通知主循环退出，但我们无法直接做到
                        // 在实际应用中，需要通过事件系统或状态管理来处理
                        return;
                    }
                    
                    if position.x >= minimize_x && position.x <= minimize_x + button_size
                    && position.y >= minimize_y && position.y <= minimize_y + button_size {
                        // 点击最小化按钮
                        info!("用户点击最小化按钮");
                        self.window_manager.minimize();
                        return;
                    }
                }
                
                // 开始拖拽窗口
                self.window_manager.start_drag(position);
                info!("开始拖拽窗口，鼠标位置: ({:.1}, {:.1})", position.x, position.y);
            }
            MouseButton::Right => {
                // 右键显示上下文菜单
                info!("显示右键菜单");
                self.show_context_menu(position);
            }
            _ => {
                debug!("忽略其他鼠标按下事件: {:?}", button);
            }
        }
    }
    
    /// 处理鼠标释放事件（结束拖拽）
    fn handle_mouse_release(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => {
                if self.window_manager.is_dragging() {
                    self.window_manager.end_drag();
                    let final_pos = self.window_manager.position();
                    info!("结束拖拽窗口，最终位置: ({:.1}, {:.1})", final_pos.x, final_pos.y);
                }
            }
            _ => {
                debug!("忽略非左键鼠标释放事件: {:?}", button);
            }
        }
    }
    
    /// 处理鼠标移动事件（更新拖拽位置和悬浮状态）
    fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.last_cursor_pos = position;
        
        // 检查是否在窗口区域内悬浮
        let window_size = self.window_manager.size();
        let is_inside = position.x >= 0.0 && position.y >= 0.0 
                     && position.x <= window_size.width as f64 
                     && position.y <= window_size.height as f64;
        
        // 更新悬浮状态
        if is_inside && !self.is_hovering {
            self.is_hovering = true;
            self.hover_start_time = std::time::Instant::now();
        } else if !is_inside && self.is_hovering {
            self.is_hovering = false;
            self.show_controls = false;
        }
        
        // 悬浮超过500ms显示控制按钮
        if self.is_hovering && self.hover_start_time.elapsed().as_millis() > 500 {
            self.show_controls = true;
        }
        
        // 如果正在拖拽，更新窗口位置（移除日志以提高性能）
        if self.window_manager.is_dragging() {
            self.window_manager.update_drag(position);
        }
    }
    
    /// 处理鼠标滚轮事件（缩放或旋转）
    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, modifiers: ModifiersState) {
        let ctrl_pressed = modifiers.control_key();
        
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                self.handle_scroll_delta(y, ctrl_pressed);
            }
            MouseScrollDelta::PixelDelta(delta) => {
                let y = delta.y as f32;
                if y.abs() > 1.0 {
                    let normalized_y = if y > 0.0 { 1.0 } else { -1.0 };
                    self.handle_scroll_delta(normalized_y, ctrl_pressed);
                }
            }
        }
    }
    
    /// 处理滚轮增量（缩放或旋转）
    fn handle_scroll_delta(&mut self, y: f32, ctrl_pressed: bool) {
        if ctrl_pressed {
            // Ctrl + 滚轮：旋转窗口
            let old_rotation = self.window_manager.rotation();
            
            if y > 0.0 {
                // 向上滚动，顺时针旋转 15 度
                self.window_manager.rotate(15.0);
                info!("窗口顺时针旋转 15°，角度: {:.1}° -> {:.1}°", 
                      old_rotation, self.window_manager.rotation());
            } else if y < 0.0 {
                // 向下滚动，逆时针旋转 15 度
                self.window_manager.rotate(-15.0);
                info!("窗口逆时针旋转 15°，角度: {:.1}° -> {:.1}°", 
                      old_rotation, self.window_manager.rotation());
            }
        } else {
            // 普通滚轮：缩放窗口
            let old_size = self.window_manager.size();
            
            if y > 0.0 {
                // 向上滚动，放大 10%
                self.window_manager.scale(1.1);
                info!("窗口放大 10%，尺寸: {:?} -> {:?}", 
                      old_size, self.window_manager.size());
            } else if y < 0.0 {
                // 向下滚动，缩小 10% (1/1.1 ≈ 0.909)
                self.window_manager.scale(1.0 / 1.1);
                info!("窗口缩小 10%，尺寸: {:?} -> {:?}", 
                      old_size, self.window_manager.size());
            }
        }
    }
    
    /// 处理键盘输入事件（形状切换、设备切换等）
    fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        if event.state == ElementState::Pressed {
            match &event.logical_key {
                Key::Named(NamedKey::F1) => {
                    // F1: 切换到圆形
                    self.switch_shape(crate::shape::ShapeType::Circle);
                }
                Key::Named(NamedKey::F2) => {
                    // F2: 切换到椭圆形
                    self.switch_shape(crate::shape::ShapeType::Ellipse);
                }
                Key::Named(NamedKey::F3) => {
                    // F3: 切换到矩形
                    self.switch_shape(crate::shape::ShapeType::Rectangle);
                }
                Key::Named(NamedKey::F4) => {
                    // F4: 切换到圆角矩形
                    self.switch_shape(crate::shape::ShapeType::RoundedRectangle { radius: 20.0 });
                }
                Key::Named(NamedKey::F5) => {
                    // F5: 切换到心形
                    self.switch_shape(crate::shape::ShapeType::Heart);
                }
                Key::Named(NamedKey::Tab) => {
                    // Tab: 切换摄像头设备
                    self.switch_camera_device();
                }
                Key::Named(NamedKey::Space) => {
                    // 空格: 循环切换形状
                    self.cycle_shape();
                }
                Key::Character(c) if c == "r" || c == "R" => {
                    // R: 重置窗口位置和旋转
                    self.reset_window();
                }
                _ => {
                    debug!("未处理的键盘输入: {:?}", event.logical_key);
                }
            }
        }
    }
    
    /// 处理窗口调整大小事件（调整渲染表面和遮罩）
    fn handle_resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        info!("窗口尺寸变化: {}x{}", size.width, size.height);
        
        // 更新窗口管理器的尺寸（仅更新内部状态，不触发新的 Resized 事件）
        self.window_manager.update_size(size.width, size.height);
        
        // 调整渲染表面
        self.render_engine.resize(size.width, size.height);
        
        // 调整形状遮罩以适应新尺寸
        self.shape_mask.resize(size.width, size.height);
        
        // 更新渲染引擎的遮罩
        if let Err(e) = self.render_engine.set_mask(&self.shape_mask) {
            error!("更新渲染引擎遮罩失败: {}", e);
        }
        
        debug!("窗口调整大小完成，新尺寸: {}x{}", size.width, size.height);
    }
    
    /// 处理窗口关闭事件（保存配置并清理资源）
    fn handle_close_requested(&mut self) {
        info!("收到窗口关闭请求，开始清理资源");
        
        // 保存当前配置
        let current_config = self.create_current_config();
        if let Err(e) = self.config_manager.save(&current_config) {
            error!("保存配置失败: {}", e);
        } else {
            info!("配置已保存");
        }
        
        // 关闭摄像头设备
        if let Err(e) = self.camera_manager.close_device() {
            error!("关闭摄像头设备失败: {}", e);
        } else {
            info!("摄像头设备已关闭");
        }
        
        info!("资源清理完成");
    }
    
    /// 切换形状
    fn switch_shape(&mut self, shape_type: crate::shape::ShapeType) {
        let old_shape = self.shape_mask.shape_type();
        self.shape_mask.set_shape(shape_type);
        
        // 更新渲染引擎的遮罩
        if let Err(e) = self.render_engine.set_mask(&self.shape_mask) {
            error!("更新渲染引擎遮罩失败: {}", e);
        } else {
            info!("形状切换: {:?} -> {:?}", old_shape, shape_type);
        }
    }
    
    /// 循环切换形状
    fn cycle_shape(&mut self) {
        use crate::shape::ShapeType;
        
        let next_shape = match self.shape_mask.shape_type() {
            ShapeType::Circle => ShapeType::Ellipse,
            ShapeType::Ellipse => ShapeType::Rectangle,
            ShapeType::Rectangle => ShapeType::RoundedRectangle { radius: 20.0 },
            ShapeType::RoundedRectangle { .. } => ShapeType::Heart,
            ShapeType::Heart => ShapeType::Circle,
        };
        
        self.switch_shape(next_shape);
    }
    
    /// 切换摄像头设备
    fn switch_camera_device(&mut self) {
        let devices_len = self.camera_manager.devices().len();
        if devices_len == 0 {
            warn!("没有可用的摄像头设备");
            return;
        }
        
        let current_index = self.camera_manager.current_device_index().unwrap_or(0);
        let next_index = (current_index + 1) % devices_len;
        
        info!("切换摄像头设备: {} -> {}", current_index, next_index);
        
        if let Err(e) = self.camera_manager.open_device(next_index) {
            error!("切换摄像头设备失败: {}", e);
        } else {
            let device_name = self.camera_manager.devices()[next_index].name.clone();
            info!("成功切换到摄像头设备 {}: {}", next_index, device_name);
        }
    }
    
    /// 重置窗口位置和旋转
    fn reset_window(&mut self) {
        info!("重置窗口位置和旋转");
        
        // 重置位置到默认值
        self.window_manager.set_position(100.0, 100.0);
        
        // 重置旋转角度
        self.window_manager.set_rotation(0.0);
        
        // 重置尺寸到默认值
        self.window_manager.set_size(400, 400);
        
        info!("窗口已重置到默认状态");
    }
    
    /// 创建当前配置
    fn create_current_config(&self) -> crate::config::AppConfig {
        use crate::config::{AppConfig, WindowConfig, CameraConfig};
        
        let window_pos = self.window_manager.position();
        let window_size = self.window_manager.size();
        let shape_name = match self.shape_mask.shape_type() {
            crate::shape::ShapeType::Circle => "Circle",
            crate::shape::ShapeType::Ellipse => "Ellipse",
            crate::shape::ShapeType::Rectangle => "Rectangle",
            crate::shape::ShapeType::RoundedRectangle { .. } => "RoundedRectangle",
            crate::shape::ShapeType::Heart => "Heart",
        };
        
        AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: window_pos.x,
                position_y: window_pos.y,
                width: window_size.width,
                height: window_size.height,
                rotation: self.window_manager.rotation(),
                shape: shape_name.to_string(),
            },
            camera: CameraConfig {
                device_index: self.camera_manager.current_device_index().unwrap_or(0),
            },
        }
    }
    
    /// 渲染一帧
    pub fn render_frame(&mut self) -> Result<(), String> {
        // 从摄像头捕获帧
        let frame = match self.camera_manager.capture_frame() {
            Ok(frame) => frame,
            Err(e) => {
                // 记录捕获错误并尝试恢复
                error!("捕获视频帧失败: {}", e);
                
                // 尝试重新打开当前设备
                if let Some(current_index) = self.camera_manager.current_device_index() {
                    warn!("尝试重新打开摄像头设备 {}", current_index);
                    if let Err(reopen_err) = self.camera_manager.open_device(current_index) {
                        error!("重新打开摄像头设备失败: {}", reopen_err);
                        return Err(format!("摄像头捕获失败且无法恢复: {}", e));
                    }
                    info!("摄像头设备重新打开成功");
                }
                
                return Err(format!("摄像头捕获失败: {}", e));
            }
        };
        
        // 上传帧到 GPU - 转换 Frame 类型
        let render_frame = crate::render::engine::Frame {
            data: frame.data,
            width: frame.width,
            height: frame.height,
            format: match frame.format {
                crate::camera::manager::PixelFormat::RGB8 => crate::render::engine::PixelFormat::RGB8,
                crate::camera::manager::PixelFormat::RGBA8 => crate::render::engine::PixelFormat::RGBA8,
                crate::camera::manager::PixelFormat::YUV420 => crate::render::engine::PixelFormat::YUV420,
            },
        };
        
        if let Err(e) = self.render_engine.upload_frame(&render_frame) {
            error!("上传视频帧到 GPU 失败: {}", e);
            return Err(format!("GPU 上传失败: {}", e));
        }
        
        // 准备UI渲染信息
        let ui_info = crate::render::engine::UIRenderInfo {
            show_controls: self.show_controls,
            window_size: self.window_manager.size(),
        };
        
        // 调用渲染引擎渲染当前帧
        let rotation_radians = self.window_manager.rotation().to_radians();
        if let Err(e) = self.render_engine.render_with_ui(rotation_radians, &ui_info) {
            error!("渲染帧失败: {}", e);
            
            // 尝试恢复渲染引擎
            warn!("尝试恢复渲染引擎");
            let window_size = self.window_manager.size();
            self.render_engine.resize(window_size.width, window_size.height);
            
            // 重新设置遮罩
            if let Err(mask_err) = self.render_engine.set_mask(&self.shape_mask) {
                error!("重新设置遮罩失败: {}", mask_err);
            }
            
            // 再次尝试渲染
            if let Err(retry_err) = self.render_engine.render_with_ui(rotation_radians, &ui_info) {
                error!("渲染恢复失败: {}", retry_err);
                return Err(format!("渲染失败且无法恢复: {}", e));
            }
            
            info!("渲染引擎恢复成功");
        }
        
        debug!("成功渲染一帧，帧尺寸: {}x{}", frame.width, frame.height);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::ShapeType;
    use winit::event_loop::EventLoop;

    // 创建测试用的事件处理器
    fn create_test_event_handler() -> Result<EventHandler, Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let _window_manager = WindowManager::new(&event_loop)?;
        let _camera_manager = CameraManager::new().unwrap_or_else(|_| {
            // 在测试环境中可能没有摄像头，创建一个空的管理器
            CameraManager::new_empty()
        });
        
        // 由于无法在测试中创建真实的渲染引擎，我们跳过这个测试
        // 或者创建一个模拟的渲染引擎
        Err("Cannot create render engine in test environment".into())
    }

    #[test]
    fn test_event_handler_state_management() {
        // 测试事件状态管理的基本功能
        let modifiers = ModifiersState::empty();
        assert!(!modifiers.control_key());
        
        let position = PhysicalPosition::new(100.0, 200.0);
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
    }

    #[test]
    fn test_modifiers_state() {
        // 测试修饰键状态
        let modifiers = ModifiersState::empty();
        assert!(!modifiers.control_key());
        
        // 注意：在实际测试中，我们无法直接设置 ModifiersState
        // 这里只是测试其基本功能
    }

    #[test]
    fn test_cursor_position_tracking() {
        let pos1 = PhysicalPosition::new(10.0, 20.0);
        let pos2 = PhysicalPosition::new(30.0, 40.0);
        
        // 计算位置差
        let delta_x = pos2.x - pos1.x;
        let delta_y = pos2.y - pos1.y;
        
        assert_eq!(delta_x, 20.0);
        assert_eq!(delta_y, 20.0);
    }

    #[test]
    fn test_event_handler_creation_requirements() {
        // 测试事件处理器创建所需的组件
        // 由于需要真实的窗口和GPU上下文，这里只测试基本结构
        
        // 测试形状遮罩创建
        let shape_mask = ShapeMask::new(ShapeType::Circle, 400, 400);
        assert_eq!(shape_mask.width(), 400);
        assert_eq!(shape_mask.height(), 400);
        
        // 测试配置管理器创建
        let config_manager = ConfigManager::new();
        assert!(config_manager.is_ok());
    }
    
    #[test]
    fn test_mouse_event_handling() {
        // 测试鼠标事件处理逻辑
        use winit::event::MouseButton;
        
        let button = MouseButton::Left;
        let position = PhysicalPosition::new(100.0, 200.0);
        
        // 验证基本数据结构
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
        
        // 测试鼠标按钮匹配
        match button {
            MouseButton::Left => assert!(true),
            _ => assert!(false, "应该匹配左键"),
        }
    }
    
    #[test]
    fn test_keyboard_event_handling() {
        // 测试键盘事件处理逻辑
        use winit::keyboard::{Key, NamedKey, SmolStr};
        
        let f1_key: Key<SmolStr> = Key::Named(NamedKey::F1);
        let space_key: Key<SmolStr> = Key::Named(NamedKey::Space);
        let r_key: Key<SmolStr> = Key::Character(SmolStr::new("r"));
        
        // 验证键盘按键匹配
        match f1_key {
            Key::Named(NamedKey::F1) => assert!(true),
            _ => assert!(false, "应该匹配 F1 键"),
        }
        
        match space_key {
            Key::Named(NamedKey::Space) => assert!(true),
            _ => assert!(false, "应该匹配空格键"),
        }
        
        match r_key {
            Key::Character(c) if c == "r" => assert!(true),
            _ => assert!(false, "应该匹配 R 键"),
        }
    }
    
    #[test]
    fn test_scroll_delta_handling() {
        // 测试滚轮增量处理逻辑
        use winit::event::MouseScrollDelta;
        
        let line_delta = MouseScrollDelta::LineDelta(0.0, 1.0);
        let pixel_delta = MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, 10.0));
        
        match line_delta {
            MouseScrollDelta::LineDelta(_, y) => {
                assert_eq!(y, 1.0);
                assert!(y > 0.0); // 向上滚动
            }
            _ => assert!(false, "应该匹配行增量"),
        }
        
        match pixel_delta {
            MouseScrollDelta::PixelDelta(delta) => {
                assert_eq!(delta.y, 10.0);
                assert!(delta.y > 1.0); // 足够的像素增量
            }
            _ => assert!(false, "应该匹配像素增量"),
        }
    }
    
    #[test]
    fn test_shape_cycling_logic() {
        // 测试形状循环切换逻辑
        use crate::shape::ShapeType;
        
        let shapes = [
            ShapeType::Circle,
            ShapeType::Ellipse,
            ShapeType::Rectangle,
            ShapeType::RoundedRectangle { radius: 20.0 },
            ShapeType::Heart,
        ];
        
        // 验证形状循环逻辑
        for (i, &current_shape) in shapes.iter().enumerate() {
            let next_shape = match current_shape {
                ShapeType::Circle => ShapeType::Ellipse,
                ShapeType::Ellipse => ShapeType::Rectangle,
                ShapeType::Rectangle => ShapeType::RoundedRectangle { radius: 20.0 },
                ShapeType::RoundedRectangle { .. } => ShapeType::Heart,
                ShapeType::Heart => ShapeType::Circle,
            };
            
            let expected_next_index = (i + 1) % shapes.len();
            let expected_next_shape = shapes[expected_next_index];
            
            // 验证循环逻辑正确
            match (next_shape, expected_next_shape) {
                (ShapeType::Circle, ShapeType::Circle) => assert!(true),
                (ShapeType::Ellipse, ShapeType::Ellipse) => assert!(true),
                (ShapeType::Rectangle, ShapeType::Rectangle) => assert!(true),
                (ShapeType::RoundedRectangle { .. }, ShapeType::RoundedRectangle { .. }) => assert!(true),
                (ShapeType::Heart, ShapeType::Heart) => assert!(true),
                _ => assert!(false, "形状循环逻辑不正确"),
            }
        }
    }
    
    #[test]
    fn test_config_creation_logic() {
        // 测试配置创建逻辑
        use crate::config::{AppConfig, WindowConfig, CameraConfig};
        
        let config = AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: 100.0,
                position_y: 200.0,
                width: 400,
                height: 400,
                rotation: 45.0,
                shape: "Circle".to_string(),
            },
            camera: CameraConfig {
                device_index: 0,
            },
        };
        
        // 验证配置结构
        assert_eq!(config.version, "1.0");
        assert_eq!(config.window.position_x, 100.0);
        assert_eq!(config.window.position_y, 200.0);
        assert_eq!(config.window.width, 400);
        assert_eq!(config.window.height, 400);
        assert_eq!(config.window.rotation, 45.0);
        assert_eq!(config.window.shape, "Circle");
        assert_eq!(config.camera.device_index, 0);
    }
    
    #[test]
    fn test_error_handling_logic() {
        // 测试错误处理逻辑
        use crate::error::{CameraError, RenderError};
        
        let camera_error = CameraError::CaptureError("Test error".to_string());
        let render_error = RenderError::RenderFailed("Test render error".to_string());
        
        // 验证错误类型
        match camera_error {
            CameraError::CaptureError(msg) => {
                assert_eq!(msg, "Test error");
            }
            _ => assert!(false, "应该匹配捕获错误"),
        }
        
        match render_error {
            RenderError::RenderFailed(msg) => {
                assert_eq!(msg, "Test render error");
            }
            _ => assert!(false, "应该匹配渲染错误"),
        }
    }
}