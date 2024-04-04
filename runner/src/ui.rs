use egui::{
    epaint::{textures::TexturesDelta, ClippedPrimitive},
    vec2, Align2, Context, Layout, Vec2,
};
use strum::IntoEnumIterator;
use egui_winit::winit::{event::WindowEvent, event_loop::EventLoopProxy};

use crate::{
    controller::Controller,
    fps_counter::FpsCounter,
    window::{UserEvent, Window},
    RustGPUShader,
};

pub struct UiState {
    pub fps: usize,
    pub show_fps: bool,
    pub vsync: bool,
    pub active_shader: RustGPUShader,
}

impl UiState {
    pub fn new(active_shader: RustGPUShader) -> Self {
        Self {
            fps: 0,
            show_fps: true,
            vsync: true,
            active_shader,
        }
    }
}

pub struct Ui {
    context: Context,
    egui_winit_state: egui_winit::State,
    event_proxy: EventLoopProxy<UserEvent>,
    fps_counter: FpsCounter,
}

impl Ui {
    pub fn new(window: &Window) -> Self {
        let event_loop = &window.event_loop;
        let mut egui_winit_state = egui_winit::State::new(event_loop);
        egui_winit_state.set_pixels_per_point(window.window.scale_factor() as f32);

        Self {
            context: Context::default(),
            egui_winit_state,
            event_proxy: event_loop.create_proxy(),
            fps_counter: FpsCounter::new(),
        }
    }

    pub fn consumes_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_winit_state
            .on_event(&self.context, &event)
            .consumed
    }

    pub fn prepare(
        &mut self,
        window: &egui_winit::winit::window::Window,
        ui_state: &mut UiState,
        controller: &mut dyn Controller,
    ) -> (Vec<ClippedPrimitive>, TexturesDelta) {
        ui_state.fps = self.fps_counter.tick();
        let raw_input = self.egui_winit_state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, |ctx| {
            self.ui(ctx, ui_state, controller);
        });
        self.egui_winit_state.handle_platform_output(
            &window,
            &self.context,
            full_output.platform_output,
        );
        let clipped_primitives = self.context.tessellate(full_output.shapes);
        (clipped_primitives, full_output.textures_delta)
    }

    fn send_event(&self, event: UserEvent) {
        let _ = self.event_proxy.send_event(event);
    }

    fn ui(&self, ctx: &Context, ui_state: &mut UiState, controller: &mut dyn Controller) {
        let window_margin = 10.0;
        egui::Window::new("Shaders")
            .resizable(false)
            .anchor(Align2::LEFT_TOP, Vec2::splat(window_margin))
            .default_width(140.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::default(), |ui| {
                    for shader in RustGPUShader::iter() {
                        if ui
                            .selectable_label(ui_state.active_shader == shader, shader.to_string())
                            .clicked()
                        {
                            if ui_state.active_shader != shader {
                                self.send_event(UserEvent::SwitchShader(shader));
                            }
                        }
                    }
                });
                ui.separator();
                ui.checkbox(&mut ui_state.show_fps, "fps counter");
                if ui.checkbox(&mut ui_state.vsync, "V-Sync").clicked() {
                    self.send_event(UserEvent::SetVSync(ui_state.vsync));
                }
            });
        if controller.has_ui() {
            egui::Window::new(ui_state.active_shader.to_string())
                .resizable(false)
                .anchor(Align2::RIGHT_TOP, window_margin * vec2(-1.0, 1.0))
                .default_width(130.0)
                .show(ctx, |ui| {
                    controller.ui(ctx, ui, &self.event_proxy);
                });
        }
        if ui_state.show_fps {
            egui::Window::new("fps")
                .title_bar(false)
                .resizable(false)
                .interactable(false)
                .anchor(Align2::RIGHT_BOTTOM, Vec2::splat(-window_margin))
                .show(ctx, |ui| {
                    ui.label(format!("FPS: {}", ui_state.fps));
                });
        }
    }
}
