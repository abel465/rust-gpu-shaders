use crate::{
    context::GraphicsContext,
    controller::{BindGroupBufferType, BufferData, Controller, SSBO},
    model::Vertex,
    shader::CompiledShaderModules,
    texture::Texture,
    ui::{Ui, UiState},
    Options,
};
use wgpu::{util::DeviceExt, BindGroupLayout, TextureView};

#[cfg(not(target_arch = "wasm32"))]
mod shaders {
    #[allow(non_upper_case_globals)]
    pub const main_fs: &str = "main_fs";
    #[allow(non_upper_case_globals)]
    pub const main_vs: &str = "main_vs";
}
#[cfg(target_arch = "wasm32")]
mod shaders {
    include!(concat!(env!("OUT_DIR"), "/entry_points.rs"));
}

pub struct RenderPass {
    render_pipeline: wgpu::RenderPipeline,
    ui_renderer: egui_wgpu::Renderer,
    options: Options,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
}

impl RenderPass {
    pub fn new(
        ctx: &GraphicsContext,
        compiled_shader_modules: CompiledShaderModules,
        options: Options,
        buffer_data: &BufferData,
    ) -> Self {
        let layouts = bind_group_layouts(ctx, buffer_data);
        let layout_refs = &layouts.iter().collect::<Vec<_>>();
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: layout_refs,
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    range: 0..shared::push_constants::largest_size() as u32,
                }],
            });

        let render_pipeline = create_pipeline(
            &options,
            &ctx.device,
            &pipeline_layout,
            ctx.config.format,
            compiled_shader_modules,
            buffer_data,
        );
        let vertex_buffer = maybe_create_vertex_buffer(ctx, buffer_data);
        let index_buffer = maybe_create_index_buffer(ctx, buffer_data);
        let bind_groups = maybe_create_bind_groups(ctx, buffer_data);

        let ui_renderer = egui_wgpu::Renderer::new(&ctx.device, ctx.config.format, None, 1);

        Self {
            render_pipeline,
            ui_renderer,
            options,
            vertex_buffer,
            index_buffer,
            bind_groups,
        }
    }

    pub fn render(
        &mut self,
        ctx: &GraphicsContext,
        window: &egui_winit::winit::window::Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut dyn Controller,
        depth_texture: Option<&Texture>,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = match ctx.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(err) => {
                eprintln!("get_current_texture error: {err:?}");
                return match err {
                    wgpu::SurfaceError::Lost => {
                        ctx.surface.configure(&ctx.device, &ctx.config);
                        Ok(())
                    }
                    _ => Err(err),
                };
            }
        };
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_shader(ctx, &output_view, controller, depth_texture);
        self.render_ui(ctx, &output_view, window, ui, ui_state, controller);

        output.present();

        Ok(())
    }

    fn render_shader(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &TextureView,
        controller: &dyn Controller,
        depth_texture: Option<&Texture>,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shader Encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shader Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(if self.index_buffer.is_some() {
                            wgpu::Color::BLACK
                        } else {
                            wgpu::Color::GREEN
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: depth_texture.map(|depth_texture| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                }),
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_push_constants(
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                0,
                controller.push_constants(),
            );
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                rpass.set_bind_group(i as u32, bind_group, &[]);
            }
            if let Some(vertex_buffer) = &self.vertex_buffer {
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                if let Some(index_buffer) = &self.index_buffer {
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    let num_indices =
                        index_buffer.size() as u32 / std::mem::size_of::<u32>() as u32;
                    rpass.draw_indexed(0..num_indices, 0, 0..1);
                } else {
                    let num_vertices =
                        vertex_buffer.size() as u32 / std::mem::size_of::<Vertex>() as u32;
                    rpass.draw(0..num_vertices, 0..1);
                }
            } else {
                rpass.draw(0..3, 0..1);
            }
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    fn render_ui(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &TextureView,
        window: &egui_winit::winit::window::Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut dyn Controller,
    ) {
        let (clipped_primitives, textures_delta) = ui.prepare(window, ui_state, controller);

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [ctx.config.width, ctx.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, delta) in &textures_delta.set {
            self.ui_renderer
                .update_texture(&ctx.device, &ctx.queue, *id, &delta);
        }

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UI Encoder"),
            });

        self.ui_renderer.update_buffers(
            &ctx.device,
            &ctx.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            for id in &textures_delta.free {
                self.ui_renderer.free_texture(id);
            }

            self.ui_renderer
                .render(&mut rpass, &clipped_primitives, &screen_descriptor);
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    pub fn new_module(
        &mut self,
        ctx: &GraphicsContext,
        new_module: CompiledShaderModules,
        buffer_data: &BufferData,
    ) {
        self.new_buffers(ctx, buffer_data);
        let layouts = bind_group_layouts(ctx, buffer_data);
        let layout_refs = &layouts.iter().collect::<Vec<_>>();
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: layout_refs,
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    range: 0..shared::push_constants::largest_size() as u32,
                }],
            });
        self.render_pipeline = create_pipeline(
            &self.options,
            &ctx.device,
            &pipeline_layout,
            ctx.config.format,
            new_module,
            buffer_data,
        );
    }

    pub fn new_buffers(&mut self, ctx: &GraphicsContext, buffer_data: &BufferData) {
        self.vertex_buffer = maybe_create_vertex_buffer(ctx, buffer_data);
        self.index_buffer = maybe_create_index_buffer(ctx, buffer_data);
        self.bind_groups = maybe_create_bind_groups(ctx, buffer_data);
    }
}

fn maybe_create_vertex_buffer(
    ctx: &GraphicsContext,
    buffer_data: &BufferData,
) -> Option<wgpu::Buffer> {
    buffer_data.vertex.map(|vertices| {
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
    })
}

fn maybe_create_index_buffer(
    ctx: &GraphicsContext,
    buffer_data: &BufferData,
) -> Option<wgpu::Buffer> {
    buffer_data.index.map(|indices| {
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            })
    })
}

fn maybe_create_bind_groups(
    ctx: &GraphicsContext,
    buffer_data: &BufferData,
) -> Vec<wgpu::BindGroup> {
    buffer_data
        .bind_group_buffers
        .iter()
        .zip(bind_group_layouts(ctx, buffer_data))
        .enumerate()
        .map(|(i, (buffer, layout))| {
            let buffer = ctx.device.create_buffer_init(&match buffer {
                BindGroupBufferType::SSBO(ssbo) => wgpu::util::BufferInitDescriptor {
                    label: Some("Bind Group Buffer"),
                    contents: ssbo.data,
                    usage: wgpu::BufferUsages::STORAGE,
                },
                BindGroupBufferType::Uniform(uniform) => wgpu::util::BufferInitDescriptor {
                    label: Some("Bind Group Buffer"),
                    contents: uniform.data,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            });
            ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some(&format!("bind_group {}", i)),
            })
        })
        .collect()
}

fn create_pipeline(
    options: &Options,
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    surface_format: wgpu::TextureFormat,
    compiled_shader_modules: CompiledShaderModules,
    buffer_data: &BufferData,
) -> wgpu::RenderPipeline {
    // FIXME(eddyb) automate this decision by default.
    let create_module = |module| {
        if options.force_spirv_passthru {
            unsafe { device.create_shader_module_spirv(&module) }
        } else {
            let wgpu::ShaderModuleDescriptorSpirV { label, source } = module;
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::SpirV(source),
            })
        }
    };

    let vs_entry_point = shaders::main_vs;
    let fs_entry_point = shaders::main_fs;

    let vs_module_descr = compiled_shader_modules.spv_module_for_entry_point(vs_entry_point);
    let fs_module_descr = compiled_shader_modules.spv_module_for_entry_point(fs_entry_point);

    // HACK(eddyb) avoid calling `device.create_shader_module` twice unnecessarily.
    let vs_fs_same_module = std::ptr::eq(&vs_module_descr.source[..], &fs_module_descr.source[..]);

    let vs_module = &create_module(vs_module_descr);
    let fs_module;
    let fs_module = if vs_fs_same_module {
        vs_module
    } else {
        fs_module = create_module(fs_module_descr);
        &fs_module
    };

    let buffers_empty = &[];
    let buffers = &[crate::model::Vertex::desc()];

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vs_module,
            entry_point: vs_entry_point,
            buffers: if buffer_data.vertex.is_some() {
                buffers
            } else {
                buffers_empty
            },
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: if buffer_data.use_depth_buffer {
            Some(wgpu::DepthStencilState {
                format: crate::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        } else {
            None
        },
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: fs_module,
            entry_point: fs_entry_point,
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    })
}

fn bind_group_layouts(ctx: &GraphicsContext, buffer_data: &BufferData) -> Vec<BindGroupLayout> {
    buffer_data
        .bind_group_buffers
        .iter()
        .enumerate()
        .map(|(i, buffer)| {
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: (match buffer {
                                BindGroupBufferType::Uniform(_) => wgpu::BufferBindingType::Uniform,
                                BindGroupBufferType::SSBO(SSBO { read_only, .. }) => {
                                    wgpu::BufferBindingType::Storage {
                                        read_only: *read_only,
                                    }
                                }
                            }),
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some(&format!("bind_group_layout {}", i)),
                })
        })
        .collect()
}
