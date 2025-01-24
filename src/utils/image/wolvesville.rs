use std::collections::HashMap;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{open, DynamicImage, ImageBuffer, Rgba};
use imageproc::drawing::{draw_text_mut, Canvas};
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

pub async fn render_wolvesville_avatar(avatar: Avatar, level: Option<i32>) -> anyhow::Result<(String, DynamicImage)> {
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

    let mut avatar_image = utils::image::get_image_by_url(avatar.url.as_str()).await?;

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

    let mask = utils::image::create_rounded_rectangle_mask(
        solid_background.width(),
        solid_background.height(),
        20.0
    );

    solid_background = utils::image::apply_mask(&solid_background, &mask);

    Ok((avatar.url, solid_background))
}

pub async fn render_all_wolvesville_avatars(ordered_urls: &Vec<String>, avatar_images: &HashMap<String, DynamicImage>) -> anyhow::Result<DynamicImage> {
    let amount_of_avatars = ordered_urls.len() as u32;
    let amount_of_avatars_on_last_row = amount_of_avatars % 3;  // 0 = 3 avatars
    let amount_of_rows = (amount_of_avatars as f32 / 3.0).ceil() as u32;

    let (avatar_width, avatar_height) = avatar_images.values().next().unwrap().dimensions();
    // 20px padding on the sides and 10px padding between avatars
    let main_image_width = avatar_width * 3 + 60;
    // 10px padding between avatars and 60px padding on the top (bottom padding is 10px)
    let main_image_height = (avatar_height + 10) * amount_of_rows + 60;

    let font = FontRef::try_from_slice(include_bytes!("../../../res/fonts/OpenSans-Bold.ttf")).expect("Error loading font");
    let scale = font.pt_to_px_scale(20.0).unwrap_or(PxScale::from(60.0));

    let mut main_image = DynamicImage::ImageRgba8(ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(
        main_image_width,
        main_image_height,
        Rgba([66, 66, 66, 255])
    ));

    draw_text_mut(
        &mut main_image,
        Rgba([255, 255, 255, 255]),
        15, 15,
        scale,
        &font,
        "Avatars"
    );

    // Place all avatars in a 3xn grid where n is the amount of rows,
    // except for the last row which has amount_of_avatars_on_last_row avatars
    for (i, url) in ordered_urls.iter().enumerate() {
        let mut x = (i as u32 % 3) * (avatar_width + 10) + 20;
        let y = (i as u32 / 3) * (avatar_height + 10) + 60;

        if i as u32 >= (amount_of_avatars - amount_of_avatars_on_last_row) {
            // amount_of_avatars_on_last_row can only be 1 or 2 here
            // if it's 1, the avatar will be centered, placed the same way as the second avatar in a row of 3
            // if it's 2, the padding between the avatars will be equal to 1/3 of the width of the avatars
            x += if amount_of_avatars_on_last_row == 1 { avatar_width + 10 }
                else { avatar_width / 3 * (i as u32 - (amount_of_avatars - 3)) };
        }

        utils::image::overlay_transparent_image(&mut main_image, avatar_images.get(url.as_str()).expect("URL mapping is wrong for some reason"), x, y);
    }

    Ok(main_image)
}