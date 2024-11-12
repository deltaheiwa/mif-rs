pub fn get_relative_timestamp(timestamp: &i64) -> String {
    format!("<t:{}:R>", timestamp)
}

pub fn get_long_date(timestamp: &i64) -> String {
    format!("<t:{}:D>", timestamp)
}