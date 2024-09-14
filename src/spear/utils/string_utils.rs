pub struct StringUtils;
impl StringUtils {
    pub fn time_to_string(miliseconds: u128) -> String {
        if miliseconds < 1000 {
            format!("{miliseconds}ms")
        } else {
            format!("{:.2}s", miliseconds as f64 / 1000.0)
        }
    }

    pub fn large_number_to_string(number: u128) -> String {
        match number {
            0..=999 => format!("{number}"),
            1000..=999_999 => format!("{:.2}K", number as f64 / 1000.0),
            1_000_000.. => format!("{:.2}M", number as f64 / 1_000_000.0),
        }
    }
}
