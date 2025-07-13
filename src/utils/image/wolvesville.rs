use std::collections::HashMap;
use std::convert::Into;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use anyhow::anyhow;
use charts_rs::{THEME_GRAFANA, svg_to_png, Series, SeriesCategory, Align, LineChart};
use chrono::{DateTime, Duration, Timelike, Utc};
use image::{open, DynamicImage, ImageBuffer, Rgba};
use imageproc::drawing::{draw_text_mut, Canvas};
use crate::db::wolvesville::player::SPRecord;
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

/// Draws a skill points plot for a player based on their skill points records.
///
/// # Arguments
/// * `data` - A vector of `SPRecord` containing the skill points records ordered by timestamp.
/// * `player_name` - The name of the player to be displayed in the plot title.
/// * `language` - The language code for localization of the plot text.
pub fn draw_sp_plot(data: &Vec<SPRecord>, player_name: &String, language: &String) -> anyhow::Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow!("Input data cannot be empty."));
    }

    let (timestamps_strings, skill_points_data) = prepare_line_chart_data(data)
        .map_err(|e| anyhow!("Failed to prepare line chart data: {}", e))?;

    let min_skill = data.iter().map(|r| r.skill).min().unwrap_or(0) as f32;
    let max_skill = data.iter().map(|r| r.skill).max().unwrap_or(0) as f32;
    
    let mut series = Series::new(
        t!(
            "commands.wov.player.search.buttons.sp_plot.series_label",
            player_name = player_name,
            locale = language
        ).into(),
        skill_points_data
    );
    
    series.category = Some(SeriesCategory::Line);
    
    let mut plot = LineChart::new_with_theme(
        vec![series],
        timestamps_strings,
        THEME_GRAFANA);
    
    plot.title_text = t!(
            "commands.wov.player.search.buttons.sp_plot.caption",
            player_name = player_name,
            locale = language
        ).into();

    plot.title_margin = Some(charts_rs::Box::from(10.0));

    // c t
    plot.y_axis_configs[0].axis_formatter = Some("{t}".to_string());
    plot.y_axis_configs[0].axis_min = Some(min_skill - 100.0);
    plot.y_axis_configs[0].axis_max = Some(max_skill + 100.0);

    plot.legend_align = Align::Right;

    let image_buffer = svg_to_png(&*plot.svg()?)
        .map_err(|e| anyhow!("Failed to convert SVG to PNG: {}", e))?;

    Ok(image_buffer)
}

enum Granularity {
    Daily,
    Hourly,
}

fn prepare_line_chart_data(
    data: &Vec<SPRecord>,
) -> anyhow::Result<(Vec<String>, Vec<f32>)> {
    if data.len() < 2 {
        return Err(anyhow!(
            "At least two data points are required for interpolation."
        ));
    }

    let first_record = data.first().unwrap();
    let last_record = data.last().unwrap();

    let granularity = if last_record.timestamp - first_record.timestamp
        >= Duration::days(1)
    {
        Granularity::Daily
    } else {
        Granularity::Hourly
    };

    let (step_duration, date_format) = match granularity {
        Granularity::Daily => (Duration::days(1), "%Y-%m-%d"),
        Granularity::Hourly => (Duration::hours(1), "%Y-%m-%d %H:00"),
    };

    let normalize = |ts: DateTime<Utc>| match granularity {
        Granularity::Daily => ts.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        Granularity::Hourly => ts
            .with_minute(0)
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap(),
    };

    let mut known_points_map = HashMap::new();
    for record in data {
        let key = normalize(record.timestamp).timestamp();
        known_points_map.insert(key, record.skill as f32);
    }

    let mut known_points: Vec<(i64, f32)> =
        known_points_map.into_iter().collect();
    known_points.sort_unstable_by_key(|k| k.0);

    if known_points.is_empty() {
        return Ok((vec![], vec![]));
    }
    let mut filled_data = Vec::new();

    let mut last_known = known_points[0];
    filled_data.push(last_known);

    for i in 1..known_points.len() {
        let next_known = known_points[i];
        let time_diff = next_known.0 - last_known.0;


        if time_diff > step_duration.num_seconds() {
            let value_diff = next_known.1 - last_known.1;
            let num_steps_in_gap =
                (time_diff / step_duration.num_seconds()) as f32;
            let value_per_step = value_diff / num_steps_in_gap;

            for step_num in 1..(num_steps_in_gap as i64) {
                let interpolated_timestamp =
                    last_known.0 + (step_duration.num_seconds() * step_num);
                let interpolated_value =
                    last_known.1 + (value_per_step * step_num as f32);
                filled_data.push((interpolated_timestamp, interpolated_value));
            }
        }

        filled_data.push(next_known);
        last_known = next_known;
    }

    let mut labels = Vec::with_capacity(filled_data.len());
    let mut values = Vec::with_capacity(filled_data.len());

    for (timestamp, value) in filled_data {
        let dt = DateTime::from_timestamp(timestamp, 0).unwrap();
        labels.push(dt.format(date_format).to_string());
        values.push(value);
    }

    Ok((labels, values))
}