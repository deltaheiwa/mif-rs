
/// Calculates the percentage of a part in a total, and rounds down the result to the second decimal. Doesn't work for very large numbers
pub fn calculate_percentage(part: i32, total: i32) -> f64 {
    ((part as f64 / total as f64) * 10000.0).round() / 100.0
}