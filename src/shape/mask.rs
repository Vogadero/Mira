// 形状遮罩实现
//
// 本模块实现了 Mira 应用的形状遮罩系统，支持以下形状：
// - 圆形 (Circle): 标准圆形遮罩
// - 椭圆形 (Ellipse): 椭圆形遮罩，适应窗口宽高比
// - 矩形 (Rectangle): 矩形遮罩，覆盖整个区域
// - 圆角矩形 (RoundedRectangle): 带圆角的矩形遮罩
// - 心形 (Heart): 使用参数方程生成的心形遮罩
//
// 所有形状生成算法都经过性能优化，确保切换时间 < 100ms。
// 遮罩数据使用单通道 alpha 值表示，255 为完全不透明，0 为完全透明。

/// 形状类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeType {
    Circle,
    Ellipse,
    Rectangle,
    RoundedRectangle { radius: f32 },
    Heart,
}

/// 形状遮罩
pub struct ShapeMask {
    shape_type: ShapeType,
    width: u32,
    height: u32,
    mask_data: Vec<u8>,
}

impl ShapeMask {
    /// 创建新的形状遮罩
    pub fn new(shape_type: ShapeType, width: u32, height: u32) -> Self {
        let mut mask = Self {
            shape_type,
            width,
            height,
            mask_data: Vec::new(),
        };
        mask.generate();
        mask
    }

    /// 生成遮罩数据（alpha 通道）
    pub fn generate(&mut self) {
        match self.shape_type {
            ShapeType::Circle => self.generate_circle(),
            ShapeType::Ellipse => self.generate_ellipse(),
            ShapeType::Rectangle => self.generate_rectangle(),
            ShapeType::RoundedRectangle { radius } => self.generate_rounded_rectangle(radius),
            ShapeType::Heart => self.generate_heart(),
        }
    }

    /// 获取遮罩数据
    pub fn data(&self) -> &[u8] {
        &self.mask_data
    }

    /// 改变形状类型
    pub fn set_shape(&mut self, shape_type: ShapeType) {
        self.shape_type = shape_type;
        self.generate();
    }

    /// 调整遮罩尺寸
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.generate();
    }

    /// 获取当前形状类型
    pub fn shape_type(&self) -> ShapeType {
        self.shape_type
    }

    /// 获取宽度
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取高度
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 生成圆形遮罩
    fn generate_circle(&mut self) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let radius = (self.width.min(self.height) as f32) / 2.0;
        
        self.mask_data = vec![0u8; (self.width * self.height) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance <= radius {
                    self.mask_data[(y * self.width + x) as usize] = 255; // 完全不透明
                } else {
                    self.mask_data[(y * self.width + x) as usize] = 0;   // 完全透明
                }
            }
        }
    }

    /// 生成椭圆形遮罩
    fn generate_ellipse(&mut self) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let radius_x = self.width as f32 / 2.0;
        let radius_y = self.height as f32 / 2.0;
        
        self.mask_data = vec![0u8; (self.width * self.height) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = (x as f32 - center_x) / radius_x;
                let dy = (y as f32 - center_y) / radius_y;
                let distance_squared = dx * dx + dy * dy;
                
                if distance_squared <= 1.0 {
                    self.mask_data[(y * self.width + x) as usize] = 255; // 完全不透明
                } else {
                    self.mask_data[(y * self.width + x) as usize] = 0;   // 完全透明
                }
            }
        }
    }

    /// 生成矩形遮罩
    fn generate_rectangle(&mut self) {
        // 矩形遮罩就是整个区域都不透明
        self.mask_data = vec![255u8; (self.width * self.height) as usize];
    }

    /// 生成圆角矩形遮罩
    fn generate_rounded_rectangle(&mut self, radius: f32) {
        let radius = radius.min(self.width as f32 / 2.0).min(self.height as f32 / 2.0);
        
        self.mask_data = vec![0u8; (self.width * self.height) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let x_f = x as f32;
                let y_f = y as f32;
                let width_f = self.width as f32;
                let height_f = self.height as f32;
                
                // 检查是否在圆角区域
                let in_corner = if x_f < radius && y_f < radius {
                    // 左上角
                    let dx = radius - x_f;
                    let dy = radius - y_f;
                    dx * dx + dy * dy <= radius * radius
                } else if x_f >= width_f - radius && y_f < radius {
                    // 右上角
                    let dx = x_f - (width_f - radius);
                    let dy = radius - y_f;
                    dx * dx + dy * dy <= radius * radius
                } else if x_f < radius && y_f >= height_f - radius {
                    // 左下角
                    let dx = radius - x_f;
                    let dy = y_f - (height_f - radius);
                    dx * dx + dy * dy <= radius * radius
                } else if x_f >= width_f - radius && y_f >= height_f - radius {
                    // 右下角
                    let dx = x_f - (width_f - radius);
                    let dy = y_f - (height_f - radius);
                    dx * dx + dy * dy <= radius * radius
                } else {
                    // 不在圆角区域，直接在矩形内
                    true
                };
                
                if in_corner {
                    self.mask_data[(y * self.width + x) as usize] = 255;
                } else {
                    self.mask_data[(y * self.width + x) as usize] = 0;
                }
            }
        }
    }

    /// 生成心形遮罩
    fn generate_heart(&mut self) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let scale = (self.width.min(self.height) as f32) / 40.0;
        
        self.mask_data = vec![0u8; (self.width * self.height) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let px = (x as f32 - center_x) / scale;
                let py = -(y as f32 - center_y) / scale;
                
                // 使用心形隐式方程: (x^2 + y^2 - 1)^3 - x^2 * y^3 <= 0
                if self.is_inside_heart(px, py) {
                    self.mask_data[(y * self.width + x) as usize] = 255;
                } else {
                    self.mask_data[(y * self.width + x) as usize] = 0;
                }
            }
        }
    }

    /// 检查点是否在心形内部
    fn is_inside_heart(&self, x: f32, y: f32) -> bool {
        let x2 = x * x;
        let y2 = y * y;
        let y3 = y2 * y;
        
        // 心形隐式方程: (x^2 + y^2 - 1)^3 - x^2 * y^3 <= 0
        let left = (x2 + y2 - 1.0).powi(3);
        let right = x2 * y3;
        
        left - right <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_mask_creation() {
        let mask = ShapeMask::new(ShapeType::Circle, 100, 100);
        assert_eq!(mask.width(), 100);
        assert_eq!(mask.height(), 100);
        assert_eq!(mask.shape_type(), ShapeType::Circle);
    }

    #[test]
    fn test_shape_mask_resize() {
        let mut mask = ShapeMask::new(ShapeType::Circle, 100, 100);
        mask.resize(200, 200);
        assert_eq!(mask.width(), 200);
        assert_eq!(mask.height(), 200);
    }

    #[test]
    fn test_shape_mask_set_shape() {
        let mut mask = ShapeMask::new(ShapeType::Circle, 100, 100);
        mask.set_shape(ShapeType::Rectangle);
        assert_eq!(mask.shape_type(), ShapeType::Rectangle);
    }

    #[test]
    fn test_circle_mask_generation() {
        let mask = ShapeMask::new(ShapeType::Circle, 100, 100);
        let data = mask.data();
        
        // 检查中心点应该是不透明的
        let center_idx = (50 * 100 + 50) as usize;
        assert_eq!(data[center_idx], 255);
        
        // 检查角落应该是透明的
        let corner_idx = (0 * 100 + 0) as usize;
        assert_eq!(data[corner_idx], 0);
    }

    #[test]
    fn test_ellipse_mask_generation() {
        let mask = ShapeMask::new(ShapeType::Ellipse, 100, 50);
        let data = mask.data();
        
        // 检查中心点应该是不透明的
        let center_idx = (25 * 100 + 50) as usize;
        assert_eq!(data[center_idx], 255);
        
        // 检查角落应该是透明的
        let corner_idx = (0 * 100 + 0) as usize;
        assert_eq!(data[corner_idx], 0);
    }

    #[test]
    fn test_rectangle_mask_generation() {
        let mask = ShapeMask::new(ShapeType::Rectangle, 100, 100);
        let data = mask.data();
        
        // 矩形遮罩所有像素都应该是不透明的
        for &pixel in data {
            assert_eq!(pixel, 255);
        }
    }

    #[test]
    fn test_rounded_rectangle_mask_generation() {
        let mask = ShapeMask::new(ShapeType::RoundedRectangle { radius: 10.0 }, 100, 100);
        let data = mask.data();
        
        // 检查中心点应该是不透明的
        let center_idx = (50 * 100 + 50) as usize;
        assert_eq!(data[center_idx], 255);
        
        // 检查角落应该是透明的（在圆角半径外）
        let corner_idx = (0 * 100 + 0) as usize;
        assert_eq!(data[corner_idx], 0);
    }

    #[test]
    fn test_heart_mask_generation() {
        let mask = ShapeMask::new(ShapeType::Heart, 100, 100);
        let data = mask.data();
        
        // 检查中心点应该是不透明的
        let center_idx = (50 * 100 + 50) as usize;
        assert_eq!(data[center_idx], 255);
        
        // 检查角落应该是透明的
        let corner_idx = (0 * 100 + 0) as usize;
        assert_eq!(data[corner_idx], 0);
    }

    #[test]
    fn test_mask_data_size() {
        let mask = ShapeMask::new(ShapeType::Circle, 100, 50);
        assert_eq!(mask.data().len(), 5000); // 100 * 50
    }

    #[test]
    fn test_shape_switching_performance() {
        let mut mask = ShapeMask::new(ShapeType::Circle, 400, 400);
        
        let start = std::time::Instant::now();
        mask.set_shape(ShapeType::Heart);
        let duration = start.elapsed();
        
        // 确保切换时间 < 100ms
        assert!(duration.as_millis() < 100, "Shape switching took {}ms", duration.as_millis());
    }

    #[test]
    fn test_all_shapes_generate_valid_data() {
        let shapes = vec![
            ShapeType::Circle,
            ShapeType::Ellipse,
            ShapeType::Rectangle,
            ShapeType::RoundedRectangle { radius: 5.0 },
            ShapeType::Heart,
        ];

        for shape in shapes {
            let mask = ShapeMask::new(shape, 100, 100);
            let data = mask.data();
            
            // 检查数据长度正确
            assert_eq!(data.len(), 10000);
            
            // 检查所有像素值都是有效的（0 或 255）
            for &pixel in data {
                assert!(pixel == 0 || pixel == 255, "Invalid pixel value: {}", pixel);
            }
        }
    }
}
