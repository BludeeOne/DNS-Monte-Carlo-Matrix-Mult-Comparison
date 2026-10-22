/// Safely divides two numbers.
/// Returns `Ok(result)` if the division succeeds,
/// or `Err("division by zero")` if `b` is zero.
pub fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("division by zero".to_string())
    }
    else {
        Ok(a / b)
    }
}

/// Calls `safe_divide` and returns a readable message.
///
/// # Example
/// ```
/// handle_division(10.0, 2.0); // -> "Result: 5"
/// handle_division(5.0, 0.0);  // -> "Error: division by zero"
/// ```
pub fn handle_division(a: f64, b: f64) -> String {
    match safe_divide(a, b) {
        Ok(result) => format!("Result: {}", result),
        Err(message) => format!("Error: {}" , message),
    }
}