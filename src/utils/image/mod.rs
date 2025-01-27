pub mod wolvesville;
use reqwest::get;
use image::{load_from_memory, DynamicImage, ImageBuffer, Pixel, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;

async fn get_image_by_url(url: &str) -> anyhow::Result<DynamicImage> {
    let bytes = get(url).await?.bytes().await?;
    Ok(load_from_memory(&bytes)?)
}

fn overlay_transparent_image(background: &mut DynamicImage, overlay: &DynamicImage, offset_x: u32, offset_y: u32) {
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

fn create_rounded_rectangle_mask(width: u32, height: u32, radius: f32) -> RgbaImage {
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(width, height, Rgba([0, 0, 0, 0]));
    let color = Rgba([255, 255, 255, 255]);

    // Fill the central rectangle
    draw_filled_rect_mut(
        &mut img,
        Rect::at(radius as i32, radius as i32).of_size(width - 2 * radius as u32, height - 2 * radius as u32),
        color,
    );

    // Fill the side rectangles
    draw_filled_rect_mut(
        &mut img,
        Rect::at(radius as i32, 0).of_size(width - 2 * radius as u32, radius as u32),
        color,
    );
    draw_filled_rect_mut(
        &mut img,
        Rect::at(radius as i32, height as i32 - radius as i32).of_size(width - 2 * radius as u32, radius as u32),
        color,
    );

    // Fill the top and bottom rectangles
    draw_filled_rect_mut(
        &mut img,
        Rect::at(0, radius as i32).of_size(radius as u32, height - 2 * radius as u32),
        color,
    );
    draw_filled_rect_mut(
        &mut img,
        Rect::at(width as i32 - radius as i32, radius as i32).of_size(radius as u32, height - 2 * radius as u32),
        color,
    );

    // Draw filled circles for the corners
    draw_filled_circle_mut(&mut img, (radius as i32, radius as i32), radius as i32, color);
    draw_filled_circle_mut(&mut img, (width as i32 - radius as i32, radius as i32), radius as i32, color);
    draw_filled_circle_mut(&mut img, (radius as i32, height as i32 - radius as i32), radius as i32, color);
    draw_filled_circle_mut(&mut img, (width as i32 - radius as i32, height as i32 - radius as i32), radius as i32, color);

    img
}
fn apply_mask(image: &DynamicImage, mask: &RgbaImage) -> DynamicImage {
    let mut rounded_image = image.to_rgba8();
    for y in 0..rounded_image.height() {
        for x in 0..rounded_image.width() {
            let mask_pixel = mask.get_pixel(x, y);
            if mask_pixel[3] == 0 { // If the pixel is transparent
                rounded_image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }
    DynamicImage::ImageRgba8(rounded_image)
}

