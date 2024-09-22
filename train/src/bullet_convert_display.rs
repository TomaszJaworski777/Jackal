use spear::StringUtils;

pub struct BulletConvertDisplay;
impl BulletConvertDisplay {
    pub fn print_report(
        current: u64,
        total: u64,
        wins: u64,
        draws: u64,
        loses: u64,
        unfiltered: u64,
        mate_scores: u64,
        material_advantage: u64,
    ) {
        jackal::clear_terminal_screen();
        println!("Converting value data...");
        println!("{}", Self::get_loading_bar(current, total, 50));
        println!(
            "Positions:       {}/{}",
            StringUtils::large_number_to_string(current as u128),
            StringUtils::large_number_to_string(total as u128)
        );
        println!(
            "White WDL:       ({}%/{}%/{}%)",
            (wins as f32 * 100.0 / unfiltered.max(1) as f32) as u64,
            (draws as f32 * 100.0 / unfiltered.max(1) as f32) as u64,
            (loses as f32 * 100.0 / unfiltered.max(1) as f32) as u64
        );
        println!(
            "Unfiltered:      {}",
            StringUtils::large_number_to_string(unfiltered as u128)
        );
        println!("Filters: ");
        println!(
            " - Mate score:   {}",
            StringUtils::large_number_to_string(mate_scores as u128)
        );
        println!(
            " - Material advantage:   {}",
            StringUtils::large_number_to_string(material_advantage as u128)
        );
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
