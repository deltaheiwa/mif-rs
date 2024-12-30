pub mod wolvesville;

use reqwest::get;
use image::{load_from_memory, DynamicImage, Pixel};


pub async fn get_image_by_url(url: &str) -> anyhow::Result<DynamicImage> {
    let bytes = get(url).await?.bytes().await?;
    Ok(load_from_memory(&bytes)?)
}

pub fn overlay_transparent_image(background: &mut DynamicImage, overlay: &DynamicImage, offset_x: u32, offset_y: u32) {
    let mut background_as_rgba = background.to_rgba8();
    let overlay = overlay.to_rgba8();

    for x in 0..overlay.width() {
        for y in 0..overlay.height() {
            let pixel = background_as_rgba.get_pixel_mut(x + offset_x, y + offset_y);
            pixel.blend(&overlay.get_pixel(x, y));
        }
    }

    *background = DynamicImage::ImageRgba8(background_as_rgba);
}
