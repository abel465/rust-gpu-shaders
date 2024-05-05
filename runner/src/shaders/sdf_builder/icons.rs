use dfutils::{grid::Grid, primitives_enum::Shape};
use egui::{Color32, ColorImage};
use glam::vec3;
use strum::IntoEnumIterator;

pub fn generate_icons() -> impl Iterator<Item = ColorImage> {
    const N: usize = 64;
    Shape::iter().map(|shape| ColorImage {
        size: [N, N],
        pixels: Grid::from_sdf(N, N, &shape)
            .buffer
            .into_iter()
            .map(color_from_distance)
            .collect(),
    })
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
