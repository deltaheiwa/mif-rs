use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{open, DynamicImage, ImageBuffer, Rgba};
use imageproc::drawing::{draw_text_mut};
use crate::utils;
use crate::utils::apicallers::wolvesville::models::Avatar;

fn add_level_rank(image: &mut DynamicImage, level: i32) {
    let mut rank_image = open(
        format!(
            "res/images/ranks/rank_{}.png",
            utils::math::determine_level_rank(level)
        )
    ).unwrap();


    let font = FontRef::try_from_slice(include_bytes!("../../../res/fonts/OpenSans-Bold.ttf")).expect("Error loading font");
    let scale = PxScale::from(70.0);
    let text_color = Rgba([255, 255, 255, 255]);

    let mut width = 0.0;
    let scaled_font = font.as_scaled(scale);
    let height = scaled_font.ascent() - scaled_font.descent();

    for c in level.to_string().chars() {
        let glyph_id = font.glyph_id(c);
        // Get the glyph metrics (horizontal)
        let h_metrics = scaled_font.h_advance(glyph_id);

        // Update the text width
        width += h_metrics;
    }

    let x = ((rank_image.width() as f32 - width) / 2.0).round() as i32;  // rightmost pixel of bg - width of the whole text = leftmost pixel for the text
    let y = ((rank_image.height() as f32 - height) / 2.0).round() as i32;  // same as above but for height

    draw_text_mut(&mut rank_image, text_color, x, y, scale, &font, &level.to_string());

    let rank_image_resized = DynamicImage::ImageRgba8(rank_image.as_rgba8().unwrap().clone()).resize(
        (rank_image.width() as f64 * 0.3) as u32,
        (rank_image.height() as f64 * 0.3) as u32,
        image::imageops::FilterType::Gaussian
    );

    utils::image::overlay_transparent_image(image, &rank_image_resized, 95, 15);
}

pub async fn render_wolvesville_avatar(avatar: Avatar, level: Option<i32>) -> anyhow::Result<DynamicImage> {
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

    let mut avatar_image = utils::image::get_image_by_url(&avatar.url).await?;

    // Crop avatar if it's too big
    if avatar_image.width() > solid_background.width() {
        // Crop the sides
        let x = (avatar_image.width() - solid_background.width()) / 2;
        avatar_image = avatar_image.crop_imm(x, 0, solid_background.width(), solid_background.height());
    }
    if avatar_image.height() > solid_background.height() {
        // Crop only the top
        avatar_image = avatar_image.crop_imm(0, 0, solid_background.width(), solid_background.height());
    }

    // Position avatar at the bottom center of the background
    let x = (solid_background.width() - avatar_image.width()) / 2;
    let y = solid_background.height() - avatar_image.height();
    utils::image::overlay_transparent_image(&mut solid_background, &avatar_image, x, y);

    // Add level render if level is provided
    if let Some(level) = level {
        add_level_rank(&mut solid_background, level);
    }

    Ok(solid_background)
}