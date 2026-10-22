/// Represents geometric shapes.
pub enum Shape {
    Circle(f64),        // radius
    Rectangle(f64, f64),// width, height
    Square(f64),        // side length
}

/// Computes the area of a given shape.
///
/// # Example
/// ```
/// area(Shape::Circle(2.0)); // -> 12.566370...
/// ```
pub fn area(shape: Shape) -> f64 {
    match shape {
        Shape::Circle(radius) => std::f64::consts::PI * radius * radius,
        Shape::Rectangle(height, width) => height * width,
        Shape::Square(side) => side * side,
    }
 }
