mod bot;
mod db;
mod utils;

use std::env;
use dotenvy::dotenv;


#[tokio::main]
async fn main() {
    dotenv().ok();
    utils::logger::install_subscriber();

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find 'DISCORD_TOKEN' in .env file");

    let mut bot = bot::Bot::new(token).await;

    db::prefixes::create_db().await;

    bot.start().await;
}
