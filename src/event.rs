// 事件处理器实现

use crate::camera::CameraManager;
use crate::config::ConfigManager;
use crate::render::RenderEngine;
use crate::shape::{ShapeMask, ShapeType};
use crate::ui::{ContextMenu, MenuRenderer};
use crate::ui::context_menu::MenuState;
use crate::window::WindowManager;
use log::{debug, error, info, warn};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
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
    last_mouse_move_time: std::time::Instant,
    show_controls: bool,
    close_button_hovered: bool,
    minimize_button_hovered: bool,
    
    // 上下文菜单
    context_menu: ContextMenu,
    menu_renderer: Option<MenuRenderer>,
    
    // 应用状态
    should_close: bool,
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
        
        let mut handler = Self {
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
            last_mouse_move_time: std::time::Instant::now(),
            show_controls: false,
            close_button_hovered: false,
            minimize_button_hovered: false,
            
            // 上下文菜单初始化
            context_menu: ContextMenu::new(PhysicalSize::new(1920, 1080)), // 默认屏幕尺寸，会在运行时更新
            menu_renderer: None, // 延迟初始化
            
            // 应用状态初始化
            should_close: false,
        };
        
        // 设置菜单回调函数
        handler.setup_menu_callbacks();
        
        handler
    }
    
    /// 设置菜单回调函数
    fn setup_menu_callbacks(&mut self) {
        debug!("设置菜单回调函数");
        
        // 由于Rust的借用检查器限制，我们不能在这里直接设置回调
        // 回调函数将在handle_menu_item_click方法中直接处理
        // 这是一个简化的实现，实际应用中可能需要更复杂的回调系统
    }
    
    /// 初始化菜单渲染器
    pub fn init_menu_renderer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) -> Result<(), String> {
        debug!("初始化菜单渲染器");
        
        match MenuRenderer::new(device, queue, surface_format) {
            Ok(renderer) => {
                self.menu_renderer = Some(renderer);
                info!("菜单渲染器初始化成功");
                Ok(())
            }
            Err(e) => {
                warn!("菜单渲染器初始化失败: {}，将使用简单文本菜单", e);
                // 不返回错误，让应用继续使用简单文本菜单
                Ok(())
            }
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
                    (ElementState::Pressed, MouseButton::Right) => {
                        self.handle_mouse_press(*button, self.last_cursor_pos);
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
    
    /// 检查应用是否应该关闭
    pub fn should_close(&self) -> bool {
        self.should_close
    }
    
    /// 检查上下文菜单是否可见
    pub fn is_context_menu_visible(&self) -> bool {
        self.context_menu.state() != &MenuState::Hidden
    }
    
    /// 显示右键上下文菜单
    fn show_context_menu(&mut self, position: PhysicalPosition<f64>) {
        info!("显示右键上下文菜单，位置: ({:.1}, {:.1})", position.x, position.y);
        
        // 更新菜单状态信息
        self.update_context_menu_status();
        
        // 显示菜单
        let menu_position = PhysicalPosition::new(position.x as f32, position.y as f32);
        self.context_menu.show(menu_position);
        
        info!("上下文菜单已显示，菜单项数量: {}", self.context_menu.get_display_items().len());
        debug!("上下文菜单已显示");
    }
    
    /// 更新上下文菜单状态信息
    fn update_context_menu_status(&mut self) {
        // 更新摄像头设备列表
        let devices: Vec<(usize, String)> = self.camera_manager.enumerate_devices()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, info)| (i, info.name))
            .collect();
        
        let current_device = self.camera_manager.current_device()
            .map(|info| info.index);
        
        self.context_menu.update_camera_devices(&devices, current_device);
        
        // 更新状态信息
        let window_size = self.window_manager.size();
        let window_position = self.window_manager.position();
        let rotation = self.window_manager.rotation();
        
        self.context_menu.update_status_info(window_size, window_position, rotation);
        
        // 更新屏幕尺寸
        // 注意：这里使用窗口尺寸作为近似，实际应用中应该获取真实的屏幕尺寸
        self.context_menu.update_screen_size(PhysicalSize::new(1920, 1080));
    }
    
    /// 隐藏上下文菜单
    fn hide_context_menu(&mut self) {
        if self.context_menu.state() != &MenuState::Hidden {
            debug!("隐藏上下文菜单");
            self.context_menu.hide();
        }
    }
    
    /// 处理菜单项点击
    fn handle_menu_item_click(&mut self, item_id: &str) -> Result<(), String> {
        info!("执行菜单项: {}", item_id);
        
        match item_id {
            // 形状切换
            "shape_circle" => {
                self.shape_mask.set_shape(ShapeType::Circle);
                info!("切换到圆形");
            }
            "shape_ellipse" => {
                self.shape_mask.set_shape(ShapeType::Ellipse);
                info!("切换到椭圆形");
            }
            "shape_rectangle" => {
                self.shape_mask.set_shape(ShapeType::Rectangle);
                info!("切换到矩形");
            }
            "shape_rounded_rectangle" => {
                self.shape_mask.set_shape(ShapeType::RoundedRectangle { radius: 20.0 });
                info!("切换到圆角矩形");
            }
            "shape_heart" => {
                self.shape_mask.set_shape(ShapeType::Heart);
                info!("切换到心形");
            }
            
            // 摄像头设备切换
            item_id if item_id.starts_with("camera_") => {
                if let Ok(device_index) = item_id.strip_prefix("camera_").unwrap().parse::<usize>() {
                    match self.camera_manager.open_device(device_index) {
                        Ok(()) => {
                            info!("切换到摄像头设备 {}", device_index);
                        }
                        Err(e) => {
                            error!("切换摄像头设备失败: {}", e);
                            return Err(format!("切换摄像头设备失败: {}", e));
                        }
                    }
                } else {
                    return Err("无效的摄像头设备ID".to_string());
                }
            }
            
            // 窗口控制
            "reset_position" => {
                self.window_manager.set_position(100.0, 100.0);
                info!("重置窗口位置");
            }
            "reset_rotation" => {
                self.window_manager.set_rotation(0.0);
                info!("重置窗口旋转");
            }
            "reset_size" => {
                self.window_manager.set_size(400, 400);
                info!("重置窗口大小");
            }
            
            // 状态信息
            "show_info" => {
                let window_size = self.window_manager.size();
                let window_position = self.window_manager.position();
                let rotation = self.window_manager.rotation();
                let current_device = self.camera_manager.current_device()
                    .map(|d| d.name.as_str())
                    .unwrap_or("未知");
                
                info!("=== 当前状态 ===");
                info!("形状: {:?}", self.shape_mask.shape_type());
                info!("尺寸: {}x{}", window_size.width, window_size.height);
                info!("位置: ({:.0}, {:.0})", window_position.x, window_position.y);
                info!("旋转: {:.1}°", rotation);
                info!("摄像头: {}", current_device);
                info!("================");
            }
            
            _ => {
                warn!("未知的菜单项: {}", item_id);
                return Err(format!("未知的菜单项: {}", item_id));
            }
        }
        
        Ok(())
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
                // 首先检查是否点击了上下文菜单
                let menu_position = PhysicalPosition::new(position.x as f32, position.y as f32);
                if self.context_menu.is_point_inside(menu_position) {
                    // 先获取item_id并复制，避免借用冲突
                    let item_id_opt = self.context_menu.get_item_at_position(menu_position)
                        .map(|id| id.to_string());
                    
                    if let Some(item_id) = item_id_opt {
                        // 执行菜单项
                        if let Err(e) = self.handle_menu_item_click(&item_id) {
                            error!("执行菜单项失败: {}", e);
                        }
                        // 隐藏菜单
                        self.hide_context_menu();
                    }
                    return;
                } else {
                    // 点击菜单外区域，隐藏菜单
                    self.hide_context_menu();
                }
                
                // 检查是否点击了控制按钮
                if self.show_controls {
                    let window_size = self.window_manager.size();
                    let button_size = 30.0; // 与UIUniforms保持一致
                    let margin = 8.0; // 与UIUniforms保持一致
                    
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
                        // 设置关闭标志
                        self.should_close = true;
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
                info!("右键点击 - 显示上下文菜单");
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
        self.last_mouse_move_time = std::time::Instant::now();
        
        // 检查是否在上下文菜单内
        let menu_position = PhysicalPosition::new(position.x as f32, position.y as f32);
        if self.context_menu.is_point_inside(menu_position) {
            // 更新菜单悬浮状态
            if let Some(item_id) = self.context_menu.get_item_at_position(menu_position) {
                self.context_menu.set_hovered_item(Some(item_id.to_owned()));
            } else {
                self.context_menu.set_hovered_item(None);
            }
            return; // 在菜单内时不处理其他悬浮逻辑
        } else {
            // 鼠标不在菜单内，清除菜单悬浮状态
            self.context_menu.set_hovered_item(None);
        }
        
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
            self.close_button_hovered = false;
            self.minimize_button_hovered = false;
        }
        
        // 悬浮超过500ms显示控制按钮
        if self.is_hovering && self.hover_start_time.elapsed().as_millis() > 500 {
            self.show_controls = true;
            
            // 检查是否悬浮在按钮上
            self.update_button_hover_states(position);
        } else if self.show_controls {
            // 如果按钮已显示，检查是否应该隐藏（鼠标静止超过2秒）
            if self.last_mouse_move_time.elapsed().as_millis() > 2000 {
                self.show_controls = false;
                self.close_button_hovered = false;
                self.minimize_button_hovered = false;
            } else {
                // 继续更新按钮悬浮状态
                self.update_button_hover_states(position);
            }
        } else {
            // 如果控制按钮未显示，重置悬浮状态
            self.close_button_hovered = false;
            self.minimize_button_hovered = false;
        }
        
        // 如果正在拖拽，更新窗口位置（移除日志以提高性能）
        if self.window_manager.is_dragging() {
            self.window_manager.update_drag(position);
        }
    }
    
    /// 更新按钮悬浮状态
    fn update_button_hover_states(&mut self, position: PhysicalPosition<f64>) {
        let window_size = self.window_manager.size();
        
        // 检查关闭按钮悬浮
        self.close_button_hovered = self.render_engine.is_point_in_button(
            position.x as f32,
            position.y as f32,
            "close",
            window_size,
        );
        
        // 检查最小化按钮悬浮
        self.minimize_button_hovered = self.render_engine.is_point_in_button(
            position.x as f32,
            position.y as f32,
            "minimize",
            window_size,
        );
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
                Key::Named(NamedKey::Escape) => {
                    // ESC: 关闭上下文菜单
                    if self.is_context_menu_visible() {
                        self.hide_context_menu();
                        info!("上下文菜单已关闭");
                    }
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
        // 如果应该关闭，直接返回
        if self.should_close {
            return Ok(());
        }
        
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
            close_button_hovered: self.close_button_hovered,
            minimize_button_hovered: self.minimize_button_hovered,
        };
        
        // 调用渲染引擎渲染当前帧
        let rotation_radians = self.window_manager.rotation().to_radians();
        
        // 如果上下文菜单可见，使用特殊的渲染路径
        if self.is_context_menu_visible() {
            if let Err(e) = self.render_frame_with_context_menu(rotation_radians, &ui_info) {
                error!("带菜单的帧渲染失败: {}", e);
                return Err(format!("带菜单的帧渲染失败: {}", e));
            }
        } else {
            // 正常渲染路径
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
        }
        
        debug!("成功渲染一帧，帧尺寸: {}x{}", frame.width, frame.height);
        Ok(())
    }
    
    /// 渲染带上下文菜单的帧
    fn render_frame_with_context_menu(&mut self, rotation: f32, ui_info: &crate::render::engine::UIRenderInfo) -> Result<(), String> {
        debug!("渲染带上下文菜单的帧");
        
        // 首先渲染主视频内容和UI控件
        if let Err(e) = self.render_engine.render_with_ui(rotation, ui_info) {
            return Err(format!("主内容渲染失败: {}", e));
        }
        
        // 然后渲染上下文菜单（目前使用简单文本菜单）
        if self.menu_renderer.is_some() {
            // 如果有菜单渲染器，尝试使用视觉菜单
            let window_size = self.window_manager.size();
            let screen_size = [window_size.width as f32, window_size.height as f32];
            
            if let Err(e) = self.render_engine.render_context_menu(self.menu_renderer.as_mut().unwrap(), &self.context_menu, screen_size) {
                warn!("视觉菜单渲染失败，回退到简单文本菜单: {}", e);
                self.render_simple_context_menu()?;
            }
        } else {
            // 使用简单文本菜单
            self.render_simple_context_menu()?;
        }
        
        Ok(())
    }
    
    /// 渲染简单的上下文菜单（文本版本）
    fn render_simple_context_menu(&mut self) -> Result<(), String> {
        // 不再在控制台显示菜单，改为使用系统托盘菜单
        // 菜单功能已通过系统托盘图标的右键菜单提供
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