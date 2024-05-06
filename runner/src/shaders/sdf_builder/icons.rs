use dfutils::{grid::Grid, primitives::Disk, primitives_enum::Shape};
use egui::{Color32, ColorImage, TextureHandle};
use glam::{vec2, vec3};
use shared::sdf_interpreter::{Instruction, Operator, SdfInstructions, Transform};
use strum::IntoEnumIterator;

pub struct IconImages {
    pub shapes: Vec<ColorImage>,
    pub operators: Vec<ColorImage>,
}

pub struct TextureHandles {
    pub shapes: Vec<TextureHandle>,
    pub operators: Vec<TextureHandle>,
}

impl TextureHandles {
    pub fn empty() -> Self {
        TextureHandles {
            shapes: vec![],
            operators: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }
}

pub fn generate_icons() -> IconImages {
    const N: usize = 64;
    IconImages {
        shapes: Shape::iter()
            .map(|shape| ColorImage {
                size: [N, N],
                pixels: Grid::from_sdf(N, N, &shape)
                    .buffer
                    .into_iter()
                    .map(color_from_distance)
                    .collect(),
            })
            .collect(),
        operators: Operator::iter()
            .map(|op| {
                let instructions = get_instructions(op);
                let sdf = SdfInstructions::new(&instructions);
                ColorImage {
                    size: [N, N],
                    pixels: Grid::from_sdf(N, N, &sdf)
                        .buffer
                        .into_iter()
                        .map(color_from_distance)
                        .collect(),
                }
            })
            .collect(),
    }
}

fn get_instructions(op: Operator) -> [Instruction; 3] {
    let disk = Shape::Disk(Disk::new(0.25));
    [
        Instruction::Shape(
            disk,
            Transform {
                position: vec2(0.1, 0.0),
            },
        ),
        Instruction::Shape(
            disk,
            Transform {
                position: vec2(-0.1, 0.0),
            },
        ),
        Instruction::Operator(op),
    ]
}

fn color_from_distance(d: f32) -> Color32 {
    let col = 255.0
        * (1.0 - (-6.0 * d.abs()).exp())
        * (if d < 0.0 {
            vec3(0.65, 0.85, 1.0)
        } else {
            vec3(0.9, 0.6, 0.3)
        });
    Color32::from_rgb(col.x as u8, col.y as u8, col.z as u8)
}
