
/// Calculates the percentage of a part in a total
pub fn calculate_percentage(part: i32, total: i32) -> f64 { if total == 0 { 0.00 } else { part as f64 * 100.0 / total as f64 } }