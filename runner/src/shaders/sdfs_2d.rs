use crate::{
    controller::{BindGroupBufferType, BufferData, SSBO},
    egui_components::{
        enabled_number::EnabledNumber,
        repetition::{Repetition, RepetitionValue},
    },
    window::UserEvent,
};
use bytemuck::Zeroable;
use egui::{Context, CursorIcon};
use glam::{vec2, UVec2, Vec2};
use sdf::grid::{SdfGrid, NUM_X, NUM_Y, BASE};
use shared::push_constants::sdfs_2d::{ShaderConstants, MAX_NUM_POINTS};
use shared::sdf_2d as sdf;
use shared::{fast_optional::Optional_f32, from_pixels};
use std::{
    fs,
    f32::consts::PI,
    time::{Duration, Instant},
};
use strum::IntoEnumIterator;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};
use steel::steel_vm::engine::Engine;
use steel_derive::Steel;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelVal;

#[derive(strum::EnumIter, strum::Display, PartialEq, Copy, Clone)]
pub enum Shape {
    Disk,
    Rectangle,
    EquilateralTriangle,
    IsoscelesTriangle,
    Triangle,
    Capsule,
    Torus,
    Line,
    Plane,
    LineSegment,
    Ray,
    Hexagon,
    Pentagon,
    Polygon,
    Cross,
}

impl Shape {
    fn labels(self) -> &'static [&'static str] {
        use Shape::*;
        const R: &'static str = "Radius";
        const W: &'static str = "Width";
        const H: &'static str = "Height";
        match self {
            Disk | Capsule | Hexagon | Pentagon | EquilateralTriangle => &[R],
            Rectangle | IsoscelesTriangle => &[W, H],
            Torus => &["Major Radius", "Minor Radius"],
            Cross => &["Length", "Thickness"],
            Triangle | Plane | Line | Ray | LineSegment | Polygon => &[],
        }
    }

    fn dim_range(&self) -> &[core::ops::RangeInclusive<f32>] {
        use Shape::*;
        match self {
            Disk | Capsule | EquilateralTriangle | Hexagon | Pentagon => &[0.0..=0.5],
            Rectangle => &[0.0..=1.0, 0.0..=1.0],
            IsoscelesTriangle => &[0.0..=1.0, -0.5..=0.5],
            Torus => &[0.0..=0.5, 0.0..=0.2],
            Cross => &[0.0..=0.5, 0.0..=0.5],
            Triangle | Plane | Line | Ray | LineSegment | Polygon => &[],
        }
    }

    fn default_dims(&self) -> &[f32] {
        use Shape::*;
        match self {
            Disk | Capsule | EquilateralTriangle | Hexagon | Pentagon => &[0.2],
            Rectangle | IsoscelesTriangle => &[0.4, 0.3],
            Torus => &[0.2, 0.1],
            Cross => &[0.35, 0.1],
            Triangle | Plane | Line | Ray | LineSegment | Polygon => &[],
        }
    }

    fn default_points(&self) -> &[[f32; 2]] {
        use Shape::*;
        match self {
            Polygon => &[
                [0.0, 0.3],
                [0.0, -0.3],
                [-0.4, -0.2],
                [0.3, 0.0],
                [-0.4, 0.2],
            ],
            Triangle => &[[-0.1, -0.2], [0.3, 0.2], [0.2, -0.3]],
            Capsule | LineSegment => &[[-0.1, -0.1], [0.2, 0.1]],
            Ray => &[[0.0, 0.0]],
            _ => &[],
        }
    }

    fn default_params(&self) -> Params {
        let default_ps = self.default_points();
        let mut ps = [[1e10; 2]; MAX_NUM_POINTS];
        for i in 0..default_ps.len() {
            ps[i] = default_ps[i];
        }

        let default_dims = self.default_dims();
        let mut dims = [0.0; 2];
        for i in 0..default_dims.len() {
            dims[i] = default_dims[i];
        }

        Params {
            shape: *self,
            dims,
            ps,
            rot: 0.0,
            repeat: RepetitionData::default(),
            pad: Optional_f32::NONE,
            onion: Optional_f32::NONE,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct RepetitionData {
    pub current: RepetitionValue,
    pub dim: Vec2,
    pub n1: UVec2,
    pub n2: UVec2,
}

impl From<Repetition> for RepetitionData {
    fn from(rep: Repetition) -> RepetitionData {
        use RepetitionValue::*;
        let dim = match rep.current {
            None => Vec2::ZERO,
            Unlimited => rep.unlimited,
            Mirrored => rep.mirrored,
            Limited => rep.limited.0,
            Rectangular => vec2(rep.rectangular.0, 0.0),
            Angular => vec2(rep.angular.0, 0.0),
        };
        let n1 = match rep.current {
            None | Unlimited | Mirrored => UVec2::ZERO,
            Limited => rep.limited.1,
            Rectangular => rep.rectangular.1,
            Angular => UVec2::new(rep.angular.1, 0),
        };
        let n2 = match rep.current {
            Limited => rep.limited.2,
            _ => UVec2::ZERO,
        };
        RepetitionData {
            current: rep.current,
            dim: dim.into(),
            n1: n1.into(),
            n2: n2.into(),
        }
    }
}

impl Default for RepetitionData {
    fn default() -> Self {
        Self {
            current: RepetitionValue::None,
            dim: Vec2::ZERO,
            n1: UVec2::ZERO,
            n2: UVec2::ZERO,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Params {
    pub shape: Shape,
    pub dims: [f32; 2],
    pub ps: [[f32; 2]; MAX_NUM_POINTS],
    pub rot: f32,
    pub repeat: RepetitionData,
    pub onion: Optional_f32,
    pub pad: Optional_f32,
}

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    elapsed: Duration,
    cursor: Vec2,
    mouse_button_pressed: bool,
    can_drag: Option<usize>,
    drag_point: Option<usize>,
    shader_constants: ShaderConstants,
    shape: Shape,
    onion: EnabledNumber<f32>,
    pad: EnabledNumber<f32>,
    repeat: Repetition,
    params: Vec<Params>,
    prev_params: Params,
    grid: SdfGrid,
    grid_needs_updating: bool,
    smooth: bool,
    vm: Engine
}

fn scm_length(p: (f32, f32)) -> f32 {
    vec2(p.0, p.1).length()
}

fn grid_to_point(i: usize, j: usize) -> (f32, f32) {
    const HALF_CELL_SIZE: f32 = 0.5 / BASE as f32;
    let x = (i as f32 - 0.5 * NUM_X as f32) / BASE as f32 + HALF_CELL_SIZE;
    let y = (j as f32 - 0.5 * NUM_Y as f32) / BASE as f32 + HALF_CELL_SIZE;

    (x, y)
}

use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    sync::{Arc, Mutex},
};
use std::ops::DerefMut;
use steel::{
    gc::unsafe_erased_pointers::CustomReference,
};

// #[derive(Clone, Steel)]
// pub struct ScmGrid<'a>(Rc<RefCell<&'a mut SdfGrid>>);
// pub struct ScmGrid(Rc<RefCell<SdfGrid>>);
pub struct ScmGrid<'a>(&'a mut SdfGrid);

impl<'a> CustomReference for ScmGrid<'a> {}

steel::custom_reference!(ScmGrid<'a>);

fn scm_grid_set<'a>(grid: &mut ScmGrid, i: usize, j: usize, v: f32) {
    grid.0.grid[i][j] = v;
    // println!("Setting grid {i} {j}: {:?}", &grid.0.grid[..5]);
}

fn scm_point_x<'a>(p: (f32, f32)) -> f32 {
    p.0
}

fn scm_point_y<'a>(p: (f32, f32)) -> f32 {
    p.1
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        let mut vm = Engine::new();

        println!("Register scm_length");
        vm.register_fn("length", scm_length);
        vm.register_fn("grid->point", grid_to_point);
        vm.register_fn("grid-set", scm_grid_set);
        vm.register_fn("point-x", scm_point_x);
        vm.register_fn("point-y", scm_point_y);
        vm.register_external_value("grid", 0);

        vm.register_external_value("NUM_X", NUM_X);
        vm.register_external_value("NUM_Y", NUM_Y);

        println!("Run some code...");
        vm.run(include_str!("./sdf.scm"))
          .expect("Failed to initialise scheme");

        Self {
            size,
            start: Instant::now(),
            elapsed: Duration::ZERO,
            cursor: Vec2::ZERO,
            mouse_button_pressed: false,
            can_drag: None,
            drag_point: None,
            shader_constants: ShaderConstants::zeroed(),
            shape: Shape::Disk,
            onion: EnabledNumber::new(0.05, false),
            pad: EnabledNumber::new(0.05, false),
            repeat: Repetition::default(),
            params: Shape::iter().map(|shape| shape.default_params()).collect(),
            prev_params: Shape::Disk.default_params(),
            grid: SdfGrid::new(),
            grid_needs_updating: true,
            smooth: true,
            vm
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    self.drag_point = self.can_drag;
                    self.mouse_button_pressed = true;
                }
                ElementState::Released => {
                    self.drag_point = None;
                    self.mouse_button_pressed = false;
                }
            }
        }
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32);
        let num_points = self.shape.default_points().len();
        if let Some(i) = self.drag_point {
            self.params[self.shape as usize].ps[i] = rotate(
                from_pixels(self.cursor, self.size.into()),
                self.params[self.shape as usize].rot,
            )
            .into();
        } else if num_points > 0 {
            self.can_drag = self.params[self.shape as usize].ps[0..num_points as usize]
                .iter()
                .position(|p| {
                    (rotate((*p).into(), -self.params[self.shape as usize].rot)
                        - from_pixels(self.cursor, self.size.into()))
                    .length_squared()
                        < 0.0003
                });
        }
    }

    fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        self.params[self.shape as usize].rot += PI / 30.0
            * match delta {
                MouseScrollDelta::LineDelta(_, y) => y,
                MouseScrollDelta::PixelDelta(p) => {
                    (1.0 + p.y.abs() as f32).ln() * p.y.signum() as f32
                }
            };
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        self.params[self.shape as usize] = self.params();
        if self.params[self.shape as usize] != self.prev_params {
            self.grid_needs_updating = true;
        }
        let cursor = self.grid.clamp(from_pixels(self.cursor, self.size.into()));
        self.elapsed = self.start.elapsed();
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.elapsed.as_secs_f32(),
            cursor: cursor.into(),
            mouse_button_pressed: !(1
                << (self.mouse_button_pressed && self.drag_point.is_none()) as u32),
            points: self.params[self.shape as usize]
                .ps
                .map(|p| rotate(p.into(), -self.params[self.shape as usize].rot).into()),
            smooth: self.smooth.into(),
            derivative_at_cursor: self.grid.derivative(cursor).into(),
        };
        self.prev_params = self.params[self.shape as usize];
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, ctx: &Context, ui: &mut egui::Ui, event_proxy: &EventLoopProxy<UserEvent>) {
        ctx.set_cursor_icon(if self.drag_point.is_some() {
            CursorIcon::Grabbing
        } else if self.can_drag.is_some() {
            CursorIcon::Grab
        } else {
            CursorIcon::Default
        });
        for shape in Shape::iter() {
            ui.radio_value(&mut self.shape, shape, shape.to_string());
        }
        {
            let params = &mut self.params[self.shape as usize];
            let labels = self.shape.labels();
            if labels.len() > 0 {
                ui.separator();
            }
            for i in 0..labels.len() {
                let ranges = self.shape.dim_range();
                let range = ranges[i].clone();
                let speed = (range.end() - range.start()) * 0.02;
                ui.horizontal(|ui| {
                    ui.label(labels[i as usize]);
                    ui.add(
                        egui::DragValue::new(&mut params.dims[i as usize])
                            .clamp_range(range)
                            .speed(speed),
                    );
                });
            }
        }
        ui.separator();
        ui.checkbox(&mut self.smooth, "Smooth");
        self.pad.ui(ui, "Pad", 0.0..=0.2, 0.01);
        self.onion.ui(ui, "Onion", 0.0..=0.2, 0.01);
        ui.heading("Repetition");
        self.repeat.ui(ui);
        if self.grid_needs_updating {
            self.grid_needs_updating = false;
            let shape = self.shape;
            let params = self.params();

            self.vm.run(fs::read_to_string("./runner/src/shaders/sdf.scm").expect("failed to read sdf.scm"))
              .expect("Failed to initialise scheme");

            self.vm.with_mut_reference::<ScmGrid, ScmGrid>(&mut ScmGrid(&mut self.grid))
                .consume(move |engine, args| {
                    let grid = args[0].clone();
                    engine.update_value("grid", grid);
                    engine.call_function_by_name_with_args(r#"run-sdf"#, vec![]);
                });

            // self.grid.update(|p| sdf(p, shape, params));
            self.signal_new_buffers(event_proxy);
        }
    }

    fn buffers(&self) -> BufferData {
        BufferData {
            bind_group_buffers: vec![BindGroupBufferType::SSBO(SSBO {
                data: bytemuck::cast_slice(&self.grid.grid),
                read_only: true,
            })],
            ..Default::default()
        }
    }
}


impl Controller {
    fn params(&self) -> Params {
        Params {
            pad: self.pad.into(),
            onion: self.onion.into(),
            repeat: self.repeat.into(),
            ..self.params[self.shape as usize]
        }
    }

    fn update_grid(&mut self) {
        // let grid_cell = RefCell::new(self.grid);
        // let grid = ScmGrid(Rc::new(grid_cell));
        // self.vm.call_function_by_name_with_args(r#"run-sdf"#, vec![ &mut self.grid ]).expect("Failed to run scheme function");
        self.vm.with_mut_reference::<ScmGrid, ScmGrid>(&mut ScmGrid(&mut self.grid))
            .consume(move |engine, args| {
                let grid = args[0].clone();
                engine.update_value("grid", grid);
                engine.call_function_by_name_with_args(r#"run-sdf"#, vec![]);
            })

        // let value = self.sdf(vec2(x, y));
    }

//     fn update_grid(&mut self) {
//         const HALF_CELL_SIZE: f32 = 0.5 / BASE as f32;
//         for i in 0..NUM_X {
//             for j in 0..NUM_Y {
//                 let x = (i as f32 - 0.5 * NUM_X as f32) / BASE as f32 + HALF_CELL_SIZE;
//                 let y = (j as f32 - 0.5 * NUM_Y as f32) / BASE as f32 + HALF_CELL_SIZE;
//                 // let value = self.sdf(vec2(x, y));
// 
//                 self.vm.register_external_value("p", (x, y))
//                   .expect("Failed to create p");
//                 let value = self.vm.run(r#"(sdf p)"#).expect("Failed to run scheme function");
// 
//                 match value[0] {
//                     SteelVal::IntV(l) => self.grid.set(i, j, l as f32),
//                     SteelVal::NumV(f) => self.grid.set(i, j, f as f32),
//                     _ => {
//                         // !("Scheme function returned funny number");
//                         self.grid.set(i, j, 0.0);
//                     }
//                 }
//             }
//         }
//     }
// 
    fn signal_new_buffers(&self, event_proxy: &EventLoopProxy<UserEvent>) {
        if event_proxy.send_event(UserEvent::NewBuffersReady).is_err() {
            panic!("Event loop dead");
        }
    }
}

fn sdf(mut p: Vec2, shape: Shape, params: Params) -> f32 {
    use Shape::*;
    let dim: Vec2 = params.dims.into();
    let radius = dim.x;
    let p0: Vec2 = params.ps[0].into();
    let p1: Vec2 = params.ps[1].into();
    let p2: Vec2 = params.ps[2].into();
    let p3: Vec2 = params.ps[3].into();
    let p4: Vec2 = params.ps[4].into();
    p = p.rotate(Vec2::from_angle(params.rot));

    let f = |p| match shape {
        Disk => sdf::disk(p, radius),
        Rectangle => sdf::rectangle(p, dim),
        EquilateralTriangle => sdf::equilateral_triangle(p, radius),
        IsoscelesTriangle => sdf::isosceles_triangle(p, dim),
        Triangle => sdf::triangle(p, p0, p1, p2),
        Capsule => sdf::capsule(p, p0, p1, radius),
        Torus => sdf::torus(p, dim),
        Line => sdf::line(p, Vec2::Y),
        Plane => sdf::plane(p, Vec2::Y),
        LineSegment => sdf::line_segment(p, p0, p1),
        Ray => sdf::ray(p - p0, Vec2::X),
        Hexagon => sdf::hexagon(p, radius),
        Pentagon => sdf::pentagon(p, radius),
        Polygon => sdf::polygon(p, [p0, p1, p2, p3, p4]),
        Cross => sdf::cross(p, dim),
    };

    let mut d = {
        let RepetitionData {
            current,
            dim,
            n1,
            n2,
        } = params.repeat;
        use sdf::ops::{fast_repeat, repeat};
        use RepetitionValue::*;
        match current {
            None => f(p),
            Unlimited => repeat::Repeat::<2>::repeat_xy(p, dim, f),
            Limited => repeat::RepeatLimited::<2>::repeat_xy(p, dim, n1, n2, f),
            Rectangular => f(fast_repeat::repeat_rectangular(p, dim.x, n1)),
            Angular => repeat::repeat_angular(p, dim.x, n1.x, f),
            Mirrored => f(fast_repeat::repeat_mirrored(p, dim)),
        }
    };

    if params.pad.has_value() {
        d = sdf::ops::pad(d, params.pad.value)
    }

    if params.onion.has_value() {
        d = sdf::ops::onion(d, params.onion.value)
    }

    d
}

fn rotate(p: Vec2, angle: f32) -> Vec2 {
    p.rotate(Vec2::from_angle(angle))
}
