// 形状遮罩演示程序

use mira::shape::{ShapeMask, ShapeType};
use std::time::Instant;

fn main() {
    println!("Mira 形状遮罩演示");
    println!("==================");

    let shapes = vec![
        ("圆形", ShapeType::Circle),
        ("椭圆形", ShapeType::Ellipse),
        ("矩形", ShapeType::Rectangle),
        ("圆角矩形", ShapeType::RoundedRectangle { radius: 15.0 }),
        ("心形", ShapeType::Heart),
    ];

    let width = 200;
    let height = 200;

    for (name, shape_type) in shapes {
        println!("\n生成 {} 遮罩 ({}x{})...", name, width, height);
        
        let start = Instant::now();
        let mask = ShapeMask::new(shape_type, width, height);
        let generation_time = start.elapsed();
        
        let data = mask.data();
        let opaque_pixels = data.iter().filter(|&&pixel| pixel == 255).count();
        let transparent_pixels = data.len() - opaque_pixels;
        
        println!("  生成时间: {:?}", generation_time);
        println!("  不透明像素: {}", opaque_pixels);
        println!("  透明像素: {}", transparent_pixels);
        println!("  覆盖率: {:.1}%", (opaque_pixels as f32 / data.len() as f32) * 100.0);
        
        // 验证性能要求 (< 100ms)
        if generation_time.as_millis() < 100 {
            println!("  ✓ 性能要求满足 (< 100ms)");
        } else {
            println!("  ✗ 性能要求不满足 (>= 100ms)");
        }
    }

    // 测试形状切换性能
    println!("\n测试形状切换性能...");
    let mut mask = ShapeMask::new(ShapeType::Circle, 400, 400);
    
    let switch_shapes = vec![
        ShapeType::Ellipse,
        ShapeType::Rectangle,
        ShapeType::RoundedRectangle { radius: 20.0 },
        ShapeType::Heart,
        ShapeType::Circle,
    ];

    for shape in switch_shapes {
        let start = Instant::now();
        mask.set_shape(shape);
        let switch_time = start.elapsed();
        
        println!("  切换到 {:?}: {:?}", shape, switch_time);
        
        if switch_time.as_millis() < 100 {
            println!("    ✓ 切换性能满足要求");
        } else {
            println!("    ✗ 切换性能不满足要求");
        }
    }

    println!("\n演示完成！");
}