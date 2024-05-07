use dfutils::primitives::*;
use dfutils::primitives_enum::*;

/// The ui for the leaf nodes of [SdfBuilderTree]
pub trait ShapeUi {
    fn ui(self, ui: &mut egui::Ui) -> Self;
}

impl ShapeUi for Disk {
    fn ui(mut self, ui: &mut egui::Ui) -> Self {
        ui.label("Radius");
        ui.add(
            egui::DragValue::new(&mut self.radius)
                .clamp_range(0.0..=f64::INFINITY)
                .speed(0.01),
        );
        self
    }
}

impl ShapeUi for Torus {
    fn ui(mut self, ui: &mut egui::Ui) -> Self {
        ui.label("Major Radius");
        ui.add(
            egui::DragValue::new(&mut self.major_radius)
                .clamp_range(0.0..=f64::INFINITY)
                .speed(0.01),
        );
        ui.end_row();
        ui.label("Minor Radius");
        ui.add(
            egui::DragValue::new(&mut self.minor_radius)
                .clamp_range(0.0..=self.major_radius)
                .speed(0.01),
        );
        self
    }
}

impl ShapeUi for Rectangle {
    fn ui(mut self, ui: &mut egui::Ui) -> Self {
        ui.label("Width");
        ui.add(
            egui::DragValue::new(&mut self.width)
                .clamp_range(0.0..=f64::INFINITY)
                .speed(0.01),
        );
        ui.end_row();
        ui.label("Height");
        ui.add(
            egui::DragValue::new(&mut self.height)
                .clamp_range(0.0..=f64::INFINITY)
                .speed(0.01),
        );
        self
    }
}

impl ShapeUi for Cross {
    fn ui(mut self, ui: &mut egui::Ui) -> Self {
        ui.label("Length");
        ui.add(
            egui::DragValue::new(&mut self.length)
                .clamp_range(0.0..=f64::INFINITY)
                .speed(0.01),
        );
        ui.end_row();
        ui.label("Thickness");
        ui.add(
            egui::DragValue::new(&mut self.thickness)
                .clamp_range(0.0..=self.length)
                .speed(0.01),
        );
        self
    }
}

impl ShapeUi for LineSegment {
    fn ui(self, _ui: &mut egui::Ui) -> Self {
        self
    }
}

impl ShapeUi for Plane {
    fn ui(self, _ui: &mut egui::Ui) -> Self {
        self
    }
}

impl ShapeUi for Ray {
    fn ui(self, _ui: &mut egui::Ui) -> Self {
        self
    }
}

impl ShapeUi for Shape {
    fn ui(self, ui: &mut egui::Ui) -> Self {
        match self {
            Shape::Disk(shape) => shape.ui(ui).into(),
            Shape::Torus(shape) => shape.ui(ui).into(),
            Shape::Rectangle(shape) => shape.ui(ui).into(),
            Shape::Cross(shape) => shape.ui(ui).into(),
            Shape::LineSegment(shape) => shape.ui(ui).into(),
            Shape::Plane(shape) => shape.ui(ui).into(),
            Shape::Ray(shape) => shape.ui(ui).into(),
        }
    }
}
