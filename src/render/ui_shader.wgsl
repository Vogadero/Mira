// UI 着色器
// 
// 专门用于渲染 2D UI 控件（按钮、菜单等）
// 支持透明度混合和简单几何图形渲染

// UI 顶点输入
struct UIVertexInput {
    @location(0) position: vec2<f32>,  // 像素坐标
    @location(1) color: vec4<f32>,     // 顶点颜色（包含透明度）
}

// UI 顶点输出
struct UIVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_position: vec2<f32>, // 世界坐标，用于圆形计算
}

// UI 统一缓冲区（窗口尺寸和按钮位置信息）
struct UIUniforms {
    window_size: vec2<f32>,           // 窗口尺寸
    close_button_pos: vec2<f32>,      // 关闭按钮位置
    minimize_button_pos: vec2<f32>,   // 最小化按钮位置
    button_size: f32,                 // 按钮尺寸
    _padding: f32,                    // 对齐填充
}

@group(0) @binding(0)
var<uniform> ui_uniforms: UIUniforms;

// UI 顶点着色器 - 处理 2D UI 元素
@vertex
fn ui_vs_main(input: UIVertexInput) -> UIVertexOutput {
    var out: UIVertexOutput;
    
    // 将像素坐标转换为 NDC 坐标 (-1 到 1)
    let ndc_x = (input.position.x / ui_uniforms.window_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (input.position.y / ui_uniforms.window_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = input.color;
    out.world_position = input.position;
    
    return out;
}

// UI 片段着色器 - 渲染半透明 UI 元素
@fragment
fn ui_fs_main(input: UIVertexOutput) -> @location(0) vec4<f32> {
    // 直接使用顶点颜色，支持透明度
    return input.color;
}

// 圆形按钮片段着色器 - 创建圆形按钮效果
@fragment
fn ui_circle_fs_main(input: UIVertexOutput) -> @location(0) vec4<f32> {
    // 计算到最近按钮中心的距离
    let close_center = ui_uniforms.close_button_pos + vec2<f32>(ui_uniforms.button_size / 2.0);
    let minimize_center = ui_uniforms.minimize_button_pos + vec2<f32>(ui_uniforms.button_size / 2.0);
    
    let dist_to_close = length(input.world_position - close_center);
    let dist_to_minimize = length(input.world_position - minimize_center);
    
    let radius = ui_uniforms.button_size / 2.0;
    let edge_softness = 1.0; // 边缘软化像素数
    
    var alpha = input.color.a;
    
    // 检查是否在任一按钮的圆形区域内
    if (dist_to_close <= radius + edge_softness) {
        // 在关闭按钮区域内
        alpha = alpha * (1.0 - smoothstep(radius - edge_softness, radius, dist_to_close));
    } else if (dist_to_minimize <= radius + edge_softness) {
        // 在最小化按钮区域内
        alpha = alpha * (1.0 - smoothstep(radius - edge_softness, radius, dist_to_minimize));
    } else {
        // 不在任何按钮区域内，完全透明
        alpha = 0.0;
    }
    
    return vec4<f32>(input.color.rgb, alpha);
}