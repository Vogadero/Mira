// WGSL 着色器
// 
// 顶点着色器：实现旋转变换矩阵
// 片段着色器：采样视频纹理和遮罩纹理，实现遮罩应用逻辑

// 顶点输入
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// 顶点输出
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// 统一缓冲区（变换矩阵）
struct Uniforms {
    transform: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> uniforms: Uniforms;

// 顶点着色器
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // 应用变换矩阵（包含旋转）
    out.clip_position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    
    return out;
}

// 纹理和采样器
@group(0) @binding(0)
var video_texture: texture_2d<f32>;

@group(0) @binding(1)
var mask_texture: texture_2d<f32>;

@group(0) @binding(2)
var texture_sampler: sampler;

// 片段着色器
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 采样视频纹理
    let video_color = textureSample(video_texture, texture_sampler, input.tex_coords);
    
    // 采样遮罩纹理（alpha 通道）
    let mask_alpha = textureSample(mask_texture, texture_sampler, input.tex_coords).r;
    
    // 应用遮罩：video.rgb * mask.a
    // 输出带透明度的颜色
    return vec4<f32>(video_color.rgb, video_color.a * mask_alpha);
}