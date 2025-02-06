#![allow(unused_imports)]
use chrono::TimeDelta;
use crate::utils::*;

#[test]
fn test_calculate_percentage() {
    assert_eq!(math::calculate_percentage(1, 2), 50.0);
    assert_eq!((math::calculate_percentage(1, 3) * 100.0).round() / 100.0, 33.33);
    assert_eq!(math::calculate_percentage(1, 4), 25.0);
    assert_eq!((math::calculate_percentage(1, 256) * 100.0).round() / 100.0, 0.39);
}

#[test]
fn test_calculate_percentage_large_total() {
    assert_eq!(math::calculate_percentage(1, 100000), 0.001);
    assert_eq!(math::calculate_percentage(1, 1000000), 0.0001);
    assert_eq!(math::calculate_percentage(1, 10000000), 0.00001);
}

#[test]
fn test_pretty_time_delta() {
    assert_eq!(time::pretty_time_delta(&TimeDelta::seconds(1)), "1 second");
    assert_eq!(time::pretty_time_delta(&TimeDelta::seconds(2)), "2 seconds");
    assert_eq!(time::pretty_time_delta(&TimeDelta::seconds(60)), "1 minute");
    assert_eq!(time::pretty_time_delta(&TimeDelta::seconds(61)), "1 minute, 1 second");
    assert_eq!(time::pretty_time_delta(&TimeDelta::seconds(62)), "1 minute, 2 seconds");
    assert_eq!(time::pretty_time_delta(&TimeDelta::seconds(120)), "2 minutes");
    assert_eq!(time::pretty_time_delta(&TimeDelta::hours(1)), "1 hour");
    assert_eq!(time::pretty_time_delta(&TimeDelta::hours(2)), "2 hours");
    assert_eq!(time::pretty_time_delta(&TimeDelta::hours(24)), "1 day");
    assert_eq!(time::pretty_time_delta(&TimeDelta::hours(25)), "1 day, 1 hour");
    assert_eq!(time::pretty_time_delta(&TimeDelta::hours(26)), "1 day, 2 hours");
}

#[test]
fn test_level_rank() {
    assert_eq!(math::determine_level_rank(-1), 0);
    assert_eq!(math::determine_level_rank(0), 1);
    assert_eq!(math::determine_level_rank(9), 1);
    assert_eq!(math::determine_level_rank(10), 2);
    assert_eq!(math::determine_level_rank(19), 2);
    assert_eq!(math::determine_level_rank(419), 42);
    assert_eq!(math::determine_level_rank(420), 43);
    assert_eq!(math::determine_level_rank(499), 43);
    assert_eq!(math::determine_level_rank(500), 44);
    assert_eq!(math::determine_level_rank(999), 48);
    assert_eq!(math::determine_level_rank(1000), 49);
}

#[test]
fn test_get_first_part_of_string() {
    assert_eq!(get_first_part_of_string(&"hello world".to_string(), ' '), "hello");
    assert_eq!(get_first_part_of_string(&"hello world".to_string(), 'o'), "hell");
    assert_eq!(get_first_part_of_string(&"hello world".to_string(), 'z'), "hello world");
}

#[test]
fn test_comma_readable_number() {
    assert_eq!(comma_readable_number(0), "0");
    assert_eq!(comma_readable_number(100), "100");
    assert_eq!(comma_readable_number(1000), "1,000");
    assert_eq!(comma_readable_number(1000000), "1,000,000");
    assert_eq!(comma_readable_number(1000000000), "1,000,000,000");
    assert_eq!(comma_readable_number(-100), "-100");
    assert_eq!(comma_readable_number(-1000), "-1,000");
    assert_eq!(comma_readable_number(-0), "0");
}