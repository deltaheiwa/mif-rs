use std::panic::{self, PanicHookInfo};
use chrono::Local;
use logfather::{error, Level, Logger};

pub fn install_subscriber() {
    let _ = Logger::new()
        .file(true)
        .path("logs/main.log")
        .timestamp_format("%Y-%m-%d %H:%M:%S")
        .log_format("{timestamp} [{level}] {module_path} | {message}")
        .file_ignore(Level::Debug);
}

pub fn set_up_panic_hook() {
    panic::set_hook(Box::new(|panic_info: &PanicHookInfo| {
        let payload = panic_info.payload().downcast_ref::<&str>()
            .map_or("<unknown>".to_string(), |s| s.to_string());
        let location = panic_info.location().map_or("<unknown>".to_string(), |loc| {
            format!("at {}:{}:{}", loc.file(), loc.line(), loc.column())
        });
        
        let backtrace = std::backtrace::Backtrace::force_capture();
        
        error!("--- PANIC DETECTED ---\n \
        Timestamp: {} \
        \nPayload: \"{}\" \
        \nLocation: {} \
        \nBacktrace:\n{} \
        \n------------------------"
        , Local::now().format("%Y-%m-%d %H:%M:%S"), payload, location, backtrace);

        eprintln!("\nCRITICAL ERROR: A panic occurred. Details logged to logs/main.log.");
        eprintln!("Panic payload: \"{}\" {}", payload, location);
    }));
}