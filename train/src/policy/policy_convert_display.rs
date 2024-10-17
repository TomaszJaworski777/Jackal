use spear::StringUtils;

pub struct PolicyConvertDisplay;
impl PolicyConvertDisplay {
    pub fn print_report(current: u64, total: u64, unfiltered: u64) {
        jackal::clear_terminal_screen();
        println!("Converting value data...");
        println!("{}", Self::get_loading_bar(current, total, 50));
        println!(
            "Positions:       {}/{}",
            StringUtils::large_number_to_string(current as u128),
            StringUtils::large_number_to_string(total as u128)
        );
        println!(
            "Unfiltered:      {}",
            StringUtils::large_number_to_string(unfiltered as u128)
        );
        println!("Filters: ");
    }

    fn get_loading_bar(current: u64, total: u64, length: usize) -> String {
        let mut result = String::new();
        let filled_spots = ((current as f64 / total as f64) * length as f64) as usize;
        result.push_str("[");

        for i in 0..length {
            let character = if i < filled_spots { "#" } else { "-" };

            result.push_str(character);
        }

        result.push_str("] ");
        result
            .push_str(format!("{}%", ((current as f64 / total as f64) * 100.0) as usize).as_str());
        result
    }
}
