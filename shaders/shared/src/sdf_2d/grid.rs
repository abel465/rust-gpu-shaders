use spirv_std::glam::{vec2, Vec2};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

const SMOOTH_PADDING: usize = 2;
pub const BASE: usize = 64 - SMOOTH_PADDING;
const AR: usize = 3;
pub const NUM_Y: usize = BASE + SMOOTH_PADDING;
pub const NUM_X: usize = BASE * AR + SMOOTH_PADDING;
const HALF_CELL_SIZE: f32 = 0.5 / BASE as f32;

pub struct SdfGrid {
    pub grid: [[f32; NUM_Y]; NUM_X],
}

#[cfg(not(target_arch = "spirv"))]
impl SdfGrid {
    pub fn new() -> Self {
        Self {
            grid: [[0.0; NUM_Y]; NUM_X],
        }
    }

    pub fn update<F>(&mut self, sdf: F)
    where
        F: Fn(Vec2) -> f32,
    {
        for i in 0..NUM_X {
            for j in 0..NUM_Y {
                self.grid[i][j] = sdf(vec2(
                    i as f32 - 0.5 * NUM_X as f32,
                    j as f32 - 0.5 * NUM_Y as f32,
                ) / BASE as f32
                    + HALF_CELL_SIZE);
            }
        }
    }
}

impl SdfGrid {
    fn indices(&self, p: Vec2) -> Vec2 {
        vec2(
            p.x * BASE as f32 + 0.5 * NUM_X as f32,
            (p.y + 0.5) * BASE as f32 + 0.5 * SMOOTH_PADDING as f32,
        )
    }

    pub fn clamp(&self, p: Vec2) -> Vec2 {
        let v = vec2(AR as f32, 1.0) * 0.5;
        p.clamp(-v, v - 0.001)
    }

    pub fn dist(&self, p: Vec2) -> f32 {
        let Vec2 { x, y } = self.indices(p);
        self.grid[x as usize][y as usize]
    }

    pub fn dist_smooth(&self, p: Vec2) -> f32 {
        let Vec2 { x, y } = self.indices(p);

        let left_index = (x - 0.5) as usize;
        let middle_index_x = x as usize;
        let right_index = (x + 0.5) as usize;
        let left_scale = (0.5 - x.fract()).max(0.0);
        let middle_scale_x = 0.5 + ((x - 0.5).fract() - 0.5).abs();
        let right_scale = (x.fract() - 0.5).max(0.0);

        let bot_index = (y - 0.5) as usize;
        let middle_index_y = y as usize;
        let top_index = (y + 0.5) as usize;
        let bot_scale = (0.5 - y.fract()).max(0.0);
        let middle_scale_y = 0.5 + ((y - 0.5).fract() - 0.5).abs();
        let top_scale = (y.fract() - 0.5).max(0.0);

        left_scale
            * (bot_scale * self.grid[left_index][bot_index]
                + middle_scale_y * self.grid[left_index][middle_index_y]
                + top_scale * self.grid[left_index][top_index])
            + middle_scale_x
                * (bot_scale * self.grid[middle_index_x][bot_index]
                    + middle_scale_y * self.grid[middle_index_x][middle_index_y]
                    + top_scale * self.grid[middle_index_x][top_index])
            + right_scale
                * (bot_scale * self.grid[right_index][bot_index]
                    + middle_scale_y * self.grid[right_index][middle_index_y]
                    + top_scale * self.grid[right_index][top_index])
    }

    pub fn derivative(&self, p: Vec2) -> Vec2 {
        const E: f32 = HALF_CELL_SIZE;
        vec2(
            self.dist_smooth(p - Vec2::X * E) - self.dist_smooth(p + Vec2::X * E),
            self.dist_smooth(p - Vec2::Y * E) - self.dist_smooth(p + Vec2::Y * E),
        ) / (E * 2.0)
    }
}
