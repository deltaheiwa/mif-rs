use tracing_subscriber::fmt::time::OffsetTime;
use time::{format_description::parse, UtcOffset};

pub fn install_subscriber() {
    let timer_format = parse(
        "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second]"
    ).unwrap();

    let time_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);

    let timer = OffsetTime::new(time_offset, timer_format);

    tracing_subscriber::fmt()
        .with_level(true)
        .with_thread_names(true)
        .with_target(false)
        .with_timer(timer)
        .compact()
        .init();
}
