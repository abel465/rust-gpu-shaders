#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub mod grid;
pub mod gridref;
pub mod primitives;
pub mod primitives_enum;
pub mod sdf;

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use glam::vec2;
    use grid::Grid;
    use primitives::*;
    use sdf::Sdf;

    #[test]
    fn primitives() {
        let disk = Disk::new(0.1);
        let d = disk.signed_distance(vec2(0.0, 0.4));
        assert_approx_eq!(d, 0.3);
        let torus = Torus::new(0.2, 0.1);
        let d = torus.signed_distance(vec2(0.4, 0.0));
        assert_approx_eq!(d, 0.1);
    }

    #[test]
    fn grid() {
        const ROWS: usize = 32;
        const COLS: usize = 32;
        const E: f32 = 1.0 / (COLS as f32) + f32::EPSILON;

        let disk = Disk::new(0.1);
        let mut grid = Grid::from_sdf(ROWS, COLS, &disk);
        let d = grid.signed_distance(vec2(0.0, 0.4));
        assert_approx_eq!(d, 0.3, E);
        let d = grid.as_ref().signed_distance(vec2(0.0, 0.4));
        assert_approx_eq!(d, 0.3, E);

        let torus = Torus::new(0.2, 0.1);
        grid.update(&torus);
        let d = grid.signed_distance(vec2(0.4, 0.0));
        assert_approx_eq!(d, 0.1, E);
        let d = grid.as_ref().signed_distance(vec2(0.4, 0.0));
        assert_approx_eq!(d, 0.1, E);
    }
}
