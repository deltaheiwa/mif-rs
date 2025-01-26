mod bot;
mod db;
mod utils;

use std::env;
use std::sync::Arc;
use dotenvy::dotenv;

// Set t! to be available in the global scope
#[macro_use]
extern crate rust_i18n;
#[allow(unused_imports)]
pub use rust_i18n::t;
use crate::bot::core::structs::MetricsManager;

i18n!("locale", fallback = "en");

#[tokio::main]
async fn main() {
    dotenv().ok();
    utils::logger::install_subscriber();

    let pid = std::process::id();

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find 'DISCORD_TOKEN' in .env file");

    let mut bot = bot::Bot::new(token).await;

    let metrics_manager = MetricsManager::new();
    tokio::spawn(bot::server::run_metrics_manager(Arc::new(metrics_manager), pid));
    bot.start().await;
}
