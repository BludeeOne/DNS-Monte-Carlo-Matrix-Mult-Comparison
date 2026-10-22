/// Safely divides two numbers.
/// Returns:
/// - Ok(result) if the division succeeds
/// - Err("division by zero") if `b` is zero
///
/// # Example
/// ```
/// safe_divide(10.0, 2.0); // -> Ok(5.0)
/// safe_divide(5.0, 0.0);  // -> Err("division by zero".to_string())
/// ```
pub fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("division by zero".to_string())
    }
    else {
        Ok(a / b)
    }

}