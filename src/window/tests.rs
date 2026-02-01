// 窗口管理器测试

use winit::dpi::{PhysicalPosition, PhysicalSize};

/// 测试用的窗口管理器，不需要真实窗口
pub struct TestWindowManager {
    position: PhysicalPosition<f64>,
    size: PhysicalSize<u32>,
    rotation: f32,
    is_dragging: bool,
    drag_offset: PhysicalPosition<f64>,
}

impl TestWindowManager {
    pub fn new_test() -> Self {
        Self {
            position: PhysicalPosition::new(100.0, 100.0),
            size: PhysicalSize::new(400, 400),
            rotation: 0.0,
            is_dragging: false,
            drag_offset: PhysicalPosition::new(0.0, 0.0),
        }
    }
    
    pub fn position(&self) -> PhysicalPosition<f64> {
        self.position
    }
    
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
    
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    
    pub fn set_rotation(&mut self, degrees: f32) {
        let normalized = self.normalize_angle(degrees);
        let aligned = self.apply_auto_alignment(normalized);
        self.rotation = aligned;
    }
    
    pub fn rotate(&mut self, delta_degrees: f32) {
        let new_angle = self.rotation + delta_degrees;
        self.set_rotation(new_angle);
    }
    
    fn normalize_angle(&self, degrees: f32) -> f32 {
        let normalized = degrees % 360.0;
        if normalized < 0.0 { normalized + 360.0 } else { normalized }
    }
    
    fn apply_auto_alignment(&self, degrees: f32) -> f32 {
        const ALIGNMENT_TOLERANCE: f32 = 5.0;
        const ALIGNMENT_ANGLES: [f32; 4] = [0.0, 90.0, 180.0, 270.0];
        
        for &target_angle in &ALIGNMENT_ANGLES {
            let diff = (degrees - target_angle).abs();
            let diff_wrapped = (degrees - (target_angle + 360.0)).abs();
            let diff_wrapped_neg = ((degrees + 360.0) - target_angle).abs();
            
            let min_diff = diff.min(diff_wrapped).min(diff_wrapped_neg);
            
            if min_diff <= ALIGNMENT_TOLERANCE {
                return target_angle;
            }
        }
        
        degrees
    }
    
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
    
    pub fn start_drag(&mut self, cursor_pos: PhysicalPosition<f64>) {
        self.is_dragging = true;
        self.drag_offset = PhysicalPosition::new(
            cursor_pos.x - self.position.x,
            cursor_pos.y - self.position.y,
        );
    }
    
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
    }
    
    pub fn update_drag(&mut self, cursor_pos: PhysicalPosition<f64>) {
        if self.is_dragging {
            // 计算新位置（考虑拖拽偏移量）
            let new_x = cursor_pos.x - self.drag_offset.x;
            let new_y = cursor_pos.y - self.drag_offset.y;
            
            // 应用边界约束
            let constrained_pos = self.constrain_position_to_screen(
                PhysicalPosition::new(new_x, new_y),
                PhysicalSize::new(1920, 1080), // 使用默认屏幕尺寸
            );
            
            self.position = constrained_pos;
        }
    }
    
    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position = PhysicalPosition::new(x, y);
    }
    
    pub fn constrain_position_to_screen(
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
    
    pub fn constrain_size(&self, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
        const MIN_SIZE: u32 = 100;
        let max_size = PhysicalSize::new(1920, 1080); // 假设的最大尺寸
        
        PhysicalSize::new(
            size.width.clamp(MIN_SIZE, max_size.width),
            size.height.clamp(MIN_SIZE, max_size.height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_normalization() {
        let mut manager = TestWindowManager::new_test();
        
        // 测试角度归一化
        manager.set_rotation(450.0);
        assert_eq!(manager.rotation(), 90.0);
        
        manager.set_rotation(-90.0);
        assert_eq!(manager.rotation(), 270.0);
        
        manager.set_rotation(0.0);
        assert_eq!(manager.rotation(), 0.0);
        
        manager.set_rotation(360.0);
        assert_eq!(manager.rotation(), 0.0);
    }
    
    #[test]
    fn test_rotation_increment() {
        let mut manager = TestWindowManager::new_test();
        
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
    
    #[test]
    fn test_auto_alignment() {
        let mut manager = TestWindowManager::new_test();
        
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
    
    #[test]
    fn test_auto_alignment_boundary_cases() {
        let mut manager = TestWindowManager::new_test();
        
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
    
    #[test]
    fn test_rotation_requirements_compliance() {
        let mut manager = TestWindowManager::new_test();
        
        // 需求 6.1: 支持 Ctrl + 鼠标滚轮旋转（通过增量旋转测试）
        manager.rotate(15.0);  // 模拟向上滚轮
        assert_eq!(manager.rotation(), 15.0);
        
        manager.rotate(-15.0); // 模拟向下滚轮
        assert_eq!(manager.rotation(), 0.0);
        
        // 需求 6.2: 向上滚动顺时针旋转 15 度
        manager.set_rotation(0.0);
        manager.rotate(15.0);
        assert_eq!(manager.rotation(), 15.0);
        
        // 需求 6.3: 向下滚动逆时针旋转 15 度
        manager.rotate(-15.0);
        assert_eq!(manager.rotation(), 0.0);
        
        // 需求 6.4: 支持 0 到 360 度的任意旋转角度
        let test_angles = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 359.9];
        for angle in test_angles {
            manager.set_rotation(angle);
            let result = manager.rotation();
            assert!(result >= 0.0 && result < 360.0, "角度 {} 超出有效范围: {}", angle, result);
        }
        
        // 需求 6.6: 自动对齐到精确角度（误差范围 ±5 度）
        let alignment_tests = [
            (2.0, 0.0),    // 0° ±5°
            (358.0, 0.0),  // 0° ±5°
            (88.0, 90.0),  // 90° ±5°
            (92.0, 90.0),  // 90° ±5°
            (178.0, 180.0), // 180° ±5°
            (182.0, 180.0), // 180° ±5°
            (268.0, 270.0), // 270° ±5°
            (272.0, 270.0), // 270° ±5°
        ];
        
        for (input, expected) in alignment_tests {
            manager.set_rotation(input);
            assert_eq!(manager.rotation(), expected, 
                "自动对齐失败: {} 度应该对齐到 {} 度，实际得到 {} 度", 
                input, expected, manager.rotation());
        }
    }
    
    #[test]
    fn test_rotation_with_multiple_increments() {
        let mut manager = TestWindowManager::new_test();
        
        // 测试多次 15 度增量旋转
        let mut expected_angle = 0.0;
        
        for i in 1..=24 { // 24 * 15 = 360 度，完整一圈
            manager.rotate(15.0);
            expected_angle = (expected_angle + 15.0) % 360.0;
            
            let actual_angle = manager.rotation();
            
            // 考虑自动对齐的影响
            if (expected_angle - 0.0_f32).abs() <= 5.0 || (expected_angle - 360.0_f32).abs() <= 5.0 {
                assert_eq!(actual_angle, 0.0, "第 {} 次旋转后应该对齐到 0°", i);
            } else if (expected_angle - 90.0_f32).abs() <= 5.0 {
                assert_eq!(actual_angle, 90.0, "第 {} 次旋转后应该对齐到 90°", i);
            } else if (expected_angle - 180.0_f32).abs() <= 5.0 {
                assert_eq!(actual_angle, 180.0, "第 {} 次旋转后应该对齐到 180°", i);
            } else if (expected_angle - 270.0_f32).abs() <= 5.0 {
                assert_eq!(actual_angle, 270.0, "第 {} 次旋转后应该对齐到 270°", i);
            } else {
                assert_eq!(actual_angle, expected_angle, "第 {} 次旋转后角度不正确", i);
            }
        }
        
        // 完整一圈后应该回到 0 度（或接近 0 度并被对齐）
        assert_eq!(manager.rotation(), 0.0);
    }
    
    #[test]
    fn test_rotation_edge_cases() {
        let mut manager = TestWindowManager::new_test();
        
        // 测试极大的角度值
        manager.set_rotation(7200.0); // 20 圈
        assert_eq!(manager.rotation(), 0.0);
        
        // 测试极小的负角度值
        manager.set_rotation(-7200.0); // -20 圈
        assert_eq!(manager.rotation(), 0.0);
        
        // 测试浮点精度边界
        manager.set_rotation(359.999);
        let result = manager.rotation();
        assert!(result >= 0.0 && result < 360.0);
        
        // 测试零值
        manager.set_rotation(0.0);
        assert_eq!(manager.rotation(), 0.0);
        
        // 测试负零值（如果存在）
        manager.set_rotation(-0.0);
        assert_eq!(manager.rotation(), 0.0);
    }
    
    #[test]
    fn test_boundary_constraint_logic() {
        let manager = TestWindowManager::new_test();
        let screen_size = PhysicalSize::new(1920, 1080);
        
        // 测试完全在屏幕外的位置（左上角）
        let outside_pos = PhysicalPosition::new(-500.0, -500.0);
        let constrained = manager.constrain_position_to_screen(outside_pos, screen_size);
        
        // 验证至少 20% 的窗口在屏幕内
        let window_width = manager.size().width as f64;
        let window_height = manager.size().height as f64;
        let min_visible_width = window_width * 0.2;
        let min_visible_height = window_height * 0.2;
        
        let expected_min_x = -(window_width - min_visible_width);
        let expected_min_y = -(window_height - min_visible_height);
        
        assert_eq!(constrained.x, expected_min_x);
        assert_eq!(constrained.y, expected_min_y);
    }
    
    #[test]
    fn test_boundary_constraint_right_bottom() {
        let manager = TestWindowManager::new_test();
        let screen_size = PhysicalSize::new(1920, 1080);
        
        // 测试完全在屏幕外的位置（右下角）
        let outside_pos = PhysicalPosition::new(2500.0, 1500.0);
        let constrained = manager.constrain_position_to_screen(outside_pos, screen_size);
        
        // 验证至少 20% 的窗口在屏幕内
        let window_width = manager.size().width as f64;
        let window_height = manager.size().height as f64;
        let min_visible_width = window_width * 0.2;
        let min_visible_height = window_height * 0.2;
        let screen_width = screen_size.width as f64;
        let screen_height = screen_size.height as f64;
        
        let expected_max_x = screen_width - min_visible_width;
        let expected_max_y = screen_height - min_visible_height;
        
        assert_eq!(constrained.x, expected_max_x);
        assert_eq!(constrained.y, expected_max_y);
    }
    
    #[test]
    fn test_boundary_constraint_within_screen() {
        let manager = TestWindowManager::new_test();
        let screen_size = PhysicalSize::new(1920, 1080);
        
        // 测试在屏幕内的位置
        let inside_pos = PhysicalPosition::new(500.0, 300.0);
        let constrained = manager.constrain_position_to_screen(inside_pos, screen_size);
        
        // 在屏幕内的位置应该保持不变
        assert_eq!(constrained.x, inside_pos.x);
        assert_eq!(constrained.y, inside_pos.y);
    }
    
    #[test]
    fn test_size_constraints() {
        let manager = TestWindowManager::new_test();
        
        // 测试过小的尺寸
        let small_size = PhysicalSize::new(50, 50);
        let constrained = manager.constrain_size(small_size);
        assert_eq!(constrained.width, 100);
        assert_eq!(constrained.height, 100);
        
        // 测试正常尺寸
        let normal_size = PhysicalSize::new(400, 300);
        let constrained = manager.constrain_size(normal_size);
        assert_eq!(constrained.width, 400);
        assert_eq!(constrained.height, 300);
        
        // 测试过大的尺寸
        let large_size = PhysicalSize::new(3000, 2000);
        let constrained = manager.constrain_size(large_size);
        assert!(constrained.width <= 1920);
        assert!(constrained.height <= 1080);
    }
    
    #[test]
    fn test_drag_state_management() {
        let mut manager = TestWindowManager::new_test();
        
        assert!(!manager.is_dragging());
        
        let cursor_pos = PhysicalPosition::new(150.0, 150.0);
        manager.start_drag(cursor_pos);
        assert!(manager.is_dragging());
        
        manager.end_drag();
        assert!(!manager.is_dragging());
    }
    
    #[test]
    fn test_twenty_percent_visibility_calculation() {
        let manager = TestWindowManager::new_test();
        let screen_size = PhysicalSize::new(1000, 800);
        
        // 窗口尺寸: 400x400
        // 20% 可见区域: 80x80
        // 允许的最小 x: -(400 - 80) = -320
        // 允许的最大 x: 1000 - 80 = 920
        // 允许的最小 y: -(400 - 80) = -320  
        // 允许的最大 y: 800 - 80 = 720
        
        let test_cases = vec![
            // (输入位置, 期望的约束后位置)
            (PhysicalPosition::new(-400.0, -400.0), PhysicalPosition::new(-320.0, -320.0)),
            (PhysicalPosition::new(1000.0, 800.0), PhysicalPosition::new(920.0, 720.0)),
            (PhysicalPosition::new(500.0, 400.0), PhysicalPosition::new(500.0, 400.0)), // 在范围内
        ];
        
        for (input_pos, expected_pos) in test_cases {
            let constrained = manager.constrain_position_to_screen(input_pos, screen_size);
            assert_eq!(constrained.x, expected_pos.x, "X 坐标约束失败，输入: {:?}", input_pos);
            assert_eq!(constrained.y, expected_pos.y, "Y 坐标约束失败，输入: {:?}", input_pos);
        }
    }
    
    #[test]
    fn test_complete_drag_workflow() {
        let mut manager = TestWindowManager::new_test();
        
        // 初始状态验证
        assert!(!manager.is_dragging());
        let initial_pos = manager.position();
        assert_eq!(initial_pos.x, 100.0);
        assert_eq!(initial_pos.y, 100.0);
        
        // 开始拖拽
        let start_cursor = PhysicalPosition::new(150.0, 150.0);
        manager.start_drag(start_cursor);
        assert!(manager.is_dragging());
        
        // 移动鼠标并更新拖拽
        let move_cursor = PhysicalPosition::new(200.0, 200.0);
        manager.update_drag(move_cursor);
        
        // 验证窗口跟随鼠标移动
        let new_pos = manager.position();
        let expected_delta_x = move_cursor.x - start_cursor.x;
        let expected_delta_y = move_cursor.y - start_cursor.y;
        let expected_x = initial_pos.x + expected_delta_x;
        let expected_y = initial_pos.y + expected_delta_y;
        
        assert_eq!(new_pos.x, expected_x);
        assert_eq!(new_pos.y, expected_y);
        
        // 结束拖拽
        manager.end_drag();
        assert!(!manager.is_dragging());
        
        // 验证拖拽结束后位置不再变化
        let pos_before_inactive_drag = manager.position();
        manager.update_drag(PhysicalPosition::new(300.0, 300.0));
        let pos_after_inactive_drag = manager.position();
        assert_eq!(pos_before_inactive_drag.x, pos_after_inactive_drag.x);
        assert_eq!(pos_before_inactive_drag.y, pos_after_inactive_drag.y);
    }
    
    #[test]
    fn test_drag_performance_requirements() {
        let mut manager = TestWindowManager::new_test();
        
        // 开始拖拽
        manager.start_drag(PhysicalPosition::new(100.0, 100.0));
        
        // 模拟快速连续的鼠标移动事件（模拟 60 FPS 的鼠标事件）
        let start_time = std::time::Instant::now();
        
        for i in 0..60 {
            let cursor_pos = PhysicalPosition::new(100.0 + i as f64, 100.0 + i as f64);
            manager.update_drag(cursor_pos);
        }
        
        let elapsed = start_time.elapsed();
        
        // 60 次更新应该在 1 秒内完成（每次 < 16.67ms）
        assert!(elapsed.as_millis() < 1000, "拖拽更新性能不符合要求: {:?}", elapsed);
        
        // 平均每次更新时间应该 < 16ms
        let avg_time_per_update = elapsed.as_millis() / 60;
        assert!(avg_time_per_update < 16, "平均拖拽响应时间过长: {}ms", avg_time_per_update);
    }
}