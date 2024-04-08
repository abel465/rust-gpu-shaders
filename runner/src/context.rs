use crate::Options;
use egui_winit::winit::{dpi::PhysicalSize, window::Window};

pub struct GraphicsContext<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'a> GraphicsContext<'a> {
    pub async fn new(window: &'a Window, options: &Options) -> GraphicsContext<'a> {
        let backends = wgpu::util::backend_bits_from_env()
            .unwrap_or(wgpu::Backends::VULKAN | wgpu::Backends::METAL);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let initial_surface = instance
            .create_surface(window)
            .expect("Failed to create surface from window");

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
            &instance,
            // Request an adapter which can render to our surface
            Some(&initial_surface),
        )
        .await
        .expect("Failed to find an appropriate adapter");

        let mut features = wgpu::Features::PUSH_CONSTANTS;
        if !options.validate_spirv {
            features |= wgpu::Features::SPIRV_SHADER_PASSTHROUGH;
        }
        let limits = wgpu::Limits {
            max_push_constant_size: 128,
            ..Default::default()
        };

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: limits,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        fn auto_configure_surface<'a>(
            adapter: &wgpu::Adapter,
            device: &wgpu::Device,
            surface: wgpu::Surface<'a>,
            size: PhysicalSize<u32>,
        ) -> (wgpu::Surface<'a>, wgpu::SurfaceConfiguration) {
            let mut surface_config = surface
                .get_default_config(adapter, size.width, size.height)
                .unwrap_or_else(|| {
                    panic!(
                        "Missing formats/present modes in surface capabilities: {:#?}",
                        surface.get_capabilities(adapter)
                    )
                });
            surface_config.present_mode = wgpu::PresentMode::AutoVsync;
            surface.configure(device, &surface_config);
            (surface, surface_config)
        }

        let (surface, config) =
            auto_configure_surface(&adapter, &device, initial_surface, window.inner_size());

        GraphicsContext {
            surface,
            device,
            queue,
            config,
        }
    }

    pub fn set_vsync(&mut self, enable: bool) {
        self.config.present_mode = if enable {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.surface.configure(&self.device, &self.config);
    }
}
