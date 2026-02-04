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
        // 确保圆形始终使用较小边作为直径，保持完美圆形
        let radius = (self.width.min(self.height) as f32) / 2.0;
        
        self.mask_data = vec![0u8; (self.width * self.height) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                // 添加抗锯齿效果，使边缘更平滑
                let alpha = if distance <= radius - 1.0 {
                    255
                } else if distance <= radius {
                    // 边缘像素使用渐变透明度
                    ((radius - distance) * 255.0) as u8
                } else {
                    0
                };
                
                self.mask_data[(y * self.width + x) as usize] = alpha;
            }
        }
    }

    /// 生成椭圆形遮罩
    fn generate_ellipse(&mut self) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        // 椭圆形使用更夸张的宽高比，让它更明显椭圆
        let radius_x = self.width as f32 / 2.0;
        let radius_y = self.height as f32 / 2.5; // 减小Y轴半径，让椭圆更扁
        
        self.mask_data = vec![0u8; (self.width * self.height) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = (x as f32 - center_x) / radius_x;
                let dy = (y as f32 - center_y) / radius_y;
                let distance_squared = dx * dx + dy * dy;
                
                // 添加抗锯齿效果，使边缘更平滑
                let alpha = if distance_squared <= 0.9 {
                    255
                } else if distance_squared <= 1.0 {
                    // 边缘像素使用渐变透明度
                    ((1.0 - distance_squared) * 255.0 / 0.1) as u8
                } else {
                    0
                };
                
                self.mask_data[(y * self.width + x) as usize] = alpha;
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
                
                // 计算到最近圆角的距离
                let alpha = if x_f < radius && y_f < radius {
                    // 左上角
                    let dx = radius - x_f;
                    let dy = radius - y_f;
                    let distance = (dx * dx + dy * dy).sqrt();
                    self.calculate_corner_alpha(distance, radius)
                } else if x_f >= width_f - radius && y_f < radius {
                    // 右上角
                    let dx = x_f - (width_f - radius);
                    let dy = radius - y_f;
                    let distance = (dx * dx + dy * dy).sqrt();
                    self.calculate_corner_alpha(distance, radius)
                } else if x_f < radius && y_f >= height_f - radius {
                    // 左下角
                    let dx = radius - x_f;
                    let dy = y_f - (height_f - radius);
                    let distance = (dx * dx + dy * dy).sqrt();
                    self.calculate_corner_alpha(distance, radius)
                } else if x_f >= width_f - radius && y_f >= height_f - radius {
                    // 右下角
                    let dx = x_f - (width_f - radius);
                    let dy = y_f - (height_f - radius);
                    let distance = (dx * dx + dy * dy).sqrt();
                    self.calculate_corner_alpha(distance, radius)
                } else {
                    // 不在圆角区域，直接在矩形内
                    255
                };
                
                self.mask_data[(y * self.width + x) as usize] = alpha;
            }
        }
    }
    
    /// 计算圆角的抗锯齿alpha值
    fn calculate_corner_alpha(&self, distance: f32, radius: f32) -> u8 {
        if distance <= radius - 1.0 {
            255 // 完全不透明
        } else if distance <= radius {
            // 边缘抗锯齿
            ((radius - distance) * 255.0) as u8
        } else {
            0 // 完全透明
        }
    }
    
    /// 应用多重采样抗锯齿（MSAA）
    fn apply_msaa(&mut self, samples: u32) {
        if samples <= 1 {
            return; // 不需要抗锯齿
        }
        
        let original_data = self.mask_data.clone();
        let sample_offset = 1.0 / (samples as f32);
        
        for y in 0..self.height {
            for x in 0..self.width {
                let mut total_alpha = 0u32;
                
                // 对每个像素进行多重采样
                for sy in 0..samples {
                    for sx in 0..samples {
                        let sample_x = x as f32 + (sx as f32 + 0.5) * sample_offset - 0.5;
                        let sample_y = y as f32 + (sy as f32 + 0.5) * sample_offset - 0.5;
                        
                        // 获取采样点的alpha值
                        let sample_alpha = self.sample_at_position(sample_x, sample_y, &original_data);
                        total_alpha += sample_alpha as u32;
                    }
                }
                
                // 计算平均alpha值
                let avg_alpha = (total_alpha / (samples * samples)) as u8;
                self.mask_data[(y * self.width + x) as usize] = avg_alpha;
            }
        }
    }
    
    /// 在指定位置采样alpha值（支持亚像素精度）
    fn sample_at_position(&self, x: f32, y: f32, data: &[u8]) -> u8 {
        // 边界检查
        if x < 0.0 || y < 0.0 || x >= self.width as f32 || y >= self.height as f32 {
            return 0;
        }
        
        // 双线性插值
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;
        
        let a00 = data[(y0 * self.width + x0) as usize] as f32;
        let a10 = data[(y0 * self.width + x1) as usize] as f32;
        let a01 = data[(y1 * self.width + x0) as usize] as f32;
        let a11 = data[(y1 * self.width + x1) as usize] as f32;
        
        let a0 = a00 * (1.0 - fx) + a10 * fx;
        let a1 = a01 * (1.0 - fx) + a11 * fx;
        let result = a0 * (1.0 - fy) + a1 * fy;
        
        result as u8
    }
    
    /// 应用高斯模糊以改善边缘质量
    fn apply_gaussian_blur(&mut self, radius: f32) {
        if radius <= 0.0 {
            return;
        }
        
        let kernel_size = (radius * 3.0).ceil() as usize * 2 + 1;
        let mut kernel = vec![0.0f32; kernel_size];
        let sigma = radius / 3.0;
        let sigma2 = sigma * sigma * 2.0;
        let center = kernel_size / 2;
        
        // 生成高斯核
        let mut sum = 0.0;
        for i in 0..kernel_size {
            let x = i as f32 - center as f32;
            let value = (-x * x / sigma2).exp();
            kernel[i] = value;
            sum += value;
        }
        
        // 归一化核
        for i in 0..kernel_size {
            kernel[i] /= sum;
        }
        
        let original_data = self.mask_data.clone();
        
        // 水平模糊
        let mut temp_data = vec![0u8; self.mask_data.len()];
        for y in 0..self.height {
            for x in 0..self.width {
                let mut blurred_value = 0.0;
                
                for i in 0..kernel_size {
                    let sample_x = x as i32 + i as i32 - center as i32;
                    if sample_x >= 0 && sample_x < self.width as i32 {
                        let idx = (y * self.width + sample_x as u32) as usize;
                        blurred_value += original_data[idx] as f32 * kernel[i];
                    }
                }
                
                temp_data[(y * self.width + x) as usize] = blurred_value as u8;
            }
        }
        
        // 垂直模糊
        for y in 0..self.height {
            for x in 0..self.width {
                let mut blurred_value = 0.0;
                
                for i in 0..kernel_size {
                    let sample_y = y as i32 + i as i32 - center as i32;
                    if sample_y >= 0 && sample_y < self.height as i32 {
                        let idx = (sample_y as u32 * self.width + x) as usize;
                        blurred_value += temp_data[idx] as f32 * kernel[i];
                    }
                }
                
                self.mask_data[(y * self.width + x) as usize] = blurred_value as u8;
            }
        }
    }
    
    /// 生成高质量形状遮罩（带抗锯齿）
    pub fn generate_high_quality(&mut self) {
        // 首先生成基本形状
        self.generate();
        
        // 根据形状类型应用不同的抗锯齿策略
        match self.shape_type {
            ShapeType::Circle | ShapeType::Ellipse => {
                // 圆形和椭圆形已经有内置抗锯齿
            }
            ShapeType::Rectangle => {
                // 矩形不需要抗锯齿
            }
            ShapeType::RoundedRectangle { .. } => {
                // 圆角矩形已经有内置抗锯齿
            }
            ShapeType::Heart => {
                // 心形已经有内置抗锯齿，但可以额外应用轻微模糊
                self.apply_gaussian_blur(0.5);
            }
        }
    }

    /// 生成心形遮罩
    fn generate_heart(&mut self) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        // 进一步增加心形大小，从 /5.0 改为 /2.0，让心形更大
        let scale = (self.width.min(self.height) as f32) / 2.0;
        
        self.mask_data = vec![0u8; (self.width * self.height) as usize];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let px = (x as f32 - center_x) / scale;
                let py = -(y as f32 - center_y) / scale;
                
                // 使用心形隐式方程: (x^2 + y^2 - 1)^3 - x^2 * y^3 <= 0
                // 添加抗锯齿效果
                let alpha = if self.is_inside_heart(px, py) {
                    // 在心形内部，检查是否在边缘附近
                    let edge_distance = self.heart_edge_distance(px, py);
                    if edge_distance > 0.1 {
                        255 // 完全不透明
                    } else {
                        // 边缘抗锯齿
                        ((edge_distance / 0.1) * 255.0) as u8
                    }
                } else {
                    0 // 完全透明
                };
                
                self.mask_data[(y * self.width + x) as usize] = alpha;
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
    
    /// 计算点到心形边缘的距离（用于抗锯齿）
    fn heart_edge_distance(&self, x: f32, y: f32) -> f32 {
        let x2 = x * x;
        let y2 = y * y;
        let y3 = y2 * y;
        
        // 心形隐式方程的值，越接近0越接近边缘
        let equation_value = (x2 + y2 - 1.0).powi(3) - x2 * y3;
        
        // 将方程值转换为距离估计
        if equation_value <= 0.0 {
            // 在心形内部，返回到边缘的估计距离
            (-equation_value).sqrt().min(1.0)
        } else {
            // 在心形外部
            0.0
        }
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
            
            // 检查所有像素值都是有效的（0-255范围，支持抗锯齿）
            for &pixel in data {
                assert!(pixel <= 255, "Invalid pixel value: {}", pixel);
            }
        }
    }

    #[test]
    fn test_circle_vs_ellipse_difference() {
        // 测试非正方形窗口中圆形和椭圆形的差异
        let width = 400;
        let height = 300;
        
        let circle_mask = ShapeMask::new(ShapeType::Circle, width, height);
        let ellipse_mask = ShapeMask::new(ShapeType::Ellipse, width, height);
        
        let circle_data = circle_mask.data();
        let ellipse_data = ellipse_mask.data();
        
        // 计算差异像素数量
        let mut diff_count = 0;
        for i in 0..circle_data.len() {
            if circle_data[i] != ellipse_data[i] {
                diff_count += 1;
            }
        }
        
        // 在非正方形窗口中，圆形和椭圆形应该有显著差异
        let total_pixels = (width * height) as usize;
        let diff_percentage = (diff_count as f32 / total_pixels as f32) * 100.0;
        
        // 至少应该有10%的像素不同
        assert!(diff_percentage > 10.0, 
                "圆形和椭圆形差异不够明显: {:.1}%", diff_percentage);
    }

    #[test]
    fn test_circle_maintains_aspect_ratio() {
        // 测试圆形在不同尺寸窗口中保持圆形
        let test_sizes = vec![(400, 300), (300, 400), (500, 200)];
        
        for (width, height) in test_sizes {
            let mask = ShapeMask::new(ShapeType::Circle, width, height);
            let data = mask.data();
            
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let expected_radius = (width.min(height) as f32) / 2.0;
            
            // 检查圆形边界上的点
            let test_angles = vec![0.0, std::f32::consts::PI / 2.0, std::f32::consts::PI, 3.0 * std::f32::consts::PI / 2.0];
            
            for angle in test_angles {
                let test_x = center_x + expected_radius * angle.cos();
                let test_y = center_y + expected_radius * angle.sin();
                
                if test_x >= 0.0 && test_x < width as f32 && test_y >= 0.0 && test_y < height as f32 {
                    let pixel_idx = (test_y as u32 * width + test_x as u32) as usize;
                    // 边界附近的像素应该不是完全透明的
                    assert!(data[pixel_idx] > 0, 
                            "圆形边界点 ({:.1}, {:.1}) 应该不透明", test_x, test_y);
                }
            }
        }
    }

    #[test]
    fn test_ellipse_adapts_to_window_ratio() {
        // 测试椭圆形适应窗口宽高比
        let test_sizes = vec![(400, 200), (200, 400)];
        
        for (width, height) in test_sizes {
            let mask = ShapeMask::new(ShapeType::Ellipse, width, height);
            let data = mask.data();
            
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            
            // 检查椭圆在长轴和短轴方向的边界
            let long_axis_x = if width > height { width as f32 / 2.0 - 1.0 } else { center_x };
            let long_axis_y = if height > width { height as f32 / 2.0 - 1.0 } else { center_y };
            
            let long_axis_idx = (long_axis_y as u32 * width + long_axis_x as u32) as usize;
            assert!(data[long_axis_idx] > 0, 
                    "椭圆长轴边界点应该不透明");
            
            // 检查椭圆外的点应该是透明的
            let outside_x = if width > height { width - 1 } else { width / 2 };
            let outside_y = if height > width { height - 1 } else { height / 2 };
            
            if width > height {
                // 宽椭圆，检查上下边界外的点
                if outside_y < height {
                    let outside_idx = (outside_y * width + outside_x) as usize;
                    assert_eq!(data[outside_idx], 0, "椭圆外的点应该透明");
                }
            } else {
                // 高椭圆，检查左右边界外的点
                if outside_x < width {
                    let outside_idx = (outside_y * width + outside_x) as usize;
                    assert_eq!(data[outside_idx], 0, "椭圆外的点应该透明");
                }
            }
        }
    }

    #[test]
    fn test_heart_size_and_coverage() {
        // 测试心形大小和覆盖率
        let test_sizes = vec![(400, 400), (300, 300), (500, 500)];
        
        for (width, height) in test_sizes {
            let mask = ShapeMask::new(ShapeType::Heart, width, height);
            let data = mask.data();
            
            // 计算心形覆盖的像素数量
            let mut covered_pixels = 0;
            for &pixel in data {
                if pixel > 0 {
                    covered_pixels += 1;
                }
            }
            
            let total_pixels = (width * height) as usize;
            let coverage_percentage = (covered_pixels as f32 / total_pixels as f32) * 100.0;
            
            // 心形应该占据至少60%的窗口面积
            assert!(coverage_percentage >= 60.0, 
                    "心形覆盖率不足: {:.1}% (期望 >= 60%)", coverage_percentage);
            
            // 但也不应该覆盖过多（不超过80%）
            assert!(coverage_percentage <= 80.0, 
                    "心形覆盖率过高: {:.1}% (期望 <= 80%)", coverage_percentage);
        }
    }

    #[test]
    fn test_heart_centering() {
        // 测试心形居中显示
        let mask = ShapeMask::new(ShapeType::Heart, 400, 400);
        let data = mask.data();
        
        let center_x = 200;
        let center_y = 200;
        
        // 检查中心点应该在心形内部
        let center_idx = (center_y * 400 + center_x) as usize;
        assert!(data[center_idx] > 0, "心形中心点应该不透明");
        
        // 检查心形的对称性（左右对称）
        for y in 100..300 {
            for x_offset in 1..50 {
                let left_x = center_x - x_offset;
                let right_x = center_x + x_offset;
                
                if left_x < 400 && right_x < 400 {
                    let left_idx = (y * 400 + left_x) as usize;
                    let right_idx = (y * 400 + right_x) as usize;
                    
                    // 左右对称点的透明度应该相近（允许一定误差）
                    let diff = (data[left_idx] as i16 - data[right_idx] as i16).abs();
                    assert!(diff <= 10, 
                            "心形左右对称性检查失败，位置 ({}, {}): 左={}, 右={}", 
                            x_offset, y, data[left_idx], data[right_idx]);
                }
            }
        }
    }

    #[test]
    fn test_heart_proportions() {
        // 测试心形比例在不同窗口尺寸下的正确性
        let test_sizes = vec![(300, 400), (400, 300), (600, 400)];
        
        for (width, height) in test_sizes {
            let mask = ShapeMask::new(ShapeType::Heart, width, height);
            let data = mask.data();
            
            // 计算心形的边界框
            let mut min_x = width;
            let mut max_x = 0;
            let mut min_y = height;
            let mut max_y = 0;
            
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    if data[idx] > 0 {
                        min_x = min_x.min(x);
                        max_x = max_x.max(x);
                        min_y = min_y.min(y);
                        max_y = max_y.max(y);
                    }
                }
            }
            
            let heart_width = max_x - min_x;
            let heart_height = max_y - min_y;
            
            // 心形应该基于较小边进行缩放，保持合理的宽高比
            let expected_size = width.min(height) as f32 * 0.8; // 约80%的较小边
            let actual_size = heart_width.max(heart_height) as f32;
            
            let size_ratio = actual_size / expected_size;
            assert!(size_ratio >= 0.8 && size_ratio <= 1.2, 
                    "心形尺寸比例不正确: {:.2} (期望接近1.0)", size_ratio);
        }
    }

    #[test]
    fn test_edge_smoothness() {
        // 测试所有形状的边缘平滑度
        let shapes = vec![
            ShapeType::Circle,
            ShapeType::Ellipse,
            ShapeType::RoundedRectangle { radius: 10.0 },
            ShapeType::Heart,
        ];
        
        for shape in shapes {
            let mask = ShapeMask::new(shape, 200, 200);
            let data = mask.data();
            
            // 计算边缘像素的数量（alpha值在1-254之间的像素）
            let mut edge_pixels = 0;
            let mut total_edge_transitions = 0;
            
            for y in 1..(200-1) {
                for x in 1..(200-1) {
                    let idx = (y * 200 + x) as usize;
                    let current = data[idx];
                    
                    // 检查是否是边缘像素（有抗锯齿效果）
                    if current > 0 && current < 255 {
                        edge_pixels += 1;
                    }
                    
                    // 检查与邻居的过渡是否平滑
                    let neighbors = [
                        data[((y-1) * 200 + x) as usize],
                        data[((y+1) * 200 + x) as usize],
                        data[(y * 200 + (x-1)) as usize],
                        data[(y * 200 + (x+1)) as usize],
                    ];
                    
                    for &neighbor in &neighbors {
                        let diff = (current as i16 - neighbor as i16).abs();
                        if diff > 128 {
                            total_edge_transitions += 1;
                        }
                    }
                }
            }
            
            // 应该有一定数量的边缘像素（表示有抗锯齿）
            assert!(edge_pixels > 0, "形状 {:?} 应该有抗锯齿边缘像素", shape);
            
            // 急剧的边缘过渡应该相对较少（表示边缘平滑）
            let total_pixels = 200 * 200;
            let harsh_transition_ratio = total_edge_transitions as f32 / total_pixels as f32;
            assert!(harsh_transition_ratio < 0.1, 
                    "形状 {:?} 的急剧边缘过渡过多: {:.3}", shape, harsh_transition_ratio);
        }
    }

    #[test]
    fn test_high_quality_generation() {
        // 测试高质量生成模式
        let mut mask = ShapeMask::new(ShapeType::Circle, 100, 100);
        
        // 生成高质量版本
        mask.generate_high_quality();
        let hq_data = mask.data();
        
        // 生成标准版本进行比较
        mask.generate();
        let std_data = mask.data();
        
        // 高质量版本应该有更多的中间alpha值（更平滑的边缘）
        let mut hq_intermediate_pixels = 0;
        let mut std_intermediate_pixels = 0;
        
        for i in 0..hq_data.len() {
            if hq_data[i] > 0 && hq_data[i] < 255 {
                hq_intermediate_pixels += 1;
            }
            if std_data[i] > 0 && std_data[i] < 255 {
                std_intermediate_pixels += 1;
            }
        }
        
        // 高质量版本应该有至少相同数量的中间像素
        assert!(hq_intermediate_pixels >= std_intermediate_pixels,
                "高质量版本应该有更多的抗锯齿像素: {} vs {}", 
                hq_intermediate_pixels, std_intermediate_pixels);
    }

    #[test]
    fn test_antialiasing_performance() {
        // 测试抗锯齿不会显著影响性能
        let mut mask = ShapeMask::new(ShapeType::Heart, 400, 400);
        
        let start = std::time::Instant::now();
        mask.generate_high_quality();
        let duration = start.elapsed();
        
        // 即使是高质量生成，也应该在合理时间内完成（< 200ms）
        assert!(duration.as_millis() < 200, 
                "高质量形状生成耗时过长: {}ms", duration.as_millis());
    }
}
