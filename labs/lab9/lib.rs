/// A trait for calculating the area of a shape.
pub trait Area {
    fn area(&self) -> f64;

}

/// A trait for calculating the perimeter of a shape.
pub trait Perimeter {
    fn perimeter(&self) -> f64;

}

/// A rectangle type.
pub struct Rectangle {
    pub width: f64,
    pub height: f64,

}

/// A circle type.
pub struct Circle {
    pub radius: f64,

}

/// Implement both traits for both shapes.
impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Perimeter for Rectangle {
    fn perimeter(&self) -> f64 {
    (self.width + self.height) * 2.0
    }
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Perimeter for Circle {
    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
}

/// A generic function that describes any shape implementing both Area and Perimeter.
/// Returns a formatted string:
/// "Area: X, Perimeter: Y"
pub fn describe_shape<T: Area + Perimeter>(shape: &T) -> String {
    format!(
        "Area: {}, Perimeter: {}", shape.area(), shape.perimeter() 
    )
}