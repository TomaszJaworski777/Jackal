use utils::{bytes_to_string, number_to_string, seconds_to_string};

#[test]
fn time_to_str() {
    assert_eq!(seconds_to_string(50), "50s");
    assert_eq!(seconds_to_string(90), "1m 30s");
    assert_eq!(seconds_to_string(3690), "1h 1m 30s");
    assert_eq!(seconds_to_string(3620), "1h 0m 20s");
}

#[test]
fn number_to_str() {
    assert_eq!(number_to_string(50), "50");
    assert_eq!(number_to_string(1260), "1.26K");
    assert_eq!(number_to_string(3699), "3.70K");
    assert_eq!(number_to_string(2_254_000), "2.25M");
    assert_eq!(number_to_string(2_256_000_000), "2.26B");
}

#[test]
fn bytes_to_str() {
    assert_eq!(bytes_to_string(50), "50");
    assert_eq!(bytes_to_string(1024), "1.00K");
    assert_eq!(bytes_to_string(1_048_576), "1.00M");
    assert_eq!(bytes_to_string(1_073_741_824), "1.00G");
    assert_eq!(bytes_to_string(1_048_576 + 10240), "1.01M");
}
