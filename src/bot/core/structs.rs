
#[derive(Debug)]
pub struct Data {} 
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type PartialContext<'a> = poise::PartialContext<'a, Data, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;