
/// Calculates the percentage of a part in a total
pub fn calculate_percentage(part: i32, total: i32) -> f64 { if total == 0 { 0.00 } else { part as f64 * 100.0 / total as f64 } }

/// Determines the level rank of a player based on their level. Used for rendering the level rank on the avatar
pub fn determine_level_rank(level: i32) -> i8 {
    if level < 0 { 0i8 }
    else if level < 420 { (level / 10 + 1) as i8 }
    else if level < 1000 { if level < 500 { 43i8 } else {(43 + (level - 400) / 100) as i8 } }
    else { 49i8 }
}
