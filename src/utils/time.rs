use serenity::all::Timestamp;



pub fn get_relative_timestamp(timestamp: Timestamp) -> String {
    return format!("<t:{}:R>", timestamp.timestamp());
}