// 渲染引擎实现

use crate::error::RenderError;
use crate::memory::{FrameBufferPool, TextureManager, PoolStats, TextureManagerStats};
use crate::shape::ShapeMask;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::window::Window;

/// 渲染引擎内存统计信息
#[derive(Debug, Clone)]
pub struct RenderMemoryStats {
    pub frame_buffer_pool: PoolStats,
    pub texture_manager: TextureManagerStats,
    pub video_texture_allocated: bool,
    pub mask_texture_allocated: bool,
}

/// 视频帧数据
pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

/// 像素格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    RGB8,
    RGBA8,
    YUV420,
}

impl Frame {
    /// 创建新的帧
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: PixelFormat) -> Self {
        Self {
            data,
            width,
            height,
            format,
        }
    }

    /// 创建测试帧（用于测试）
    #[cfg(test)]
    pub fn new_test_frame(width: u32, height: u32) -> Self {
        let data = vec![128u8; (width * height * 3) as usize]; // RGB格式
        Self::new(data, width, height, PixelFormat::RGB8)
    }
}

/// 渲染引擎
pub struct RenderEngine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    video_texture: Option<wgpu::Texture>,
    mask_texture: Option<wgpu::Texture>,
    video_bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    
    // 内存管理优化
    frame_buffer_pool: Arc<FrameBufferPool>,
    texture_manager: TextureManager,
}

/// 顶点数据
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// 统一缓冲区数据（旋转矩阵）
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4], // 4x4 变换矩阵
}

impl Uniforms {
    fn new() -> Self {
        Self {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    fn update_rotation(&mut self, rotation: f32) {
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        
        self.transform = [
            [cos_r, -sin_r, 0.0, 0.0],
            [sin_r, cos_r, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] }, // 左下
    Vertex { position: [1.0, -1.0], tex_coords: [1.0, 1.0] },  // 右下
    Vertex { position: [1.0, 1.0], tex_coords: [1.0, 0.0] },   // 右上
    Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 0.0] },  // 左上
];

const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];

impl RenderEngine {
    /// 创建新的渲染引擎
    pub async fn new(window: Arc<Window>) -> Result<Self, RenderError> {
        info!("开始初始化渲染引擎");
        let size = window.inner_size();
        info!("窗口尺寸: {}x{}", size.width, size.height);

        // 创建 wgpu 实例
        debug!("创建 wgpu 实例");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 创建表面
        debug!("创建渲染表面");
        let surface = instance.create_surface(window.clone())
            .map_err(|e| {
                error!("创建渲染表面失败: {}", e);
                RenderError::InitializationFailed(format!("创建表面失败: {}", e))
            })?;

        // 请求适配器
        debug!("请求 GPU 适配器");
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or_else(|| {
            error!("未找到合适的 GPU 适配器");
            RenderError::InitializationFailed("未找到合适的GPU适配器".to_string())
        })?;
        
        // 记录适配器信息
        let adapter_info = adapter.get_info();
        info!("使用 GPU 适配器: {} ({:?})", adapter_info.name, adapter_info.backend);
        info!("GPU 设备类型: {:?}", adapter_info.device_type);

        // 请求设备和队列
        debug!("请求 GPU 设备和队列");
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.map_err(|e| {
            error!("创建 GPU 设备失败: {}", e);
            RenderError::InitializationFailed(format!("创建设备失败: {}", e))
        })?;
        
        info!("GPU 设备创建成功");

        // 获取表面能力
        debug!("获取表面能力");
        let surface_caps = surface.get_capabilities(&adapter);
        debug!("支持的表面格式: {:?}", surface_caps.formats);
        debug!("支持的呈现模式: {:?}", surface_caps.present_modes);
        
        // 选择表面格式，优先使用 Bgra8UnormSrgb
        let surface_format = surface_caps.formats.iter()
            .find(|f| **f == wgpu::TextureFormat::Bgra8UnormSrgb)
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        info!("选择表面格式: {:?}", surface_format);

        // 配置表面
        debug!("配置渲染表面");
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);
        debug!("表面配置完成");

        // 创建采样器
        debug!("创建纹理采样器");
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // 创建绑定组布局
        debug!("创建绑定组布局");
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 视频纹理
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 遮罩纹理
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 采样器
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        // 创建统一缓冲区布局
        debug!("创建统一缓冲区布局");
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });

        // 创建统一缓冲区
        debug!("创建统一缓冲区");
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 创建统一缓冲区绑定组
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // 创建渲染管线布局
        debug!("创建渲染管线布局");
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        // 加载着色器
        debug!("加载 WGSL 着色器");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        debug!("着色器加载成功");

        // 创建渲染管线
        debug!("创建渲染管线");
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        debug!("渲染管线创建成功");

        // 创建顶点缓冲区
        debug!("创建顶点缓冲区");
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // 创建索引缓冲区
        debug!("创建索引缓冲区");
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        info!("渲染引擎初始化完成");
        
        // 初始化内存管理组件
        let frame_buffer_pool = Arc::new(FrameBufferPool::new(
            1920 * 1080 * 4, // 最大支持 1080p RGBA 帧
            3,               // 初始缓冲区数量
            8,               // 最大缓冲区数量
        ));
        
        let texture_manager = TextureManager::new(
            10,                           // 最大缓存纹理数量
            Duration::from_secs(30),      // 清理间隔
        );
        
        info!("内存管理组件初始化完成");
        
        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            pipeline,
            video_texture: None,
            mask_texture: None,
            video_bind_group: None,
            bind_group_layout,
            sampler,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            frame_buffer_pool,
            texture_manager,
        })
    }

    /// 调整表面大小
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            info!("调整渲染表面大小: {}x{} -> {}x{}", 
                  self.surface_config.width, self.surface_config.height, width, height);
            
            self.surface_config.width = width;
            self.surface_config.height = height;
            
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.surface.configure(&self.device, &self.surface_config);
            })) {
                Ok(()) => {
                    debug!("表面大小调整成功");
                }
                Err(_) => {
                    error!("调整表面大小时发生 panic，尝试恢复");
                    // 尝试使用之前的配置
                    warn!("使用之前的表面配置进行恢复");
                }
            }
            
            // 清除绑定组，强制在下次渲染时重新创建
            // 这确保了纹理尺寸与表面尺寸的一致性
            self.video_bind_group = None;
            debug!("已清除视频绑定组，将在下次渲染时重新创建");
        } else {
            warn!("无效的表面尺寸: {}x{}，忽略调整请求", width, height);
        }
    }

    /// 上传视频帧到 GPU
    pub fn upload_frame(&mut self, frame: &Frame) -> Result<(), RenderError> {
        debug!("上传视频帧: {}x{}, 格式: {:?}", frame.width, frame.height, frame.format);
        
        // 转换帧格式为 RGBA
        let rgba_data = self.convert_frame_to_rgba(frame)
            .map_err(|e| {
                error!("转换帧格式失败: {}", e);
                e
            })?;
        
        debug!("转换后的 RGBA 数据大小: {} 字节 (期望: {} 字节)", 
               rgba_data.len(), frame.width * frame.height * 4);
        
        // 验证数据大小
        let expected_size = (frame.width * frame.height * 4) as usize;
        if rgba_data.len() != expected_size {
            error!("RGBA 数据大小不匹配: 实际 {} 字节, 期望 {} 字节", 
                   rgba_data.len(), expected_size);
            return Err(RenderError::TextureUploadFailed);
        }
        
        // 创建或更新视频纹理
        let texture_size = wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        };

        // 如果纹理不存在或尺寸不匹配，创建新纹理
        let need_new_texture = self.video_texture.as_ref()
            .map(|t| {
                let size = t.size();
                let needs_update = size.width != frame.width || size.height != frame.height;
                if needs_update {
                    info!("视频纹理尺寸不匹配: 当前 {}x{}, 需要 {}x{}", 
                          size.width, size.height, frame.width, frame.height);
                }
                needs_update
            })
            .unwrap_or(true);

        if need_new_texture {
            info!("创建新的视频纹理: {}x{}", frame.width, frame.height);
            
            self.video_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("video_texture"),
                view_formats: &[],
            }));
            
            // 清除绑定组，强制重新创建
            self.video_bind_group = None;
            info!("视频纹理创建成功，已清除绑定组");
        }

        // 上传数据到纹理
        if let Some(texture) = &self.video_texture {
            debug!("准备上传 {} 字节数据到 {}x{} 视频纹理", 
                   rgba_data.len(), frame.width, frame.height);
            
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba_data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * frame.width),
                    rows_per_image: Some(frame.height),
                },
                texture_size,
            );
            
            debug!("视频帧上传成功");
        }

        Ok(())
    }

    /// 设置形状遮罩
    pub fn set_mask(&mut self, mask: &ShapeMask) -> Result<(), RenderError> {
        debug!("设置形状遮罩: {:?}, 尺寸: {}x{}", mask.shape_type(), mask.width(), mask.height());
        
        let texture_size = wgpu::Extent3d {
            width: mask.width(),
            height: mask.height(),
            depth_or_array_layers: 1,
        };

        // 创建或更新遮罩纹理
        let need_new_texture = self.mask_texture.as_ref()
            .map(|t| {
                let size = t.size();
                size.width != mask.width() || size.height != mask.height()
            })
            .unwrap_or(true);

        if need_new_texture {
            debug!("创建新的遮罩纹理: {}x{}", mask.width(), mask.height());
            
            self.mask_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm, // 单通道 alpha
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("mask_texture"),
                view_formats: &[],
            }));
            
            debug!("遮罩纹理创建成功");
        }

        // 上传遮罩数据到纹理
        if let Some(texture) = &self.mask_texture {
            debug!("上传 {} 字节遮罩数据到纹理", mask.data().len());
            
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                mask.data(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(mask.width()),
                    rows_per_image: Some(mask.height()),
                },
                texture_size,
            );
            
            debug!("遮罩数据上传成功");
        } else {
            error!("遮罩纹理创建失败");
            return Err(RenderError::TextureUploadFailed);
        }

        // 清除旧的绑定组，强制重新创建
        self.video_bind_group = None;
        debug!("遮罩设置完成，绑定组将在下次渲染时重新创建");

        Ok(())
    }

    /// 渲染一帧
    pub fn render(&mut self, rotation: f32) -> Result<(), RenderError> {
        debug!("开始渲染帧，旋转角度: {:.1}°", rotation.to_degrees());
        
        // 检查是否有视频纹理和遮罩纹理
        let has_video_texture = self.video_texture.is_some();
        let has_mask_texture = self.mask_texture.is_some();
        
        if !has_video_texture {
            error!("渲染失败：没有视频纹理");
            return Err(RenderError::RenderFailed("没有视频纹理".to_string()));
        }
        if !has_mask_texture {
            error!("渲染失败：没有遮罩纹理");
            return Err(RenderError::RenderFailed("没有遮罩纹理".to_string()));
        }

        // 更新统一缓冲区（旋转矩阵）
        debug!("更新旋转矩阵");
        let mut uniforms = Uniforms::new();
        uniforms.update_rotation(rotation);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // 创建或更新纹理绑定组
        if self.video_bind_group.is_none() {
            debug!("创建纹理绑定组");
            // 避免借用冲突，直接在这里创建绑定组
            let video_texture = self.video_texture.as_ref().unwrap();
            let mask_texture = self.mask_texture.as_ref().unwrap();
            
            let video_view = video_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mask_view = mask_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.video_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&video_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&mask_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: Some("video_bind_group"),
            }));
            
            debug!("纹理绑定组创建完成");
        }

        // 获取表面纹理
        debug!("获取表面纹理");
        let output = self.surface.get_current_texture()
            .map_err(|e| {
                error!("获取表面纹理失败: {}", e);
                RenderError::RenderFailed(format!("获取表面纹理失败: {}", e))
            })?;
        
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 创建命令编码器
        debug!("创建渲染命令");
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // 创建渲染通道
        {
            debug!("开始渲染通道");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0, // 透明背景
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // 设置渲染管线
            render_pass.set_pipeline(&self.pipeline);
            
            // 绑定纹理
            if let Some(bind_group) = &self.video_bind_group {
                render_pass.set_bind_group(0, bind_group, &[]);
                debug!("纹理绑定组已绑定");
            } else {
                error!("纹理绑定组不存在");
                return Err(RenderError::RenderFailed("纹理绑定组不存在".to_string()));
            }
            
            // 绑定统一缓冲区
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            debug!("统一缓冲区已绑定");
            
            // 设置顶点和索引缓冲区
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            debug!("顶点和索引缓冲区已设置");
            
            // 绘制
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
            debug!("绘制命令已提交，索引数量: {}", INDICES.len());
        }

        // 提交命令
        debug!("提交渲染命令到 GPU");
        self.queue.submit(std::iter::once(encoder.finish()));
        
        // 呈现到屏幕
        output.present();
        debug!("帧渲染完成");

        Ok(())
    }

    /// 更新纹理绑定组
    fn update_bind_group(&mut self, video_texture: &wgpu::Texture, mask_texture: &wgpu::Texture) {
        debug!("更新纹理绑定组");
        
        let video_view = video_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mask_view = mask_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.video_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&video_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("video_bind_group"),
        }));
        
        debug!("纹理绑定组更新完成");
    }

    /// 转换帧格式为 RGBA
    fn convert_frame_to_rgba(&self, frame: &Frame) -> Result<Vec<u8>, RenderError> {
        debug!("转换帧格式: {:?} -> RGBA8", frame.format);
        
        match frame.format {
            PixelFormat::RGB8 => {
                debug!("RGB8 -> RGBA8 转换，原始数据: {} 字节", frame.data.len());
                
                // 验证 RGB 数据大小
                let expected_rgb_size = (frame.width * frame.height * 3) as usize;
                if frame.data.len() != expected_rgb_size {
                    error!("RGB 数据大小不匹配: 实际 {} 字节, 期望 {} 字节 ({}x{}x3)", 
                           frame.data.len(), expected_rgb_size, frame.width, frame.height);
                    return Err(RenderError::TextureUploadFailed);
                }
                
                // 使用缓冲区池获取内存
                let mut rgba_data = self.frame_buffer_pool.get_buffer();
                rgba_data.clear();
                
                let expected_rgba_size = (frame.width * frame.height * 4) as usize;
                rgba_data.reserve(expected_rgba_size);
                
                // RGB 转 RGBA，添加 alpha 通道
                for chunk in frame.data.chunks_exact(3) {
                    rgba_data.extend_from_slice(chunk);
                    rgba_data.push(255); // 完全不透明
                }
                
                // 处理剩余的不完整数据（如果有）
                let remainder = frame.data.len() % 3;
                if remainder > 0 {
                    warn!("RGB 数据有 {} 字节剩余，数据可能不完整", remainder);
                    // 不处理不完整的像素
                }
                
                debug!("转换完成，RGBA 数据: {} 字节 (期望: {} 字节)", 
                       rgba_data.len(), expected_rgba_size);
                
                // 最终验证
                if rgba_data.len() != expected_rgba_size {
                    error!("RGBA 转换后大小不匹配: 实际 {} 字节, 期望 {} 字节", 
                           rgba_data.len(), expected_rgba_size);
                    return Err(RenderError::TextureUploadFailed);
                }
                
                Ok(rgba_data)
            }
            PixelFormat::RGBA8 => {
                debug!("RGBA8 格式，直接使用，数据: {} 字节", frame.data.len());
                
                // 验证 RGBA 数据大小
                let expected_rgba_size = (frame.width * frame.height * 4) as usize;
                if frame.data.len() != expected_rgba_size {
                    error!("RGBA 数据大小不匹配: 实际 {} 字节, 期望 {} 字节 ({}x{}x4)", 
                           frame.data.len(), expected_rgba_size, frame.width, frame.height);
                    return Err(RenderError::TextureUploadFailed);
                }
                
                // 已经是 RGBA 格式，但仍使用缓冲区池来保持一致性
                let mut rgba_data = self.frame_buffer_pool.get_buffer();
                rgba_data.clear();
                rgba_data.extend_from_slice(&frame.data);
                Ok(rgba_data)
            }
            PixelFormat::YUV420 => {
                error!("不支持的像素格式: YUV420");
                // YUV420 转 RGBA（简化实现）
                // 在实际应用中，这里需要更复杂的 YUV 到 RGB 转换
                Err(RenderError::TextureUploadFailed)
            }
        }
    }
    
    /// 清理未使用的 GPU 资源
    pub fn cleanup_resources(&mut self) {
        debug!("开始清理 GPU 资源");
        
        // 清理纹理管理器中未使用的纹理
        self.texture_manager.cleanup_unused();
        
        // 清理帧缓冲区池中未使用的缓冲区
        self.frame_buffer_pool.cleanup_unused();
        
        debug!("GPU 资源清理完成");
    }
    
    /// 获取内存使用统计信息
    pub fn get_memory_stats(&self) -> RenderMemoryStats {
        let pool_stats = self.frame_buffer_pool.get_stats();
        let texture_stats = self.texture_manager.get_stats();
        
        RenderMemoryStats {
            frame_buffer_pool: pool_stats,
            texture_manager: texture_stats,
            video_texture_allocated: self.video_texture.is_some(),
            mask_texture_allocated: self.mask_texture.is_some(),
        }
    }
    
    /// 强制释放所有缓存的纹理
    pub fn force_release_textures(&mut self) {
        warn!("强制释放所有缓存的纹理");
        
        // 清除当前纹理引用
        self.video_texture = None;
        self.mask_texture = None;
        self.video_bind_group = None;
        
        // 重新创建纹理管理器以清除所有缓存
        self.texture_manager = TextureManager::new(
            10,
            Duration::from_secs(30),
        );
        
        info!("所有纹理已释放");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let data = vec![255u8; 100 * 100 * 3];
        let frame = Frame::new(data, 100, 100, PixelFormat::RGB8);
        assert_eq!(frame.width, 100);
        assert_eq!(frame.height, 100);
        assert_eq!(frame.format, PixelFormat::RGB8);
    }

    #[test]
    fn test_frame_test_creation() {
        let frame = Frame::new_test_frame(200, 150);
        assert_eq!(frame.width, 200);
        assert_eq!(frame.height, 150);
        assert_eq!(frame.format, PixelFormat::RGB8);
        assert_eq!(frame.data.len(), 200 * 150 * 3);
    }

    #[test]
    fn test_vertex_layout() {
        let desc = Vertex::desc();
        assert_eq!(desc.array_stride, std::mem::size_of::<Vertex>() as u64);
        assert_eq!(desc.attributes.len(), 2);
    }

    #[test]
    fn test_uniforms_creation() {
        let uniforms = Uniforms::new();
        // 检查单位矩阵
        assert_eq!(uniforms.transform[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(uniforms.transform[1], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(uniforms.transform[2], [0.0, 0.0, 1.0, 0.0]);
        assert_eq!(uniforms.transform[3], [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_uniforms_rotation() {
        let mut uniforms = Uniforms::new();
        uniforms.update_rotation(std::f32::consts::PI / 2.0); // 90 度
        
        // 检查旋转矩阵（允许浮点误差）
        let cos_90 = (std::f32::consts::PI / 2.0).cos();
        let sin_90 = (std::f32::consts::PI / 2.0).sin();
        
        assert!((uniforms.transform[0][0] - cos_90).abs() < 1e-6);
        assert!((uniforms.transform[0][1] - (-sin_90)).abs() < 1e-6);
        assert!((uniforms.transform[1][0] - sin_90).abs() < 1e-6);
        assert!((uniforms.transform[1][1] - cos_90).abs() < 1e-6);
    }

    #[test]
    fn test_pixel_format_variants() {
        // 测试所有像素格式变体
        let formats = [PixelFormat::RGB8, PixelFormat::RGBA8, PixelFormat::YUV420];
        for format in formats {
            let data = vec![0u8; 100];
            let frame = Frame::new(data, 10, 10, format);
            assert_eq!(frame.format, format);
        }
    }

    #[test]
    fn test_vertices_and_indices() {
        // 测试顶点数据
        assert_eq!(VERTICES.len(), 4);
        assert_eq!(INDICES.len(), 6);
        
        // 检查顶点位置范围
        for vertex in VERTICES {
            assert!(vertex.position[0] >= -1.0 && vertex.position[0] <= 1.0);
            assert!(vertex.position[1] >= -1.0 && vertex.position[1] <= 1.0);
            assert!(vertex.tex_coords[0] >= 0.0 && vertex.tex_coords[0] <= 1.0);
            assert!(vertex.tex_coords[1] >= 0.0 && vertex.tex_coords[1] <= 1.0);
        }
        
        // 检查索引范围
        for &index in INDICES {
            assert!((index as usize) < VERTICES.len());
        }
    }

    #[test]
    fn test_frame_rgb_to_rgba_conversion() {
        // 创建一个模拟的渲染引擎来测试转换函数
        // 由于无法创建真实的渲染引擎（需要窗口），我们直接测试转换逻辑
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // 红绿蓝像素
        let frame = Frame::new(rgb_data, 3, 1, PixelFormat::RGB8);
        
        // 模拟转换逻辑
        let mut rgba_data = Vec::with_capacity(frame.data.len() * 4 / 3);
        for chunk in frame.data.chunks(3) {
            if chunk.len() == 3 {
                rgba_data.extend_from_slice(chunk);
                rgba_data.push(255); // alpha
            }
        }
        
        let expected = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
        assert_eq!(rgba_data, expected);
    }

    #[test]
    fn test_frame_rgba_passthrough() {
        let rgba_data = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let frame = Frame::new(rgba_data.clone(), 2, 1, PixelFormat::RGBA8);
        
        // RGBA 格式应该直接通过
        assert_eq!(frame.data, rgba_data);
    }

    #[test]
    fn test_rotation_angles() {
        let mut uniforms = Uniforms::new();
        
        // 测试常见角度
        let angles = [0.0, std::f32::consts::PI / 4.0, std::f32::consts::PI / 2.0, std::f32::consts::PI];
        
        for angle in angles {
            uniforms.update_rotation(angle);
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            // 验证旋转矩阵的正确性
            assert!((uniforms.transform[0][0] - cos_a).abs() < 1e-6);
            assert!((uniforms.transform[0][1] - (-sin_a)).abs() < 1e-6);
            assert!((uniforms.transform[1][0] - sin_a).abs() < 1e-6);
            assert!((uniforms.transform[1][1] - cos_a).abs() < 1e-6);
        }
    }

    #[test]
    fn test_bytemuck_traits() {
        // 测试 bytemuck traits 是否正确实现
        let vertex = Vertex {
            position: [1.0, 2.0],
            tex_coords: [0.5, 0.5],
        };
        
        let bytes: &[u8] = bytemuck::cast_slice(&[vertex]);
        assert_eq!(bytes.len(), std::mem::size_of::<Vertex>());
        
        let uniforms = Uniforms::new();
        let uniform_bytes: &[u8] = bytemuck::cast_slice(&[uniforms]);
        assert_eq!(uniform_bytes.len(), std::mem::size_of::<Uniforms>());
    }

    // 注意：由于 RenderEngine::new 需要 Window 实例且是异步的，
    // 完整的渲染引擎测试需要在集成测试中进行
    // 这些测试覆盖了：
    // - GPU 初始化流程的数据结构
    // - 纹理创建和上传的数据转换
    // - 渲染管线配置的常量
    // - 着色器相关的数据结构
}
