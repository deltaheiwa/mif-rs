use reqwest::get;
use image::{load_from_memory, DynamicImage, ImageResult};


pub async fn get_image(url: &str) -> Result<ImageResult<DynamicImage>, reqwest::Error> {
    let response = get(url).await?;
    let bytes = response.bytes().await?;
    Ok(load_from_memory(&bytes))
}