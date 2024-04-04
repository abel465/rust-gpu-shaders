use crate::{
    context::GraphicsContext,
    controller::{new_controller, Controller},
    render_pass::RenderPass,
    shader::{self, CompiledShaderModules},
    texture::Texture,
    ui::{Ui, UiState},
    window::Window,
    Options, RustGPUShader,
};
use strum::IntoEnumIterator;
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, WindowEvent},
};

pub struct State {
    rpass: RenderPass,
    ctx: GraphicsContext,
    controllers: Vec<Box<dyn Controller>>,
    ui: Ui,
    ui_state: UiState,
    depth_texture: Texture,
}

impl State {
    pub async fn new(
        window: &Window,
        compiled_shader_modules: CompiledShaderModules,
        options: Options,
    ) -> Self {
        let ctx = GraphicsContext::new(&window.window, &options).await;

        let ui = Ui::new(window);

        let ui_state = UiState::new(options.shader);

        let controllers = RustGPUShader::iter()
            .map(|s| new_controller(s, window.window.inner_size()))
            .collect::<Vec<Box<dyn Controller>>>();

        let controller = &controllers[ui_state.active_shader as usize];

        let rpass = RenderPass::new(
            &ctx,
            compiled_shader_modules,
            options,
            &controller.buffers(),
        );

        let depth_texture =
            Texture::create_depth_texture(&ctx.device, &ctx.config, "depth_texture");

        Self {
            rpass,
            controllers,
            ctx,
            ui,
            ui_state,
            depth_texture,
        }
    }

    fn controller(&mut self) -> &mut dyn Controller {
        &mut *self.controllers[self.ui_state.active_shader as usize]
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width != 0 && size.height != 0 {
            self.ctx.config.width = size.width;
            self.ctx.config.height = size.height;
            self.ctx
                .surface
                .configure(&self.ctx.device, &self.ctx.config);
            self.controller().resize(size);
            self.depth_texture =
                Texture::create_depth_texture(&self.ctx.device, &self.ctx.config, "depth_texture");
        }
    }

    pub fn keyboard_input(&mut self, input: KeyboardInput) {
        self.controller().keyboard_input(input);
    }

    pub fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        self.controller().mouse_input(state, button);
    }

    pub fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.controller().mouse_move(position);
    }

    pub fn mouse_delta(&mut self, position: (f64, f64)) {
        self.controller().mouse_delta(position);
    }

    pub fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        self.controller().mouse_scroll(delta);
    }

    pub fn update(&mut self) {
        self.controller().update();
    }

    pub fn render(&mut self, window: &egui_winit::winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let controller = &mut *self.controllers[self.ui_state.active_shader as usize];
        let depth_texture = controller
            .buffers()
            .use_depth_buffer
            .then_some(&self.depth_texture);

        self.rpass.render(
            &self.ctx,
            window,
            &mut self.ui,
            &mut self.ui_state,
            controller,
            depth_texture,
        )
    }

    pub fn update_and_render(
        &mut self,
        window: &egui_winit::winit::window::Window,
    ) -> Result<(), wgpu::SurfaceError> {
        self.update();
        self.render(window)
    }

    pub fn ui_consumes_event(&mut self, event: &WindowEvent) -> bool {
        self.ui.consumes_event(event)
    }

    pub fn new_module(&mut self, shader: RustGPUShader, new_module: CompiledShaderModules) {
        let controller = &self.controllers[shader as usize];
        let buffers = controller.buffers();
        self.ui_state.active_shader = shader;
        self.rpass.new_module(&self.ctx, new_module, &buffers);
    }

    pub fn new_buffers(&mut self) {
        let controller = &self.controllers[self.ui_state.active_shader as usize];
        self.rpass.new_buffers(&self.ctx, &controller.buffers());
    }

    pub fn switch_shader(&mut self, shader: RustGPUShader) {
        self.new_module(
            shader,
            shader::maybe_watch(
                &Options {
                    force_spirv_passthru: false,
                    shader,
                },
                None,
            ),
        )
    }

    pub fn set_vsync(&mut self, enable: bool) {
        self.ctx.set_vsync(enable);
    }

    pub fn cursor_visible(&self) -> bool {
        let controller = &self.controllers[self.ui_state.active_shader as usize];
        controller.cursor_visible()
    }
}
