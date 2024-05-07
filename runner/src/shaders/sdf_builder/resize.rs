use dfutils::primitives::*;
use dfutils::primitives_enum::Shape;
use glam::*;

pub trait Resize {
    fn resize(self, initial: Vec2, current: Vec2, derivative: Vec2) -> Self;
}

impl Resize for Disk {
    fn resize(self, initial: Vec2, current: Vec2, derivative: Vec2) -> Self {
        let s = (current - initial) * derivative;
        Disk::new((self.radius + s.x + s.y).max(0.0))
    }
}

impl Resize for Torus {
    fn resize(mut self, initial: Vec2, current: Vec2, derivative: Vec2) -> Self {
        let s = (current - initial) * derivative;
        if initial.length() > self.major_radius {
            self.major_radius = (self.major_radius + s.x + s.y).max(0.0);
        } else {
            self.minor_radius = (self.minor_radius + s.x + s.y).max(0.0);
        }
        self.minor_radius = self.minor_radius.min(self.major_radius);
        self
    }
}

impl Resize for Rectangle {
    fn resize(self, initial: Vec2, current: Vec2, derivative: Vec2) -> Self {
        let scale = {
            let Vec2 { x, y } = derivative;
            vec2(
                if x.abs() > 0.05 { x.signum() } else { x },
                if y.abs() > 0.05 { y.signum() } else { y },
            ) * 2.0
        };
        let s = (current - initial) * scale;
        Rectangle::new((self.width + s.x).max(0.0), (self.height + s.y).max(0.0))
    }
}

impl Resize for Cross {
    fn resize(mut self, initial: Vec2, current: Vec2, derivative: Vec2) -> Self {
        let s = (current - initial) * derivative;
        if initial.abs().max_element() < self.length - 0.01 {
            self.thickness = (self.thickness + s.x + s.y).max(0.0);
        } else if derivative.x.abs() > 0.05 && derivative.y.abs() > 0.05 {
            let mut s = (current - initial) * derivative.signum();
            if initial.y.abs() > initial.x.abs() {
                s = s.yx();
            }
            self.length = (self.length + s.x).max(0.0);
            self.thickness = (self.thickness + s.y).max(0.0);
        } else {
            self.length = (self.length + s.x + s.y).max(0.0);
        }
        self.thickness = self.thickness.min(self.length);
        self
    }
}

impl Resize for LineSegment {
    fn resize(mut self, initial: Vec2, current: Vec2, _derivative: Vec2) -> Self {
        if initial.distance(self.a) < 0.01 {
            self.a += current - initial;
        } else {
            self.b += current - initial;
        }
        self
    }
}

impl Resize for Plane {
    fn resize(self, _initial: Vec2, _current: Vec2, _derivative: Vec2) -> Self {
        self
    }
}

impl Resize for Ray {
    fn resize(mut self, _initial: Vec2, current: Vec2, _derivative: Vec2) -> Self {
        self.direction = current.normalize();
        self
    }
}

impl Resize for Shape {
    fn resize(self, initial: Vec2, current: Vec2, derivative: Vec2) -> Self {
        match self {
            Shape::Disk(shape) => shape.resize(initial, current, derivative).into(),
            Shape::Torus(shape) => shape.resize(initial, current, derivative).into(),
            Shape::Rectangle(shape) => shape.resize(initial, current, derivative).into(),
            Shape::Cross(shape) => shape.resize(initial, current, derivative).into(),
            Shape::LineSegment(shape) => shape.resize(initial, current, derivative).into(),
            Shape::Plane(shape) => shape.resize(initial, current, derivative).into(),
            Shape::Ray(shape) => shape.resize(initial, current, derivative).into(),
        }
    }
}
