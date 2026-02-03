// 菜单着色器
//
// 用于渲染上下文菜单的背景、边框、文本等元素

// 菜单顶点输入
struct MenuVertexInput {
    @location(0) position: vec2<f32>,    // 屏幕坐标
    @location(1) color: vec4<f32>,       // 颜色
    @location(2) tex_coords: vec2<f32>,  // 纹理坐标
}

// 菜单顶点输出
struct MenuVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// 文本顶点输入
struct TextVertexInput {
    @location(0) position: vec2<f32>,    // 屏幕坐标
    @location(1) tex_coords: vec2<f32>,  // 字体纹理坐标
    @location(2) color: vec4<f32>,       // 文本颜色
}

// 文本顶点输出
struct TextVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// 菜单统一缓冲区
struct MenuUniforms {
    screen_size: vec2<f32>,     // 屏幕尺寸
    menu_position: vec2<f32>,   // 菜单位置
    menu_size: vec2<f32>,       // 菜单尺寸
    _padding: vec2<f32>,        // 对齐填充
}

@group(0) @binding(0)
var<uniform> uniforms: MenuUniforms;

@group(0) @binding(1)
var font_texture: texture_2d<f32>;

@group(0) @binding(2)
var font_sampler: sampler;

// 菜单顶点着色器 - 处理菜单背景和边框
@vertex
fn menu_vs_main(input: MenuVertexInput) -> MenuVertexOutput {
    var out: MenuVertexOutput;
    
    // 将屏幕坐标转换为 NDC 坐标 (-1 到 1)
    let ndc_x = (input.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (input.position.y / uniforms.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = input.color;
    out.tex_coords = input.tex_coords;
    
    return out;
}

// 菜单片段着色器 - 渲染菜单背景和边框
@fragment
fn menu_fs_main(input: MenuVertexOutput) -> @location(0) vec4<f32> {
    // 直接使用顶点颜色渲染菜单背景和边框
    return input.color;
}

// 文本顶点着色器 - 处理文本渲染
@vertex
fn text_vs_main(input: TextVertexInput) -> TextVertexOutput {
    var out: TextVertexOutput;
    
    // 将屏幕坐标转换为 NDC 坐标 (-1 到 1)
    let ndc_x = (input.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (input.position.y / uniforms.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    out.color = input.color;
    
    return out;
}

// 文本片段着色器 - 渲染文本字符
@fragment
fn text_fs_main(input: TextVertexOutput) -> @location(0) vec4<f32> {
    // 从字体纹理采样字符
    let font_alpha = textureSample(font_texture, font_sampler, input.tex_coords).r;
    
    // 使用字体alpha和文本颜色
    var final_color = input.color;
    final_color.a = final_color.a * font_alpha;
    
    // 如果alpha太低，丢弃片段
    if (final_color.a < 0.01) {
        discard;
    }
    
    return final_color;
}

// 图标片段着色器 - 渲染菜单项图标（未来扩展）
@fragment
fn icon_fs_main(input: MenuVertexOutput) -> @location(0) vec4<f32> {
    // 简单的图标渲染（可以扩展为真实的图标纹理）
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(input.tex_coords, center);
    
    // 创建简单的圆形图标
    if (dist < 0.3) {
        return input.color;
    } else {
        return vec4<f32>(input.color.rgb, 0.0);
    }
}

// 分隔线片段着色器 - 渲染菜单分隔线
@fragment
fn separator_fs_main(input: MenuVertexOutput) -> @location(0) vec4<f32> {
    // 创建渐变分隔线效果
    let gradient = smoothstep(0.0, 0.1, input.tex_coords.x) * 
                   smoothstep(1.0, 0.9, input.tex_coords.x);
    
    var final_color = input.color;
    final_color.a = final_color.a * gradient;
    
    return final_color;
}

// 悬浮高亮片段着色器 - 渲染菜单项悬浮效果
@fragment
fn hover_fs_main(input: MenuVertexOutput) -> @location(0) vec4<f32> {
    // 创建柔和的悬浮高亮效果
    let center_y = 0.5;
    let dist_from_center = abs(input.tex_coords.y - center_y);
    let highlight = 1.0 - smoothstep(0.0, 0.5, dist_from_center);
    
    var final_color = input.color;
    final_color.rgb = final_color.rgb + vec3<f32>(0.1, 0.1, 0.1) * highlight;
    
    return final_color;
}