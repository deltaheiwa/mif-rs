use image::{open, DynamicImage, ImageBuffer, Rgba};
use crate::utils;
use crate::utils::apicallers::wolvesville::models::Avatar;

fn add_level_render() {

}

pub async fn render_wolvesville_avatar(avatar: Avatar) -> anyhow::Result<DynamicImage> {
    // Import avatar background to maintain aspect ratio
    let overlay_background = open("res/images/wov_small_night_avatar.png")?;
    // Lay avatar background above solid dark blue color
    let color = Rgba([78, 96, 120, 255]);
    let mut solid_background = DynamicImage::ImageRgba8(ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(
        overlay_background.width(),
        overlay_background.height(),
        color
    ));

    utils::image::overlay_transparent_image(&mut solid_background, &overlay_background, 0, 0);

    let avatar_image = utils::image::get_image_by_url(&avatar.url).await?;
    // Position avatar at the bottom center of the background
    let x = (solid_background.width() - avatar.width as u32) / 2;
    let y = solid_background.height() - avatar.height as u32;
    utils::image::overlay_transparent_image(&mut solid_background, &avatar_image, x, y);


    Ok(solid_background)
}