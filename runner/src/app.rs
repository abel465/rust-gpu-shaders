use crate::{
    shader::{maybe_watch, CompiledShaderModules},
    state,
    window::{UserEvent, Window},
    Options,
};
use egui_winit::winit::{
    event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{Key, NamedKey},
    window::CursorGrabMode,
};

async fn run(options: Options, window: Window, compiled_shader_modules: CompiledShaderModules) {
    let event_loop = window.event_loop;
    let mut app = state::State::new(
        &window.window,
        event_loop.create_proxy(),
        compiled_shader_modules,
        options,
    )
    .await;

    let exit = event_loop.run(|event, event_loop_window_target| {
        let window = &window.window;
        event_loop_window_target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, window_id }
                if window_id == window.id() && !app.ui_consumes_event(&window, &event) =>
            {
                match event {
                    WindowEvent::RedrawRequested => {
                        window.request_redraw();

                        if let Err(wgpu::SurfaceError::OutOfMemory) = app.update_and_render(&window)
                        {
                            event_loop_window_target.exit()
                        }
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => event_loop_window_target.exit(),
                    WindowEvent::KeyboardInput { event, .. } => app.keyboard_input(event),
                    WindowEvent::Resized(size) => app.resize(size),
                    WindowEvent::MouseInput { state, button, .. } => app.mouse_input(state, button),
                    WindowEvent::MouseWheel { delta, .. } => app.mouse_scroll(delta),
                    WindowEvent::CursorMoved { position, .. } => {
                        if window.has_focus() {
                            if app.cursor_visible() {
                                window.set_cursor_grab(CursorGrabMode::None).unwrap();
                                window.set_cursor_visible(true);
                            } else {
                                window
                                    .set_cursor_grab(CursorGrabMode::Confined)
                                    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
                                    .unwrap();
                                window.set_cursor_visible(false);
                            }
                        }
                        app.mouse_move(position)
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => app.mouse_delta(delta),
            Event::UserEvent(event) => match event {
                UserEvent::NewModule(shader, new_module) => {
                    app.new_module(shader, new_module);
                    window.request_redraw();
                }
                UserEvent::SwitchShader(shader) => {
                    app.switch_shader(shader);
                }
                UserEvent::SetVSync(enable) => {
                    app.set_vsync(enable);
                }
                UserEvent::NewBuffersReady => {
                    app.new_buffers();
                }
            },
            _ => {}
        }
    });
    match exit {
        Result::Err(e) => eprintln!("Event loop Error: {e}"),
        Ok(()) => {}
    }
}

pub fn start(options: Options) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
        } else {
            env_logger::init();
        }
    }

    let window = Window::new();

    // Build the shader before we pop open a window, since it might take a while.
    let initial_shader = maybe_watch(
        &options,
        #[cfg(not(target_arch = "wasm32"))]
        {
            let proxy = window.event_loop.create_proxy();
            Some(Box::new(move |res| {
                match proxy.send_event(UserEvent::NewModule(options.shader, res)) {
                    Ok(it) => it,
                    // ShaderModuleDescriptor is not `Debug`, so can't use unwrap/expect
                    Err(_err) => panic!("Event loop dead"),
                }
            }))
        },
    );

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::WindowExtWebSys;
            // On wasm, append the canvas to the document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
            wasm_bindgen_futures::spawn_local(run(
                options.clone(),
                window,
                initial_shader,
            ));
        } else {
            futures::executor::block_on(run(
                options.clone(),
                window,
                initial_shader,
            ));
        }
    }
}
