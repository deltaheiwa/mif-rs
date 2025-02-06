pub mod language;
pub mod logger;
pub mod time;
pub mod apicallers;
pub mod image;
pub mod math;
mod tests;

pub fn get_first_part_of_string(input: &String, delimiter: char) -> String {
    input.split_once(delimiter).map_or(input.clone(), |(first, _)| first.to_string())
}

pub fn comma_readable_number(number: i64) -> String {
    let mut result = String::new();
    let number_string = number.abs().to_string();
    let mut count = 0;
    for c in number_string.chars().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(c);
        count += 1;
    }
    if number < 0 {
        result.push('-');
    }
    result.chars().rev().collect()
}