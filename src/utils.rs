use std::process::Command;

use colored::Colorize;

pub fn clear_terminal_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "cls"])
            .spawn()
            .expect("cls command failed to start")
            .wait()
            .expect("failed to wait");
    } else {
        Command::new("clear")
            .spawn()
            .expect("clear command failed to start")
            .wait()
            .expect("failed to wait");
    };
}

const HEAT_RED: (u8, u8, u8) = (200, 71, 71);
const HEAT_YELLOW: (u8, u8, u8) = (200, 160, 71);
const HEAT_GREEN: (u8, u8, u8) = (80, 191, 71);
pub fn heat_color(content: &str, value: f32, min_value: f32, max_value: f32) -> String {
    let difference = max_value - min_value;
    let min_value = min_value + difference * 0.1;
    let max_value = max_value - difference * 0.2;

    let value = value.max(min_value).min(max_value);
    let scalar = if min_value == max_value {
        0.5
    } else {
        (value - min_value) / (max_value - min_value)
    };

    if scalar >= 0.5 {
        lerp_color(content, HEAT_YELLOW, HEAT_GREEN, (scalar - 0.5) * 2.0)
    } else {
        lerp_color(content, HEAT_RED, HEAT_YELLOW, scalar * 2.0)
    }
}

pub fn lerp_color(content: &str, a: (u8, u8, u8), b: (u8, u8, u8), value: f32) -> String {
    let (r, g, b) = lerp_color_internal(a, b, value);
    content.truecolor(r, g, b).to_string()
}

fn lerp_color_internal(a: (u8, u8, u8), b: (u8, u8, u8), value: f32) -> (u8, u8, u8) {
    let result_r = a.0 as i16 + ((b.0 as i16 - a.0 as i16) as f32 * value) as i16;
    let result_g = a.1 as i16 + ((b.1 as i16 - a.1 as i16) as f32 * value) as i16;
    let result_b = a.2 as i16 + ((b.2 as i16 - a.2 as i16) as f32 * value) as i16;
    (result_r as u8, result_g as u8, result_b as u8)
}
