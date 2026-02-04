use spirv_std::glam::{vec2, vec3, Vec2, Vec3};

pub fn integrate(f: &dyn Fn(f32) -> f32, a: f32, b: f32, p: u32) -> f32 {
    if p == 0 || (a - b).abs() == 0.0 {
        return 0.0;
    }

    let delta = (b - a) / p as f32;

    let mut integral = 0.0;
    let mut pos = a;
    for _ in 0..p {
        integral += f(pos);
        pos += delta;
    }

    integral * delta
}

pub fn integrate_2d(f: &dyn Fn(Vec2) -> f32, a: Vec2, b: Vec2, p: [u32; 2]) -> f32 {
    if p[0] == 0 || p[1] == 0 || (a - b).abs().min_element() == 0.0 {
        return 0.0;
    }

    let delta = (b - a) / vec2(p[0] as f32, p[1] as f32);

    let mut integral = 0.0;
    let mut pos = a;
    for _ in 0..p[0] {
        for _ in 0..p[1] {
            integral += f(pos);
            pos.y += delta.y;
        }
        pos.y = a.y;
        pos.x += delta.x;
    }

    integral * delta.x * delta.y
}

pub fn integrate_3d(f: &dyn Fn(Vec3) -> f32, a: Vec3, b: Vec3, p: [u32; 3]) -> f32 {
    if p[0] == 0 || p[1] == 0 || p[2] == 0 || (a - b).abs().min_element() == 0.0 {
        return 0.0;
    }

    let delta = (b - a) / vec3(p[0] as f32, p[1] as f32, p[2] as f32);

    let mut integral = 0.0;
    let mut pos = a;
    for _ in 0..p[0] {
        for _ in 0..p[1] {
            for _ in 0..p[2] {
                integral += f(pos);
                pos.z += delta.z;
            }
            pos.z = a.z;
            pos.y += delta.y;
        }
        pos.y = a.y;
        pos.x += delta.x;
    }

    integral * delta.x * delta.y * delta.z
}
