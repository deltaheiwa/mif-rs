mod bot;
mod db;
mod utils;

use std::env;
use dotenvy::dotenv;

// Set t! to be available in the global scope
#[macro_use]
extern crate rust_i18n;
pub use rust_i18n::t;
i18n!("locale", fallback = "en");

#[tokio::main]
async fn main() {
    dotenv().ok();
    utils::logger::install_subscriber();

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find 'DISCORD_TOKEN' in .env file");

    let mut bot = bot::Bot::new(token).await;

    bot.start().await;
}
