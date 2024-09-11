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

pub fn heat_color(content: &str, value: f32, min_value: f32, max_value: f32) -> String {
    let scalar = if min_value == max_value { 0.5 } else { (value - min_value) / (max_value - min_value) };

    let r = (255.0 * (1.0 - scalar)) as u8;
    let g = (255.0 * scalar) as u8;
    content.truecolor(r, g, if r < 100 || g < 100 { 10 } else { 0 }).to_string()
}
