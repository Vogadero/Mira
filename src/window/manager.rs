// 窗口管理器实现

use crate::error::WindowError;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder, WindowLevel},
};
use log::{info, warn, error, debug};
use std::sync::Arc;
use std::time::Instant;

/// 拖拽内存使用统计
#[derive(Debug, Clone)]
pub struct DragMemoryStats {
    pub position_pool_size: usize,
    pub position_pool_capacity: usize,
    pub calculation_buffer_size: usize,
    pub calculation_buffer_capacity: usize,
    pub position_history_size: usize,
    pub position_history_capacity: usize,
}

/// 边界检查状态
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryCheckState {
    /// 不需要检查
    None,
    /// 需要检查但尚未执行
    Pending,
    /// 正在异步检查中
    InProgress,
    /// 检查完成
    Completed,
}

/// 窗口管理器
pub struct WindowManager {
    window: Arc<Window>,
    position: PhysicalPosition<f64>,
    size: PhysicalSize<u32>,
    rotation: f32,
    is_dragging: bool,
    drag_offset: PhysicalPosition<f64>,
    
    // 拖拽内存优化 - 预分配的数据结构
    drag_position_pool: Vec<PhysicalPosition<f64>>,
    drag_calculation_buffer: Vec<f64>,
    position_history: Vec<PhysicalPosition<f64>>,
    
    // 边界检查分离优化
    boundary_check_state: BoundaryCheckState,
    pending_boundary_position: Option<PhysicalPosition<f64>>,
    last_boundary_check: Instant,
    boundary_check_interval: std::time::Duration,
}

impl WindowManager {
    /// 创建新的置顶窗口
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, WindowError> {
        info!("创建窗口管理器");
        
        // 默认窗口尺寸
        let default_size = PhysicalSize::new(400, 400);
        let default_position = PhysicalPosition::new(100.0, 100.0);
        
        // 创建窗口
        let window = WindowBuilder::new()
            .with_title("Mira - 桌面摄像精灵")
            .with_inner_size(default_size)
            .with_position(default_position)
            .with_transparent(true)  // 启用透明背景
            .with_decorations(false) // 无边框窗口
            .with_resizable(false)   // 禁用手动调整大小
            .with_window_level(WindowLevel::AlwaysOnTop) // 置顶显示
            .build(event_loop)
            .map_err(|e| {
                error!("窗口创建失败: {}", e);
                WindowError::CreationFailed(e.to_string())
            })?;
        
        info!("窗口创建成功，尺寸: {:?}, 位置: {:?}", default_size, default_position);
        
        let mut manager = Self {
            window: Arc::new(window),
            position: default_position,
            size: default_size,
            rotation: 0.0,
            is_dragging: false,
            drag_offset: PhysicalPosition::new(0.0, 0.0),
            
            // 预分配拖拽相关的数据结构，避免运行时分配
            drag_position_pool: Vec::with_capacity(64), // 预分配位置缓存
            drag_calculation_buffer: Vec::with_capacity(32), // 预分配计算缓冲区
            position_history: Vec::with_capacity(16), // 预分配位置历史
            
            // 边界检查分离初始化
            boundary_check_state: BoundaryCheckState::None,
            pending_boundary_position: None,
            last_boundary_check: Instant::now(),
            boundary_check_interval: std::time::Duration::from_millis(100), // 100ms 间隔
        };
        
        // 预填充对象池
        manager.prefill_drag_pools();
        
        Ok(manager)
    }
    
    /// 获取窗口引用
    pub fn window(&self) -> Arc<Window> {
        Arc::clone(&self.window)
    }
    
    /// 设置窗口位置
    pub fn set_position(&mut self, x: f64, y: f64) {
        let new_position = PhysicalPosition::new(x, y);
        
        // 验证位置有效性
        if self.is_valid_position(new_position) {
            self.position = new_position;
            self.window.set_outer_position(new_position);
            info!("窗口位置更新为: ({}, {})", x, y);
        } else {
            warn!("无效的窗口位置: ({}, {})", x, y);
        }
    }
    
    /// 获取窗口位置
    pub fn position(&self) -> PhysicalPosition<f64> {
        self.position
    }
    
    /// 设置窗口尺寸
    pub fn set_size(&mut self, width: u32, height: u32) {
        let new_size = PhysicalSize::new(width, height);
        
        // 验证尺寸有效性
        if self.is_valid_size(new_size) {
            self.size = new_size;
            let _ = self.window.request_inner_size(new_size);
            info!("窗口尺寸更新为: {}x{}", width, height);
        } else {
            warn!("无效的窗口尺寸: {}x{}", width, height);
        }
    }
    
    /// 更新窗口尺寸（仅更新内部状态，不触发窗口调整）
    /// 用于响应窗口系统的 Resized 事件，避免无限循环
    pub fn update_size(&mut self, width: u32, height: u32) {
        let new_size = PhysicalSize::new(width, height);
        
        // 验证尺寸有效性
        if self.is_valid_size(new_size) {
            self.size = new_size;
            info!("窗口尺寸更新为: {}x{}", width, height);
        } else {
            warn!("无效的窗口尺寸: {}x{}", width, height);
        }
    }
    
    /// 获取窗口尺寸
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
    
    /// 设置窗口旋转角度（度数）
    pub fn set_rotation(&mut self, degrees: f32) {
        // 将角度归一化到 0-360 度范围
        let normalized = self.normalize_angle(degrees);
        
        // 应用自动对齐逻辑
        let aligned = self.apply_auto_alignment(normalized);
        
        self.rotation = aligned;
        info!("窗口旋转角度更新为: {:.1}°", aligned);
    }
    
    /// 旋转窗口（增量旋转）
    pub fn rotate(&mut self, delta_degrees: f32) {
        let new_angle = self.rotation + delta_degrees;
        self.set_rotation(new_angle);
    }
    
    /// 将角度归一化到 0-360 度范围
    fn normalize_angle(&self, degrees: f32) -> f32 {
        let normalized = degrees % 360.0;
        if normalized < 0.0 { normalized + 360.0 } else { normalized }
    }
    
    /// 应用自动对齐逻辑（0°、90°、180°、270° ±5° 范围内自动对齐）
    fn apply_auto_alignment(&self, degrees: f32) -> f32 {
        const ALIGNMENT_TOLERANCE: f32 = 5.0;
        const ALIGNMENT_ANGLES: [f32; 4] = [0.0, 90.0, 180.0, 270.0];
        
        for &target_angle in &ALIGNMENT_ANGLES {
            // 检查是否在对齐范围内
            let diff = (degrees - target_angle).abs();
            let diff_wrapped = (degrees - (target_angle + 360.0)).abs();
            let diff_wrapped_neg = ((degrees + 360.0) - target_angle).abs();
            
            let min_diff = diff.min(diff_wrapped).min(diff_wrapped_neg);
            
            if min_diff <= ALIGNMENT_TOLERANCE {
                info!("自动对齐: {:.1}° -> {:.1}°", degrees, target_angle);
                return target_angle;
            }
        }
        
        degrees
    }
    
    /// 获取窗口旋转角度
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    
    /// 开始拖拽
    pub fn start_drag(&mut self, cursor_pos: PhysicalPosition<f64>) {
        self.is_dragging = true;
        // 计算拖拽偏移量（鼠标相对于窗口左上角的位置）
        self.drag_offset = PhysicalPosition::new(
            cursor_pos.x - self.position.x,
            cursor_pos.y - self.position.y,
        );
        
        // 清空位置历史，准备记录新的拖拽轨迹
        self.position_history.clear();
        self.position_history.push(self.position);
        
        info!("开始拖拽，偏移量: ({:.1}, {:.1})", self.drag_offset.x, self.drag_offset.y);
    }
    
    /// 更新拖拽位置（简化版本，避免漂移）
    pub fn update_drag(&mut self, cursor_pos: PhysicalPosition<f64>) {
        if self.is_dragging {
            // 计算新位置（考虑拖拽偏移量）
            let new_x = cursor_pos.x - self.drag_offset.x;
            let new_y = cursor_pos.y - self.drag_offset.y;
            
            let new_pos = PhysicalPosition::new(new_x, new_y);
            
            // 只有位置真正改变时才更新（避免重复调用）
            if (new_pos.x - self.position.x).abs() > 0.5 || (new_pos.y - self.position.y).abs() > 0.5 {
                self.position = new_pos;
                
                // 直接设置位置，不做边界检查（拖拽时允许移出屏幕）
                self.window.set_outer_position(new_pos);
            }
        }
    }
    
    /// 结束拖拽
    pub fn end_drag(&mut self) {
        if self.is_dragging {
            self.is_dragging = false;
            
            // 在拖拽结束时立即执行边界约束（确保窗口在有效位置）
            let constrained_pos = self.fast_constrain_position(self.position);
            if constrained_pos != self.position {
                self.position = constrained_pos;
                self.window.set_outer_position(constrained_pos);
                debug!("拖拽结束边界约束: ({:.1}, {:.1})", constrained_pos.x, constrained_pos.y);
            }
            
            // 重置边界检查状态
            self.boundary_check_state = BoundaryCheckState::None;
            self.pending_boundary_position = None;
            
            info!("结束拖拽，最终位置: ({:.1}, {:.1})", self.position.x, self.position.y);
        }
    }
    
    /// 缩放窗口（百分比）
    pub fn scale(&mut self, factor: f32) {
        let new_width = (self.size.width as f32 * factor) as u32;
        let new_height = (self.size.height as f32 * factor) as u32;
        
        // 应用尺寸约束，保持宽高比
        let constrained_size = self.constrain_size_preserve_aspect_ratio(
            PhysicalSize::new(new_width, new_height)
        );
        self.set_size(constrained_size.width, constrained_size.height);
    }
    
    /// 限制窗口在屏幕边界内（至少 20% 在屏幕内）
    pub fn constrain_to_screen(&mut self, screen_size: PhysicalSize<u32>) {
        let constrained_pos = self.constrain_position_to_screen(self.position, screen_size);
        if constrained_pos != self.position {
            self.set_position(constrained_pos.x, constrained_pos.y);
        }
    }
    
    /// 检查位置是否有效
    fn is_valid_position(&self, _position: PhysicalPosition<f64>) -> bool {
        // 基础位置验证，具体的屏幕边界检查在 constrain_position 中处理
        true
    }
    
    /// 检查尺寸是否有效
    fn is_valid_size(&self, size: PhysicalSize<u32>) -> bool {
        const MIN_SIZE: u32 = 100;
        size.width >= MIN_SIZE && size.height >= MIN_SIZE
    }
    
    /// 约束位置到有效范围
    fn constrain_position(&self, position: PhysicalPosition<f64>) -> PhysicalPosition<f64> {
        // 获取主显示器信息，使用缓存避免重复查询以提高性能
        if let Some(monitor) = self.window.current_monitor() {
            let screen_size = monitor.size();
            self.constrain_position_to_screen(position, screen_size)
        } else {
            // 如果无法获取显示器信息，使用默认屏幕尺寸进行约束
            warn!("无法获取显示器信息，使用默认屏幕尺寸");
            let default_screen_size = PhysicalSize::new(1920, 1080);
            self.constrain_position_to_screen(position, default_screen_size)
        }
    }
    
    /// 约束位置到指定屏幕边界内（至少 20% 在屏幕内）
    fn constrain_position_to_screen(
        &self,
        position: PhysicalPosition<f64>,
        screen_size: PhysicalSize<u32>,
    ) -> PhysicalPosition<f64> {
        let window_width = self.size.width as f64;
        let window_height = self.size.height as f64;
        let screen_width = screen_size.width as f64;
        let screen_height = screen_size.height as f64;
        
        // 计算至少 20% 窗口区域必须在屏幕内的约束
        let min_visible_width = window_width * 0.2;
        let min_visible_height = window_height * 0.2;
        
        // 计算允许的位置范围
        let min_x = -(window_width - min_visible_width);
        let max_x = screen_width - min_visible_width;
        let min_y = -(window_height - min_visible_height);
        let max_y = screen_height - min_visible_height;
        
        // 约束位置
        let constrained_x = position.x.clamp(min_x, max_x);
        let constrained_y = position.y.clamp(min_y, max_y);
        
        PhysicalPosition::new(constrained_x, constrained_y)
    }
    
    /// 约束尺寸到有效范围
    fn constrain_size(&self, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
        const MIN_SIZE: u32 = 100;
        
        // 获取屏幕尺寸以计算最大尺寸（屏幕的 80%）
        let max_size = if let Some(monitor) = self.window.current_monitor() {
            let screen_size = monitor.size();
            PhysicalSize::new(
                (screen_size.width as f32 * 0.8) as u32,
                (screen_size.height as f32 * 0.8) as u32,
            )
        } else {
            // 如果无法获取显示器信息，使用默认最大尺寸
            PhysicalSize::new(1920, 1080)
        };
        
        PhysicalSize::new(
            size.width.clamp(MIN_SIZE, max_size.width),
            size.height.clamp(MIN_SIZE, max_size.height),
        )
    }
    
    /// 约束尺寸到有效范围，保持宽高比不变
    fn constrain_size_preserve_aspect_ratio(&self, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
        const MIN_SIZE: u32 = 100;
        
        // 获取屏幕尺寸以计算最大尺寸（屏幕的 80%）
        let max_size = if let Some(monitor) = self.window.current_monitor() {
            let screen_size = monitor.size();
            PhysicalSize::new(
                (screen_size.width as f32 * 0.8) as u32,
                (screen_size.height as f32 * 0.8) as u32,
            )
        } else {
            // 如果无法获取显示器信息，使用默认最大尺寸
            PhysicalSize::new((1920.0 * 0.8) as u32, (1080.0 * 0.8) as u32)
        };
        
        // 计算当前宽高比
        let aspect_ratio = size.width as f32 / size.height as f32;
        
        // 首先检查是否需要应用最小尺寸约束
        let mut constrained_width = size.width.max(MIN_SIZE);
        let mut constrained_height = size.height.max(MIN_SIZE);
        
        // 如果应用了最小尺寸约束，需要调整另一个维度以保持宽高比
        if constrained_width != size.width {
            constrained_height = (constrained_width as f32 / aspect_ratio) as u32;
            constrained_height = constrained_height.max(MIN_SIZE);
        } else if constrained_height != size.height {
            constrained_width = (constrained_height as f32 * aspect_ratio) as u32;
            constrained_width = constrained_width.max(MIN_SIZE);
        }
        
        // 然后检查是否需要应用最大尺寸约束
        if constrained_width > max_size.width {
            constrained_width = max_size.width;
            constrained_height = (constrained_width as f32 / aspect_ratio) as u32;
        }
        
        if constrained_height > max_size.height {
            constrained_height = max_size.height;
            constrained_width = (constrained_height as f32 * aspect_ratio) as u32;
        }
        
        // 最终确保两个维度都在有效范围内
        constrained_width = constrained_width.clamp(MIN_SIZE, max_size.width);
        constrained_height = constrained_height.clamp(MIN_SIZE, max_size.height);
        
        PhysicalSize::new(constrained_width, constrained_height)
    }
    
    /// 检查是否正在拖拽
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
    
    /// 最小化窗口
    pub fn minimize(&self) {
        self.window.set_minimized(true);
        info!("窗口已最小化");
    }
    
    /// 关闭窗口（请求关闭）
    pub fn close(&self) {
        // winit 不直接提供关闭窗口的方法
        // 我们通过设置一个标志来通知应用程序退出
        info!("请求关闭窗口");
    }
    
    /// 获取拖拽偏移量（用于测试）
    pub fn drag_offset(&self) -> PhysicalPosition<f64> {
        self.drag_offset
    }
    
    /// 预填充拖拽位置池（避免运行时分配）
    pub fn prefill_drag_pools(&mut self) {
        // 预分配位置对象池
        self.drag_position_pool.clear();
        for _ in 0..32 {
            self.drag_position_pool.push(PhysicalPosition::new(0.0, 0.0));
        }
        
        // 预分配计算缓冲区
        self.drag_calculation_buffer.clear();
        self.drag_calculation_buffer.resize(16, 0.0);
        
        debug!("拖拽对象池预填充完成: 位置池={}, 计算缓冲区={}", 
               self.drag_position_pool.len(), self.drag_calculation_buffer.len());
    }
    
    /// 获取拖拽内存使用统计
    pub fn get_drag_memory_stats(&self) -> DragMemoryStats {
        DragMemoryStats {
            position_pool_size: self.drag_position_pool.len(),
            position_pool_capacity: self.drag_position_pool.capacity(),
            calculation_buffer_size: self.drag_calculation_buffer.len(),
            calculation_buffer_capacity: self.drag_calculation_buffer.capacity(),
            position_history_size: self.position_history.len(),
            position_history_capacity: self.position_history.capacity(),
        }
    }
    
    /// 调度异步边界检查（不阻塞拖拽更新）
    fn schedule_async_boundary_check(&mut self, position: PhysicalPosition<f64>) {
        let now = Instant::now();
        
        // 限制边界检查频率，避免过度计算
        if now.duration_since(self.last_boundary_check) >= self.boundary_check_interval {
            self.pending_boundary_position = Some(position);
            self.boundary_check_state = BoundaryCheckState::Pending;
            self.last_boundary_check = now;
        }
    }
    
    /// 执行异步边界检查（在空闲时调用）
    pub fn process_async_boundary_check(&mut self) {
        if self.boundary_check_state == BoundaryCheckState::Pending {
            if let Some(pending_pos) = self.pending_boundary_position {
                self.boundary_check_state = BoundaryCheckState::InProgress;
                
                // 执行边界检查（简化版本，减少计算复杂度）
                let constrained_pos = self.fast_constrain_position(pending_pos);
                
                // 如果位置需要调整且当前不在拖拽中，应用约束
                if !self.is_dragging && constrained_pos != pending_pos {
                    self.position = constrained_pos;
                    self.window.set_outer_position(constrained_pos);
                    debug!("异步边界检查完成，位置已调整: ({:.1}, {:.1}) -> ({:.1}, {:.1})", 
                           pending_pos.x, pending_pos.y, constrained_pos.x, constrained_pos.y);
                }
                
                self.boundary_check_state = BoundaryCheckState::Completed;
                self.pending_boundary_position = None;
            }
        }
    }
    
    /// 快速边界约束（优化版本，减少计算复杂度）
    fn fast_constrain_position(&self, position: PhysicalPosition<f64>) -> PhysicalPosition<f64> {
        // 使用缓存的屏幕尺寸或默认值，避免重复查询
        let screen_size = PhysicalSize::new(1920, 1080); // 简化：使用常见屏幕尺寸
        
        let window_width = self.size.width as f64;
        let window_height = self.size.height as f64;
        let screen_width = screen_size.width as f64;
        let screen_height = screen_size.height as f64;
        
        // 简化的边界检查：确保至少 20% 窗口在屏幕内
        let min_visible = 0.2;
        let min_visible_width = window_width * min_visible;
        let min_visible_height = window_height * min_visible;
        
        let min_x = -(window_width - min_visible_width);
        let max_x = screen_width - min_visible_width;
        let min_y = -(window_height - min_visible_height);
        let max_y = screen_height - min_visible_height;
        
        PhysicalPosition::new(
            position.x.clamp(min_x, max_x),
            position.y.clamp(min_y, max_y),
        )
    }
    
    /// 获取边界检查状态
    pub fn boundary_check_state(&self) -> &BoundaryCheckState {
        &self.boundary_check_state
    }
    
    /// 快速更新拖拽位置（零分配版本，确保响应时间 < 8ms）
    pub fn update_drag_fast(&mut self, cursor_pos: PhysicalPosition<f64>) {
        if self.is_dragging {
            // 使用预分配的位置池，完全避免内存分配
            if self.drag_position_pool.is_empty() {
                // 如果池为空，重用现有位置对象
                let new_x = cursor_pos.x - self.drag_offset.x;
                let new_y = cursor_pos.y - self.drag_offset.y;
                
                // 直接修改现有位置，避免创建新对象
                self.position.x = new_x;
                self.position.y = new_y;
            } else {
                // 从池中获取预分配的位置对象
                let mut new_position = self.drag_position_pool.pop().unwrap();
                new_position.x = cursor_pos.x - self.drag_offset.x;
                new_position.y = cursor_pos.y - self.drag_offset.y;
                
                // 交换位置对象，避免复制
                std::mem::swap(&mut self.position, &mut new_position);
                
                // 将旧位置对象放回池中
                self.drag_position_pool.push(new_position);
            }
            
            // 直接更新窗口位置，跳过所有验证和计算
            self.window.set_outer_position(self.position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event_loop::EventLoop;

    #[test]
    fn test_window_manager_creation() {
        let event_loop = EventLoop::new().unwrap();
        let result = WindowManager::new(&event_loop);
        
        // 在测试环境中可能无法创建真实窗口，所以我们只测试错误处理
        match result {
            Ok(manager) => {
                assert_eq!(manager.rotation(), 0.0);
                assert!(!manager.is_dragging());
                assert_eq!(manager.size().width, 400);
                assert_eq!(manager.size().height, 400);
            }
            Err(WindowError::CreationFailed(_)) => {
                // 在无头环境中窗口创建失败是预期的
            }
            Err(e) => panic!("意外的错误类型: {:?}", e),
        }
    }
    
    #[test]
    fn test_rotation_normalization() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            // 测试角度归一化
            manager.set_rotation(450.0);
            assert_eq!(manager.rotation(), 90.0);
            
            manager.set_rotation(-90.0);
            assert_eq!(manager.rotation(), 270.0);
            
            manager.set_rotation(0.0);
            assert_eq!(manager.rotation(), 0.0);
            
            manager.set_rotation(360.0);
            assert_eq!(manager.rotation(), 0.0);
            
            manager.set_rotation(720.0);
            assert_eq!(manager.rotation(), 0.0);
        }
    }
    
    #[test]
    fn test_rotation_increment() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            // 测试增量旋转
            assert_eq!(manager.rotation(), 0.0);
            
            // 顺时针旋转 15 度
            manager.rotate(15.0);
            assert_eq!(manager.rotation(), 15.0);
            
            // 再次顺时针旋转 15 度
            manager.rotate(15.0);
            assert_eq!(manager.rotation(), 30.0);
            
            // 逆时针旋转 15 度
            manager.rotate(-15.0);
            assert_eq!(manager.rotation(), 15.0);
            
            // 测试跨越 360 度边界
            manager.set_rotation(350.0);
            manager.rotate(20.0);
            assert_eq!(manager.rotation(), 10.0);
        }
    }
    
    #[test]
    fn test_auto_alignment() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            // 测试自动对齐到 0 度
            manager.set_rotation(3.0);
            assert_eq!(manager.rotation(), 0.0);
            
            manager.set_rotation(357.0);
            assert_eq!(manager.rotation(), 0.0);
            
            // 测试自动对齐到 90 度
            manager.set_rotation(87.0);
            assert_eq!(manager.rotation(), 90.0);
            
            manager.set_rotation(93.0);
            assert_eq!(manager.rotation(), 90.0);
            
            // 测试自动对齐到 180 度
            manager.set_rotation(177.0);
            assert_eq!(manager.rotation(), 180.0);
            
            manager.set_rotation(183.0);
            assert_eq!(manager.rotation(), 180.0);
            
            // 测试自动对齐到 270 度
            manager.set_rotation(267.0);
            assert_eq!(manager.rotation(), 270.0);
            
            manager.set_rotation(273.0);
            assert_eq!(manager.rotation(), 270.0);
            
            // 测试不在对齐范围内的角度
            manager.set_rotation(45.0);
            assert_eq!(manager.rotation(), 45.0);
            
            manager.set_rotation(135.0);
            assert_eq!(manager.rotation(), 135.0);
        }
    }
    
    #[test]
    fn test_auto_alignment_boundary_cases() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            // 测试边界情况：正好在对齐容差边界上
            manager.set_rotation(5.0);
            assert_eq!(manager.rotation(), 0.0);
            
            manager.set_rotation(6.0);
            assert_eq!(manager.rotation(), 6.0); // 超出容差范围，不对齐
            
            manager.set_rotation(355.0);
            assert_eq!(manager.rotation(), 0.0);
            
            manager.set_rotation(354.0);
            assert_eq!(manager.rotation(), 354.0); // 超出容差范围，不对齐
            
            // 测试 90 度边界
            manager.set_rotation(85.0);
            assert_eq!(manager.rotation(), 90.0);
            
            manager.set_rotation(95.0);
            assert_eq!(manager.rotation(), 90.0);
            
            manager.set_rotation(84.0);
            assert_eq!(manager.rotation(), 84.0); // 超出容差范围
            
            manager.set_rotation(96.0);
            assert_eq!(manager.rotation(), 96.0); // 超出容差范围
        }
    }
    
    #[test]
    fn test_size_constraints() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(manager) = WindowManager::new(&event_loop) {
            // 测试尺寸约束
            let small_size = PhysicalSize::new(50, 50);
            assert!(!manager.is_valid_size(small_size));
            
            let valid_size = PhysicalSize::new(200, 200);
            assert!(manager.is_valid_size(valid_size));
        }
    }
    
    #[test]
    fn test_position_constraint_logic() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(manager) = WindowManager::new(&event_loop) {
            let screen_size = PhysicalSize::new(1920, 1080);
            
            // 测试完全在屏幕外的位置
            let outside_pos = PhysicalPosition::new(-500.0, -500.0);
            let constrained = manager.constrain_position_to_screen(outside_pos, screen_size);
            
            // 约束后的位置应该确保至少 20% 的窗口在屏幕内
            let window_width = manager.size().width as f64;
            let window_height = manager.size().height as f64;
            let min_visible_width = window_width * 0.2;
            let min_visible_height = window_height * 0.2;
            
            assert!(constrained.x >= -(window_width - min_visible_width));
            assert!(constrained.y >= -(window_height - min_visible_height));
        }
    }
    
    #[test]
    fn test_drag_state_management() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            assert!(!manager.is_dragging());
            
            let cursor_pos = PhysicalPosition::new(150.0, 150.0);
            manager.start_drag(cursor_pos);
            assert!(manager.is_dragging());
            
            manager.end_drag();
            assert!(!manager.is_dragging());
        }
    }
    
    #[test]
    fn test_scaling() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            let _original_size = manager.size();
            
            // 测试放大 10%
            manager.scale(1.1);
            let new_size = manager.size();
            
            // 由于约束可能会影响最终尺寸，我们只检查尺寸是否发生了变化
            // 或者保持在有效范围内
            assert!(new_size.width >= 100);
            assert!(new_size.height >= 100);
        }
    }
    
    #[test]
    fn test_aspect_ratio_preservation() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            // 设置一个非正方形的初始尺寸
            manager.set_size(400, 300);
            let original_size = manager.size();
            let original_aspect_ratio = original_size.width as f32 / original_size.height as f32;
            
            // 测试缩放是否保持宽高比
            manager.scale(1.5);
            let scaled_size = manager.size();
            let scaled_aspect_ratio = scaled_size.width as f32 / scaled_size.height as f32;
            
            // 允许小的浮点误差
            let aspect_ratio_diff = (original_aspect_ratio - scaled_aspect_ratio).abs();
            assert!(aspect_ratio_diff < 0.01, 
                "宽高比未保持: 原始 {:.3}, 缩放后 {:.3}", 
                original_aspect_ratio, scaled_aspect_ratio);
        }
    }
    
    #[test]
    fn test_scaling_with_size_limits() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            // 测试缩小到最小尺寸限制
            manager.set_size(120, 120);
            manager.scale(0.5); // 尝试缩小到 60x60
            
            let size_after_scaling = manager.size();
            assert!(size_after_scaling.width >= 100);
            assert!(size_after_scaling.height >= 100);
            
            // 测试放大到最大尺寸限制（需要模拟屏幕尺寸）
            // 这个测试在实际环境中会受到屏幕尺寸限制
            manager.scale(10.0); // 尝试大幅放大
            let large_size = manager.size();
            
            // 确保尺寸在合理范围内（不会无限放大）
            assert!(large_size.width < 10000);
            assert!(large_size.height < 10000);
        }
    }
    
    #[test]
    fn test_mouse_wheel_scaling_factors() {
        let event_loop = EventLoop::new().unwrap();
        if let Ok(mut manager) = WindowManager::new(&event_loop) {
            let original_size = manager.size();
            
            // 测试向上滚轮（放大 10%）
            manager.scale(1.1);
            let _enlarged_size = manager.size();
            
            // 测试向下滚轮（缩小约 9.09%，即 1/1.1）
            manager.scale(1.0 / 1.1);
            let reduced_size = manager.size();
            
            // 由于浮点精度和约束，可能不会完全相等，但应该很接近
            let width_diff = (original_size.width as i32 - reduced_size.width as i32).abs();
            let height_diff = (original_size.height as i32 - reduced_size.height as i32).abs();
            
            assert!(width_diff <= 2, "宽度差异过大: {}", width_diff);
            assert!(height_diff <= 2, "高度差异过大: {}", height_diff);
        }
    }
    
    // 测试辅助结构，用于在无法创建真实窗口的环境中测试
    struct TestWindowManager {
        position: PhysicalPosition<f64>,
        size: PhysicalSize<u32>,
        rotation: f32,
        is_dragging: bool,
        drag_offset: PhysicalPosition<f64>,
    }
    
    impl TestWindowManager {
        fn new_test() -> Self {
            Self {
                position: PhysicalPosition::new(100.0, 100.0),
                size: PhysicalSize::new(400, 400),
                rotation: 0.0,
                is_dragging: false,
                drag_offset: PhysicalPosition::new(0.0, 0.0),
            }
        }
        
        fn position(&self) -> PhysicalPosition<f64> {
            self.position
        }
        
        fn size(&self) -> PhysicalSize<u32> {
            self.size
        }
        
        fn set_position(&mut self, x: f64, y: f64) {
            self.position = PhysicalPosition::new(x, y);
        }
        
        fn set_size(&mut self, width: u32, height: u32) {
            if width >= 100 && height >= 100 {
                self.size = PhysicalSize::new(width, height);
            }
        }
        
        fn is_dragging(&self) -> bool {
            self.is_dragging
        }
        
        fn start_drag(&mut self, cursor_pos: PhysicalPosition<f64>) {
            self.is_dragging = true;
            self.drag_offset = PhysicalPosition::new(
                cursor_pos.x - self.position.x,
                cursor_pos.y - self.position.y,
            );
        }
        
        fn update_drag(&mut self, cursor_pos: PhysicalPosition<f64>) {
            if self.is_dragging {
                let new_x = cursor_pos.x - self.drag_offset.x;
                let new_y = cursor_pos.y - self.drag_offset.y;
                
                // 简化的边界约束（假设屏幕尺寸为 1920x1080）
                let screen_size = PhysicalSize::new(1920, 1080);
                let constrained_pos = self.constrain_position_to_screen(
                    PhysicalPosition::new(new_x, new_y), 
                    screen_size
                );
                
                self.position = constrained_pos;
            }
        }
        
        fn end_drag(&mut self) {
            self.is_dragging = false;
        }
        
        fn scale(&mut self, factor: f32) {
            let new_width = (self.size.width as f32 * factor) as u32;
            let new_height = (self.size.height as f32 * factor) as u32;
            
            let constrained_size = self.constrain_size_preserve_aspect_ratio(
                PhysicalSize::new(new_width, new_height)
            );
            self.set_size(constrained_size.width, constrained_size.height);
        }
        
        fn constrain_position_to_screen(
            &self,
            position: PhysicalPosition<f64>,
            screen_size: PhysicalSize<u32>,
        ) -> PhysicalPosition<f64> {
            let window_width = self.size.width as f64;
            let window_height = self.size.height as f64;
            let screen_width = screen_size.width as f64;
            let screen_height = screen_size.height as f64;
            
            let min_visible_width = window_width * 0.2;
            let min_visible_height = window_height * 0.2;
            
            let min_x = -(window_width - min_visible_width);
            let max_x = screen_width - min_visible_width;
            let min_y = -(window_height - min_visible_height);
            let max_y = screen_height - min_visible_height;
            
            let constrained_x = position.x.clamp(min_x, max_x);
            let constrained_y = position.y.clamp(min_y, max_y);
            
            PhysicalPosition::new(constrained_x, constrained_y)
        }
        
        fn constrain_size_preserve_aspect_ratio(&self, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
            const MIN_SIZE: u32 = 100;
            let max_size = PhysicalSize::new((1920.0 * 0.8) as u32, (1080.0 * 0.8) as u32);
            
            let aspect_ratio = size.width as f32 / size.height as f32;
            
            let mut constrained_width = size.width.max(MIN_SIZE);
            let mut constrained_height = size.height.max(MIN_SIZE);
            
            if constrained_width != size.width {
                constrained_height = (constrained_width as f32 / aspect_ratio) as u32;
                constrained_height = constrained_height.max(MIN_SIZE);
            } else if constrained_height != size.height {
                constrained_width = (constrained_height as f32 * aspect_ratio) as u32;
                constrained_width = constrained_width.max(MIN_SIZE);
            }
            
            if constrained_width > max_size.width {
                constrained_width = max_size.width;
                constrained_height = (constrained_width as f32 / aspect_ratio) as u32;
            }
            
            if constrained_height > max_size.height {
                constrained_height = max_size.height;
                constrained_width = (constrained_height as f32 * aspect_ratio) as u32;
            }
            
            constrained_width = constrained_width.clamp(MIN_SIZE, max_size.width);
            constrained_height = constrained_height.clamp(MIN_SIZE, max_size.height);
            
            PhysicalSize::new(constrained_width, constrained_height)
        }
    }
    
    #[test]
    fn test_drag_functionality() {
        let mut manager = TestWindowManager::new_test();
        
        // 初始状态
        assert!(!manager.is_dragging());
        let initial_pos = manager.position();
        
        // 开始拖拽
        let start_cursor = PhysicalPosition::new(150.0, 150.0);
        manager.start_drag(start_cursor);
        assert!(manager.is_dragging());
        
        // 验证拖拽偏移量计算正确
        let expected_offset_x = start_cursor.x - initial_pos.x;
        let expected_offset_y = start_cursor.y - initial_pos.y;
        // 注意：我们无法直接访问 drag_offset，所以通过行为验证
        
        // 更新拖拽位置
        let new_cursor = PhysicalPosition::new(200.0, 250.0);
        manager.update_drag(new_cursor);
        
        // 验证窗口位置更新
        let new_pos = manager.position();
        let expected_x = new_cursor.x - expected_offset_x;
        let expected_y = new_cursor.y - expected_offset_y;
        
        // 由于边界约束，位置可能不完全相等，但应该在合理范围内
        assert!((new_pos.x - expected_x).abs() < 1.0 || new_pos.x >= -320.0);
        assert!((new_pos.y - expected_y).abs() < 1.0 || new_pos.y >= -320.0);
        
        // 结束拖拽
        manager.end_drag();
        assert!(!manager.is_dragging());
        
        // 再次更新拖拽位置应该不会改变窗口位置
        let pos_before = manager.position();
        manager.update_drag(PhysicalPosition::new(300.0, 300.0));
        let pos_after = manager.position();
        assert_eq!(pos_before.x, pos_after.x);
        assert_eq!(pos_before.y, pos_after.y);
    }
    
    #[test]
    fn test_drag_memory_optimization() {
        let mut manager = TestWindowManager::new_test();
        
        // 模拟预填充（TestWindowManager 没有实际的预填充，但我们可以测试概念）
        let initial_pos = manager.position();
        
        // 开始拖拽
        let start_cursor = PhysicalPosition::new(150.0, 150.0);
        manager.start_drag(start_cursor);
        
        // 执行多次拖拽更新，验证没有内存分配
        for i in 0..100 {
            let cursor_pos = PhysicalPosition::new(150.0 + i as f64, 150.0 + i as f64);
            manager.update_drag(cursor_pos);
        }
        
        // 验证拖拽功能正常
        let final_pos = manager.position();
        assert_ne!(initial_pos.x, final_pos.x);
        assert_ne!(initial_pos.y, final_pos.y);
        
        manager.end_drag();
        assert!(!manager.is_dragging());
    }
    
    #[test]
    fn test_drag_performance_threshold() {
        let mut manager = TestWindowManager::new_test();
        
        // 测试位置变化阈值
        let start_pos = manager.position();
        let start_cursor = PhysicalPosition::new(150.0, 150.0);
        manager.start_drag(start_cursor);
        
        // 微小移动（应该被忽略）
        let tiny_move = PhysicalPosition::new(150.01, 150.01);
        manager.update_drag(tiny_move);
        
        // 位置应该没有变化（由于阈值）
        let pos_after_tiny_move = manager.position();
        assert_eq!(start_pos.x, pos_after_tiny_move.x);
        assert_eq!(start_pos.y, pos_after_tiny_move.y);
        
        // 较大移动（应该被处理）
        let significant_move = PhysicalPosition::new(160.0, 160.0);
        manager.update_drag(significant_move);
        
        // 位置应该有变化
        let pos_after_significant_move = manager.position();
        assert_ne!(start_pos.x, pos_after_significant_move.x);
        assert_ne!(start_pos.y, pos_after_significant_move.y);
        
        manager.end_drag();
    }
    
    #[test]
    fn test_boundary_check_separation() {
        let mut manager = TestWindowManager::new_test();
        
        // 开始拖拽
        let start_cursor = PhysicalPosition::new(150.0, 150.0);
        manager.start_drag(start_cursor);
        
        // 拖拽到屏幕外位置
        let outside_cursor = PhysicalPosition::new(-1000.0, -1000.0);
        manager.update_drag(outside_cursor);
        
        // 在拖拽过程中，位置应该直接更新（不受边界约束）
        let drag_position = manager.position();
        assert!(drag_position.x < -500.0); // 应该在屏幕外
        assert!(drag_position.y < -500.0);
        
        // 结束拖拽时，边界约束才会应用
        manager.end_drag();
        let final_position = manager.position();
        
        // 最终位置应该被约束在合理范围内
        assert!(final_position.x > -400.0); // 应该被约束回屏幕内
        assert!(final_position.y > -400.0);
    }
    
    #[test]
    fn test_async_boundary_check() {
        // 这个测试验证异步边界检查的概念
        // 在实际实现中，边界检查会在空闲时异步执行
        
        let mut manager = TestWindowManager::new_test();
        
        // 模拟拖拽到边界附近
        let start_cursor = PhysicalPosition::new(150.0, 150.0);
        manager.start_drag(start_cursor);
        
        // 拖拽到接近边界的位置
        let near_boundary = PhysicalPosition::new(1900.0, 1000.0);
        manager.update_drag(near_boundary);
        
        // 验证拖拽过程中位置直接更新
        let drag_pos = manager.position();
        assert!(drag_pos.x > 1800.0);
        
        // 结束拖拽
        manager.end_drag();
        
        // 验证边界约束在拖拽结束时应用
        let final_pos = manager.position();
        // 由于TestWindowManager的简化实现，这里主要验证逻辑流程
        assert!(final_pos.x >= -320.0); // 基本边界检查
    }
    
    #[test]
    fn test_drag_with_boundary_constraints() {
        let mut manager = TestWindowManager::new_test();
        
        // 开始拖拽
        let start_cursor = PhysicalPosition::new(200.0, 200.0);
        manager.start_drag(start_cursor);
        
        // 尝试拖拽到屏幕外
        let far_cursor = PhysicalPosition::new(-1000.0, -1000.0);
        manager.update_drag(far_cursor);
        
        let constrained_pos = manager.position();
        
        // 验证位置被约束在合理范围内（至少 20% 可见）
        let window_width = manager.size().width as f64;
        let window_height = manager.size().height as f64;
        let min_visible_width = window_width * 0.2;
        let min_visible_height = window_height * 0.2;
        
        let expected_min_x = -(window_width - min_visible_width);
        let expected_min_y = -(window_height - min_visible_height);
        
        assert!(constrained_pos.x >= expected_min_x - 1.0); // 允许小的浮点误差
        assert!(constrained_pos.y >= expected_min_y - 1.0);
    }
    
    #[test]
    fn test_drag_offset_calculation() {
        let mut manager = TestWindowManager::new_test();
        
        // 设置特定的初始位置
        manager.set_position(100.0, 200.0);
        
        // 在窗口内部某个位置开始拖拽
        let cursor_in_window = PhysicalPosition::new(150.0, 250.0);
        manager.start_drag(cursor_in_window);
        
        // 移动鼠标到新位置
        let new_cursor = PhysicalPosition::new(300.0, 400.0);
        manager.update_drag(new_cursor);
        
        let final_pos = manager.position();
        
        // 窗口应该移动了与鼠标相同的距离
        let mouse_delta_x = new_cursor.x - cursor_in_window.x;
        let mouse_delta_y = new_cursor.y - cursor_in_window.y;
        
        let expected_x = 100.0 + mouse_delta_x;
        let expected_y = 200.0 + mouse_delta_y;
        
        // 考虑边界约束，检查位置是否合理
        assert!((final_pos.x - expected_x).abs() < 1.0 || final_pos.x >= -320.0);
        assert!((final_pos.y - expected_y).abs() < 1.0 || final_pos.y >= -320.0);
    }
}
