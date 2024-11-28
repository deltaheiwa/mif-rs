use crate::utils::math::calculate_percentage;

#[test]
fn test_calculate_percentage() {
    assert_eq!(calculate_percentage(1, 2), 50.0);
    assert_eq!(calculate_percentage(1, 3), 33.33);
    assert_eq!(calculate_percentage(1, 4), 25.0);
    assert_eq!(calculate_percentage(1, 256), 0.39);
    assert_eq!(calculate_percentage(1, 1000000), 0.0);
}