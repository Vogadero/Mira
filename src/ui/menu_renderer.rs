// 菜单渲染器
//
// 负责渲染上下文菜单的视觉元素，包括背景、文本、图标、边框等

use crate::ui::context_menu::{ContextMenu, MenuItemType, MenuLayout};
use log::debug;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// 菜单渲染器
pub struct MenuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    
    // 渲染管线
    menu_pipeline: Option<wgpu::RenderPipeline>,
    text_pipeline: Option<wgpu::RenderPipeline>,
    
    // 缓冲区
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    uniform_buffer: Option<wgpu::Buffer>,
    
    // 绑定组
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    uniform_bind_group: Option<wgpu::BindGroup>,
    
    // 字体和文本渲染
    font_atlas: Option<FontAtlas>,
    text_vertices: Vec<TextVertex>,
    
    // 菜单几何体缓存
    menu_geometry: Option<MenuGeometry>,
}

/// 菜单顶点数据
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct MenuVertex {
    position: [f32; 2],  // 屏幕坐标
    color: [f32; 4],     // 颜色（RGBA）
    tex_coords: [f32; 2], // 纹理坐标（用于图标）
}

impl MenuVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,  // position
        1 => Float32x4,  // color
        2 => Float32x2   // tex_coords
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MenuVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// 文本顶点数据
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],  // 屏幕坐标
    tex_coords: [f32; 2], // 字体纹理坐标
    color: [f32; 4],     // 文本颜色
}

impl TextVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,  // position
        1 => Float32x2,  // tex_coords
        2 => Float32x4   // color
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// 菜单统一缓冲区
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct MenuUniforms {
    screen_size: [f32; 2],     // 屏幕尺寸
    menu_position: [f32; 2],   // 菜单位置
    menu_size: [f32; 2],       // 菜单尺寸
    _padding: [f32; 2],        // 对齐填充
}

/// 菜单几何体数据
struct MenuGeometry {
    vertices: Vec<MenuVertex>,
    indices: Vec<u16>,
    vertex_count: u32,
    index_count: u32,
}

/// 简化的字体图集
struct FontAtlas {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    char_map: HashMap<char, CharInfo>,
}

/// 字符信息
struct CharInfo {
    tex_coords: [f32; 4], // [x1, y1, x2, y2] 纹理坐标
    size: [f32; 2],       // 字符尺寸
    bearing: [f32; 2],    // 字符偏移
    advance: f32,         // 字符间距
}

impl MenuRenderer {
    /// 创建菜单渲染器
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_format: wgpu::TextureFormat) -> Result<Self, String> {
        debug!("创建菜单渲染器");
        
        let mut renderer = Self {
            device,
            queue,
            menu_pipeline: None,
            text_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffer: None,
            bind_group_layout: None,
            uniform_bind_group: None,
            font_atlas: None,
            text_vertices: Vec::new(),
            menu_geometry: None,
        };
        
        // 初始化渲染管线
        renderer.init_pipelines(surface_format)?;
        
        // 创建简化的字体图集
        renderer.create_font_atlas()?;
        
        debug!("菜单渲染器创建完成");
        Ok(renderer)
    }
    
    /// 初始化渲染管线
    fn init_pipelines(&mut self, surface_format: wgpu::TextureFormat) -> Result<(), String> {
        debug!("初始化菜单渲染管线");
        
        // 创建绑定组布局
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 统一缓冲区
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 字体纹理
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
            label: Some("menu_bind_group_layout"),
        });
        
        // 创建统一缓冲区
        let uniforms = MenuUniforms {
            screen_size: [1920.0, 1080.0], // 默认值，会在渲染时更新
            menu_position: [0.0, 0.0],
            menu_size: [200.0, 100.0],
            _padding: [0.0, 0.0],
        };
        
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Menu Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // 创建菜单渲染管线布局
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Menu Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // 创建菜单着色器
        let menu_shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Menu Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("menu_shader.wgsl").into()),
        });
        
        // 创建菜单渲染管线
        let menu_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Menu Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &menu_shader,
                entry_point: "menu_vs_main",
                buffers: &[MenuVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &menu_shader,
                entry_point: "menu_fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
        
        // 创建文本渲染管线
        let text_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &menu_shader,
                entry_point: "text_vs_main",
                buffers: &[TextVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &menu_shader,
                entry_point: "text_fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
        
        // 存储组件
        self.bind_group_layout = Some(bind_group_layout);
        self.uniform_buffer = Some(uniform_buffer);
        self.menu_pipeline = Some(menu_pipeline);
        self.text_pipeline = Some(text_pipeline);
        
        debug!("菜单渲染管线初始化完成");
        Ok(())
    }
    
    /// 创建简化的字体图集
    fn create_font_atlas(&mut self) -> Result<(), String> {
        debug!("创建字体图集");
        
        // 创建简化的字体纹理（8x8像素的字符，支持基本ASCII字符）
        let atlas_width = 128;
        let atlas_height = 128;
        let char_size = 8;
        
        // 生成简化的字体数据（位图字体）
        let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];
        let mut char_map = HashMap::new();
        
        // 添加基本字符（简化实现，实际应用中需要真实的字体渲染）
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 .,!?-+()[]{}:;";
        
        for (i, ch) in chars.chars().enumerate() {
            let x = (i % 16) * char_size;
            let y = (i / 16) * char_size;
            
            // 简化的字符渲染（实际应用中需要使用真实字体）
            self.render_simple_char(&mut atlas_data, atlas_width, x, y, char_size, ch);
            
            // 添加字符信息
            let tex_x1 = x as f32 / atlas_width as f32;
            let tex_y1 = y as f32 / atlas_height as f32;
            let tex_x2 = (x + char_size) as f32 / atlas_width as f32;
            let tex_y2 = (y + char_size) as f32 / atlas_height as f32;
            
            char_map.insert(ch, CharInfo {
                tex_coords: [tex_x1, tex_y1, tex_x2, tex_y2],
                size: [char_size as f32, char_size as f32],
                bearing: [0.0, 0.0],
                advance: char_size as f32,
            });
        }
        
        // 创建字体纹理
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("font_atlas_texture"),
            view_formats: &[],
        });
        
        // 上传字体数据
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(atlas_width),
                rows_per_image: Some(atlas_height),
            },
            wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
        );
        
        // 创建纹理视图和采样器
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        // 创建字体图集
        self.font_atlas = Some(FontAtlas {
            texture,
            texture_view,
            sampler,
            char_map,
        });
        
        debug!("字体图集创建完成");
        Ok(())
    }
    
    /// 渲染简化字符到图集
    fn render_simple_char(&self, atlas_data: &mut [u8], atlas_width: u32, x: usize, y: usize, size: usize, ch: char) {
        // 简化的字符渲染（使用基本形状表示字符）
        match ch {
            ' ' => {}, // 空格不渲染
            '.' => {
                // 点
                let center_x = x + size / 2;
                let center_y = y + size - 2;
                if center_x < atlas_width as usize && center_y < atlas_width as usize {
                    atlas_data[center_y * atlas_width as usize + center_x] = 255;
                }
            }
            '-' => {
                // 横线
                let line_y = y + size / 2;
                for px in (x + 1)..(x + size - 1) {
                    if px < atlas_width as usize && line_y < atlas_width as usize {
                        atlas_data[line_y * atlas_width as usize + px] = 255;
                    }
                }
            }
            '|' => {
                // 竖线
                let line_x = x + size / 2;
                for py in (y + 1)..(y + size - 1) {
                    if line_x < atlas_width as usize && py < atlas_width as usize {
                        atlas_data[py * atlas_width as usize + line_x] = 255;
                    }
                }
            }
            _ => {
                // 其他字符用简单矩形表示
                for py in (y + 1)..(y + size - 1) {
                    for px in (x + 1)..(x + size - 1) {
                        if px < atlas_width as usize && py < atlas_width as usize {
                            // 创建简单的字符轮廓
                            if px == x + 1 || px == x + size - 2 || py == y + 1 || py == y + size - 2 {
                                atlas_data[py * atlas_width as usize + px] = 255;
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 渲染菜单
    pub fn render_menu(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        menu: &ContextMenu,
        screen_size: [f32; 2],
    ) -> Result<(), String> {
        if menu.state() != &crate::ui::context_menu::MenuState::Visible {
            return Ok(()); // 菜单不可见，跳过渲染
        }
        
        debug!("渲染上下文菜单");
        
        // 生成菜单几何体
        self.generate_menu_geometry(menu)?;
        
        // 更新统一缓冲区
        self.update_uniforms(menu.layout(), screen_size)?;
        
        // 创建绑定组
        self.create_bind_group()?;
        
        // 创建渲染通道
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Menu Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // 保留现有内容
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        // 渲染菜单背景和边框
        if let (Some(pipeline), Some(bind_group)) = (&self.menu_pipeline, &self.uniform_bind_group) {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            
            if let Some(geometry) = &self.menu_geometry {
                if let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer) {
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..geometry.index_count, 0, 0..1);
                }
            }
        }
        
        // 结束渲染通道以释放借用
        drop(render_pass);
        
        // 渲染文本
        self.render_menu_text(encoder, view, menu)?;
        
        debug!("菜单渲染完成");
        Ok(())
    }
    
    /// 生成菜单几何体
    fn generate_menu_geometry(&mut self, menu: &ContextMenu) -> Result<(), String> {
        let layout = menu.layout();
        let items = menu.get_display_items();
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset = 0u16;
        
        // 菜单背景颜色
        let bg_color = [0.2, 0.2, 0.2, 0.9]; // 深灰色半透明背景
        let border_color = [0.4, 0.4, 0.4, 1.0]; // 边框颜色
        let hover_color = [0.3, 0.3, 0.3, 0.9]; // 悬浮高亮颜色
        
        // 生成菜单背景矩形
        let bg_vertices = self.create_rect_vertices(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
            bg_color,
        );
        vertices.extend(bg_vertices);
        
        // 背景矩形索引
        let bg_indices = [0, 1, 2, 0, 2, 3];
        indices.extend(bg_indices.iter().map(|&i| i + vertex_offset));
        vertex_offset += 4;
        
        // 生成菜单边框
        let border_width = layout.border_width;
        if border_width > 0.0 {
            // 上边框
            let top_border = self.create_rect_vertices(
                layout.position.x,
                layout.position.y,
                layout.size.width,
                border_width,
                border_color,
            );
            vertices.extend(top_border);
            indices.extend(bg_indices.iter().map(|&i| i + vertex_offset));
            vertex_offset += 4;
            
            // 下边框
            let bottom_border = self.create_rect_vertices(
                layout.position.x,
                layout.position.y + layout.size.height - border_width,
                layout.size.width,
                border_width,
                border_color,
            );
            vertices.extend(bottom_border);
            indices.extend(bg_indices.iter().map(|&i| i + vertex_offset));
            vertex_offset += 4;
            
            // 左边框
            let left_border = self.create_rect_vertices(
                layout.position.x,
                layout.position.y,
                border_width,
                layout.size.height,
                border_color,
            );
            vertices.extend(left_border);
            indices.extend(bg_indices.iter().map(|&i| i + vertex_offset));
            vertex_offset += 4;
            
            // 右边框
            let right_border = self.create_rect_vertices(
                layout.position.x + layout.size.width - border_width,
                layout.position.y,
                border_width,
                layout.size.height,
                border_color,
            );
            vertices.extend(right_border);
            indices.extend(bg_indices.iter().map(|&i| i + vertex_offset));
            vertex_offset += 4;
        }
        
        // 生成菜单项背景（悬浮高亮和分隔线）
        let mut current_y = layout.position.y + layout.padding;
        
        for item in items {
            match item.item_type {
                MenuItemType::Separator => {
                    // 分隔线
                    let separator_vertices = self.create_rect_vertices(
                        layout.position.x + layout.padding,
                        current_y + layout.separator_height / 2.0 - 0.5,
                        layout.size.width - layout.padding * 2.0,
                        1.0,
                        [0.5, 0.5, 0.5, 0.8],
                    );
                    vertices.extend(separator_vertices);
                    indices.extend(bg_indices.iter().map(|&i| i + vertex_offset));
                    vertex_offset += 4;
                    
                    current_y += layout.separator_height;
                }
                _ => {
                    // 检查是否为悬浮项
                    let is_hovered = menu.hovered_item() == Some(&item.id);
                    
                    if is_hovered {
                        // 悬浮高亮背景
                        let hover_vertices = self.create_rect_vertices(
                            layout.position.x + 1.0,
                            current_y,
                            layout.size.width - 2.0,
                            layout.item_height,
                            hover_color,
                        );
                        vertices.extend(hover_vertices);
                        indices.extend(bg_indices.iter().map(|&i| i + vertex_offset));
                        vertex_offset += 4;
                    }
                    
                    current_y += layout.item_height;
                }
            }
        }
        
        // 创建GPU缓冲区
        if !vertices.is_empty() {
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Menu Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            
            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Menu Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            
            let vertex_count = vertices.len() as u32;
            let index_count = indices.len() as u32;
            
            self.vertex_buffer = Some(vertex_buffer);
            self.index_buffer = Some(index_buffer);
            
            self.menu_geometry = Some(MenuGeometry {
                vertices,
                indices,
                vertex_count,
                index_count,
            });
        }
        
        Ok(())
    }
    
    /// 创建矩形顶点
    fn create_rect_vertices(&self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) -> Vec<MenuVertex> {
        vec![
            MenuVertex { position: [x, y], color, tex_coords: [0.0, 0.0] },                           // 左上
            MenuVertex { position: [x + width, y], color, tex_coords: [1.0, 0.0] },                  // 右上
            MenuVertex { position: [x + width, y + height], color, tex_coords: [1.0, 1.0] },         // 右下
            MenuVertex { position: [x, y + height], color, tex_coords: [0.0, 1.0] },                 // 左下
        ]
    }
    
    /// 渲染菜单文本
    fn render_menu_text(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, menu: &ContextMenu) -> Result<(), String> {
        // 生成文本顶点
        self.generate_text_vertices(menu)?;
        
        if let (Some(text_pipeline), Some(bind_group), Some(_font_atlas)) = 
            (&self.text_pipeline, &self.uniform_bind_group, &self.font_atlas) {
            
            if !self.text_vertices.is_empty() {
                // 创建文本顶点缓冲区
                let text_vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Text Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.text_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                
                // 创建新的渲染通道用于文本
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Text Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                
                // 切换到文本渲染管线
                render_pass.set_pipeline(text_pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                
                // 绘制文本（使用三角形列表，每个字符6个顶点）
                let vertex_count = self.text_vertices.len() as u32;
                render_pass.draw(0..vertex_count, 0..1);
            }
        }
        
        Ok(())
    }
    
    /// 生成文本顶点
    fn generate_text_vertices(&mut self, menu: &ContextMenu) -> Result<(), String> {
        self.text_vertices.clear();
        
        let layout = menu.layout();
        let items = menu.get_display_items();
        let font_atlas = self.font_atlas.as_ref().ok_or("字体图集未初始化")?;
        
        let mut current_y = layout.position.y + layout.padding;
        let text_color = [1.0, 1.0, 1.0, 1.0]; // 白色文本
        let disabled_color = [0.6, 0.6, 0.6, 1.0]; // 禁用文本颜色
        
        for item in items {
            match item.item_type {
                MenuItemType::Separator => {
                    current_y += layout.separator_height;
                }
                _ => {
                    if !item.text.is_empty() {
                        let color = if item.enabled { text_color } else { disabled_color };
                        let text_x = layout.position.x + layout.padding + 24.0; // 为图标留出空间
                        let text_y = current_y + (layout.item_height - 8.0) / 2.0; // 垂直居中
                        
                        // 渲染文本字符
                        let mut char_x = text_x;
                        for ch in item.text.chars() {
                            if let Some(char_info) = font_atlas.char_map.get(&ch) {
                                // 创建字符的四个顶点（两个三角形）
                                let x1 = char_x;
                                let y1 = text_y;
                                let x2 = char_x + char_info.size[0];
                                let y2 = text_y + char_info.size[1];
                                
                                let tex_coords = char_info.tex_coords;
                                
                                // 第一个三角形
                                self.text_vertices.push(TextVertex {
                                    position: [x1, y1],
                                    tex_coords: [tex_coords[0], tex_coords[1]],
                                    color,
                                });
                                self.text_vertices.push(TextVertex {
                                    position: [x2, y1],
                                    tex_coords: [tex_coords[2], tex_coords[1]],
                                    color,
                                });
                                self.text_vertices.push(TextVertex {
                                    position: [x1, y2],
                                    tex_coords: [tex_coords[0], tex_coords[3]],
                                    color,
                                });
                                
                                // 第二个三角形
                                self.text_vertices.push(TextVertex {
                                    position: [x2, y1],
                                    tex_coords: [tex_coords[2], tex_coords[1]],
                                    color,
                                });
                                self.text_vertices.push(TextVertex {
                                    position: [x2, y2],
                                    tex_coords: [tex_coords[2], tex_coords[3]],
                                    color,
                                });
                                self.text_vertices.push(TextVertex {
                                    position: [x1, y2],
                                    tex_coords: [tex_coords[0], tex_coords[3]],
                                    color,
                                });
                                
                                char_x += char_info.advance;
                            }
                        }
                    }
                    
                    current_y += layout.item_height;
                }
            }
        }
        
        Ok(())
    }
    
    /// 更新统一缓冲区
    fn update_uniforms(&mut self, layout: &MenuLayout, screen_size: [f32; 2]) -> Result<(), String> {
        let uniforms = MenuUniforms {
            screen_size,
            menu_position: [layout.position.x, layout.position.y],
            menu_size: [layout.size.width, layout.size.height],
            _padding: [0.0, 0.0],
        };
        
        if let Some(uniform_buffer) = &self.uniform_buffer {
            self.queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
        
        Ok(())
    }
    
    /// 创建绑定组
    fn create_bind_group(&mut self) -> Result<(), String> {
        if self.uniform_bind_group.is_some() {
            return Ok(()); // 已创建
        }
        
        let (bind_group_layout, uniform_buffer, font_atlas) = match (
            &self.bind_group_layout,
            &self.uniform_buffer,
            &self.font_atlas,
        ) {
            (Some(layout), Some(buffer), Some(atlas)) => (layout, buffer, atlas),
            _ => return Err("绑定组组件未初始化".to_string()),
        };
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&font_atlas.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&font_atlas.sampler),
                },
            ],
            label: Some("menu_bind_group"),
        });
        
        self.uniform_bind_group = Some(bind_group);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalSize;

    #[test]
    fn test_menu_vertex_layout() {
        let desc = MenuVertex::desc();
        assert_eq!(desc.array_stride, std::mem::size_of::<MenuVertex>() as u64);
        assert_eq!(desc.attributes.len(), 3);
    }

    #[test]
    fn test_text_vertex_layout() {
        let desc = TextVertex::desc();
        assert_eq!(desc.array_stride, std::mem::size_of::<TextVertex>() as u64);
        assert_eq!(desc.attributes.len(), 3);
    }

    #[test]
    fn test_menu_uniforms_size() {
        let uniforms = MenuUniforms {
            screen_size: [1920.0, 1080.0],
            menu_position: [100.0, 100.0],
            menu_size: [200.0, 150.0],
            _padding: [0.0, 0.0],
        };
        
        let bytes = bytemuck::cast_slice(&[uniforms]);
        assert_eq!(bytes.len(), std::mem::size_of::<MenuUniforms>());
    }

    #[test]
    fn test_rect_vertices_creation() {
        // 由于需要设备实例，这里只测试数据结构
        let vertices = vec![
            MenuVertex { position: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
            MenuVertex { position: [10.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
            MenuVertex { position: [10.0, 10.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
            MenuVertex { position: [0.0, 10.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },
        ];
        
        assert_eq!(vertices.len(), 4);
        assert_eq!(vertices[0].position, [0.0, 0.0]);
        assert_eq!(vertices[2].position, [10.0, 10.0]);
    }
}