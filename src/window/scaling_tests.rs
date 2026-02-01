// 窗口缩放功能的独立测试

use winit::dpi::PhysicalSize;

/// 测试用的简化窗口管理器，不依赖真实窗口系统
pub struct MockWindowManager {
    size: PhysicalSize<u32>,
}

impl MockWindowManager {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            size: PhysicalSize::new(width, height),
        }
    }
    
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
    
    pub fn scale(&mut self, factor: f32) {
        let new_width = (self.size.width as f32 * factor) as u32;
        let new_height = (self.size.height as f32 * factor) as u32;
        
        // 应用尺寸约束，保持宽高比
        let constrained_size = self.constrain_size_preserve_aspect_ratio(
            PhysicalSize::new(new_width, new_height)
        );
        self.size = constrained_size;
    }
    
    fn constrain_size_preserve_aspect_ratio(&self, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
        const MIN_SIZE: u32 = 100;
        let max_size = PhysicalSize::new((1920.0 * 0.8) as u32, (1080.0 * 0.8) as u32);
        
        let aspect_ratio = size.width as f32 / size.height as f32;
        
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaling_preserves_aspect_ratio() {
        let mut manager = MockWindowManager::new(400, 300);
        let original_size = manager.size();
        let original_aspect_ratio = original_size.width as f32 / original_size.height as f32;
        
        // 测试放大
        manager.scale(1.5);
        let scaled_size = manager.size();
        let scaled_aspect_ratio = scaled_size.width as f32 / scaled_size.height as f32;
        
        let aspect_ratio_diff = (original_aspect_ratio - scaled_aspect_ratio).abs();
        assert!(aspect_ratio_diff < 0.01, 
            "宽高比未保持: 原始 {:.3}, 缩放后 {:.3}, 差异 {:.3}", 
            original_aspect_ratio, scaled_aspect_ratio, aspect_ratio_diff);
    }
    
    #[test]
    fn test_mouse_wheel_scaling_increments() {
        let mut manager = MockWindowManager::new(400, 400);
        let original_size = manager.size();
        
        // 测试向上滚轮（放大 10%）
        manager.scale(1.1);
        let enlarged_size = manager.size();
        
        // 验证尺寸增加了约 10%
        let width_increase = (enlarged_size.width as f32 / original_size.width as f32 - 1.0) * 100.0;
        let height_increase = (enlarged_size.height as f32 / original_size.height as f32 - 1.0) * 100.0;
        
        assert!((width_increase - 10.0).abs() < 1.0, "宽度增加不是 10%: {:.1}%", width_increase);
        assert!((height_increase - 10.0).abs() < 1.0, "高度增加不是 10%: {:.1}%", height_increase);
        
        // 测试向下滚轮（缩小约 9.09%，即 1/1.1）
        manager.scale(1.0 / 1.1);
        let reduced_size = manager.size();
        
        // 验证回到了接近原始尺寸
        let width_diff = (original_size.width as i32 - reduced_size.width as i32).abs();
        let height_diff = (original_size.height as i32 - reduced_size.height as i32).abs();
        
        assert!(width_diff <= 2, "宽度差异过大: {}", width_diff);
        assert!(height_diff <= 2, "高度差异过大: {}", height_diff);
    }
    
    #[test]
    fn test_minimum_size_constraint() {
        let mut manager = MockWindowManager::new(120, 120);
        
        // 尝试缩小到低于最小尺寸
        manager.scale(0.5); // 尝试缩小到 60x60
        
        let size_after_scaling = manager.size();
        assert!(size_after_scaling.width >= 100, "宽度低于最小限制: {}", size_after_scaling.width);
        assert!(size_after_scaling.height >= 100, "高度低于最小限制: {}", size_after_scaling.height);
    }
    
    #[test]
    fn test_maximum_size_constraint() {
        let mut manager = MockWindowManager::new(1000, 1000);
        
        // 尝试放大到超过最大尺寸（屏幕的 80%）
        manager.scale(2.0); // 尝试放大到 2000x2000
        
        let size_after_scaling = manager.size();
        let max_width = (1920.0 * 0.8) as u32;
        let max_height = (1080.0 * 0.8) as u32;
        
        assert!(size_after_scaling.width <= max_width, 
            "宽度超过最大限制: {} > {}", size_after_scaling.width, max_width);
        assert!(size_after_scaling.height <= max_height, 
            "高度超过最大限制: {} > {}", size_after_scaling.height, max_height);
    }
    
    #[test]
    fn test_aspect_ratio_with_constraints() {
        // 测试非正方形窗口在约束下的宽高比保持
        let mut manager = MockWindowManager::new(800, 600); // 4:3 宽高比
        let original_aspect_ratio = 800.0 / 600.0;
        
        // 尝试放大到接近最大尺寸
        manager.scale(2.0);
        let scaled_size = manager.size();
        let scaled_aspect_ratio = scaled_size.width as f32 / scaled_size.height as f32;
        
        // 即使受到尺寸约束，宽高比也应该尽可能保持
        let aspect_ratio_diff = (original_aspect_ratio - scaled_aspect_ratio).abs();
        assert!(aspect_ratio_diff < 0.1, 
            "在约束下宽高比变化过大: 原始 {:.3}, 约束后 {:.3}", 
            original_aspect_ratio, scaled_aspect_ratio);
    }
    
    #[test]
    fn test_scaling_edge_cases() {
        let mut manager = MockWindowManager::new(400, 300);
        
        // 测试缩放因子为 1.0（不应该改变尺寸）
        let original_size = manager.size();
        manager.scale(1.0);
        let unchanged_size = manager.size();
        
        assert_eq!(original_size.width, unchanged_size.width);
        assert_eq!(original_size.height, unchanged_size.height);
        
        // 测试非常小的缩放因子
        manager.scale(0.01); // 尝试缩小到 4x3
        let tiny_size = manager.size();
        
        assert!(tiny_size.width >= 100);
        assert!(tiny_size.height >= 100);
        
        // 测试非常大的缩放因子
        manager = MockWindowManager::new(100, 100);
        manager.scale(100.0); // 尝试放大到 10000x10000
        let huge_size = manager.size();
        
        let max_width = (1920.0 * 0.8) as u32;
        let max_height = (1080.0 * 0.8) as u32;
        
        assert!(huge_size.width <= max_width);
        assert!(huge_size.height <= max_height);
    }
}