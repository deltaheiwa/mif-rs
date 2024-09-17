use std::env;
use dotenvy::dotenv;

mod bot;


#[tokio::main]
async fn main() {
    dotenv().ok();
    bot::utils::logger::install_subscriber();

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find 'DISCORD_TOKEN' in .env file");

    let mut bot = bot::core::build_client(token).await.expect("Failure to build the client");

    bot::db::prefixes::create_db().await;

    let _ = bot.start().await;
}
