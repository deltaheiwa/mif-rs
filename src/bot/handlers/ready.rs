use poise::serenity_prelude as serenity;
use logfather::info;
use crate::db;

pub async fn on_ready(_ctx: serenity::Context, ready: serenity::Ready) {
    db::create_db().await;
    info!("Connected to {}", ready.user.name);
}