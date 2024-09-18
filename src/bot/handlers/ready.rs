use poise::serenity_prelude as serenity;
use tracing::info;
use crate::db;

pub async fn on_ready(_ctx: serenity::Context, ready: serenity::Ready) {
    db::prefixes::create_db().await;
    info!("Connected to {}", ready.user.name);
}