use spirv_std::glam::{vec2, Vec2, Vec3, Vec3Swizzles};

pub fn torus_x(p: Vec3, r: Vec2) -> f32 {
    vec2(p.yz().length() - r.x, p.x).length() - r.y
}

pub fn torus_y(p: Vec3, r: Vec2) -> f32 {
    vec2(p.xz().length() - r.x, p.y).length() - r.y
}

pub fn torus_z(p: Vec3, r: Vec2) -> f32 {
    vec2(p.xy().length() - r.x, p.z).length() - r.y
}

pub fn disk_x(p: Vec3, r: f32) -> f32 {
    let v = vec2(p.yz().length() - r, p.x.abs());
    v.max(Vec2::ZERO).length() + v.max_element().min(0.0)
}

pub fn disk_y(p: Vec3, r: f32) -> f32 {
    let v = vec2(p.xz().length() - r, p.y.abs());
    v.max(Vec2::ZERO).length() + v.max_element().min(0.0)
}

pub fn disk_z(p: Vec3, r: f32) -> f32 {
    let v = vec2(p.xy().length() - r, p.z.abs());
    v.max(Vec2::ZERO).length() + v.max_element().min(0.0)
}
