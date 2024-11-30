use chrono::TimeDelta;
use crate::utils::math::calculate_percentage;
use crate::utils::time;

#[test]
fn test_calculate_percentage() {
    assert_eq!(calculate_percentage(1, 2), 50.0);
    assert_eq!((calculate_percentage(1, 3) * 100.0).round() / 100.0, 33.33);
    assert_eq!(calculate_percentage(1, 4), 25.0);
    assert_eq!((calculate_percentage(1, 256) * 100.0).round() / 100.0, 0.39);
}

#[test]
fn test_calculate_percentage_large_total() {
    assert_eq!(calculate_percentage(1, 100000), 0.001);
    assert_eq!(calculate_percentage(1, 1000000), 0.0001);
    assert_eq!(calculate_percentage(1, 10000000), 0.00001);
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