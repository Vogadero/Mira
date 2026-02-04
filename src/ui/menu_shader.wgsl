// 菜单渲染着色器

// 统一缓冲区
struct MenuUniforms {
    screen_size: vec2<f32>,
    menu_position: vec2<f32>,
    menu_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: MenuUniforms;

@group(0) @binding(1)
var font_texture: texture_2d<f32>;

@group(0) @binding(2)
var font_sampler: sampler;

// 菜单顶点着色器
struct MenuVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct MenuVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn menu_vs_main(vertex: MenuVertexInput) -> MenuVertexOutput {
    var out: MenuVertexOutput;
    
    // 将屏幕坐标转换为NDC坐标
    let ndc_x = (vertex.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (vertex.position.y / uniforms.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = vertex.color;
    out.tex_coords = vertex.tex_coords;
    
    return out;
}

// 菜单片段着色器
@fragment
fn menu_fs_main(in: MenuVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// 文本顶点着色器
struct TextVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct TextVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn text_vs_main(vertex: TextVertexInput) -> TextVertexOutput {
    var out: TextVertexOutput;
    
    // 将屏幕坐标转换为NDC坐标
    let ndc_x = (vertex.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (vertex.position.y / uniforms.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    
    return out;
}

// 文本片段着色器
@fragment
fn text_fs_main(in: TextVertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(font_texture, font_sampler, in.tex_coords).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}