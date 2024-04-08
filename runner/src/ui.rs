use crate::{controller::Controller, fps_counter::FpsCounter, window::UserEvent, RustGPUShader};
use egui::{
    epaint::{textures::TexturesDelta, ClippedPrimitive},
    vec2, Align2, Context, Layout, Vec2,
};
use egui_winit::{
    winit::{event::WindowEvent, event_loop::EventLoopProxy, window::Window},
    State,
};
use strum::IntoEnumIterator;

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
    egui_winit_state: State,
    event_proxy: EventLoopProxy<UserEvent>,
    fps_counter: FpsCounter,
}

impl Ui {
    pub fn new(window: &Window, event_proxy: EventLoopProxy<UserEvent>) -> Self {
        let context = Context::default();
        let viewport_id = context.viewport_id();
        let egui_winit_state = State::new(
            context,
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
        );

        Self {
            egui_winit_state,
            event_proxy,
            fps_counter: FpsCounter::new(),
        }
    }

    pub fn consumes_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui_winit_state
            .on_window_event(window, &event)
            .consumed
    }

    pub fn prepare(
        &mut self,
        window: &Window,
        ui_state: &mut UiState,
        controller: &mut dyn Controller,
    ) -> (Vec<ClippedPrimitive>, TexturesDelta) {
        ui_state.fps = self.fps_counter.tick();
        let raw_input = self.egui_winit_state.take_egui_input(&window);
        let full_output = self.egui_winit_state.egui_ctx().run(raw_input, |ctx| {
            self.ui(ctx, ui_state, controller);
        });
        self.egui_winit_state
            .handle_platform_output(&window, full_output.platform_output);
        let clipped_primitives = self
            .egui_winit_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
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
