use logfather::{Level, Logger};

pub fn install_subscriber() {

    let _ = Logger::new()
        .file(true)
        .path("logs/main.log")
        .timestamp_format("%Y-%m-%d %H:%M:%S")
        .log_format("{timestamp} [{level}]: {message}")
        .file_ignore(Level::Debug);
}
