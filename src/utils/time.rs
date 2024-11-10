pub fn get_relative_timestamp(timestamp: &i64) -> String {
    return format!("<t:{}:R>", timestamp);
}