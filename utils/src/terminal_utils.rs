use std::process::Command;

use crate::{lerp_color, CustomColor};

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

pub fn create_loading_bar(
    length: usize,
    fill: f32,
    low_color: (u8, u8, u8),
    high_color: (u8, u8, u8),
) -> String {
    let mut result = String::from("[");

    for i in 0..length {
        let percentage = i as f32 / (length - 1) as f32;
        let char = if percentage <= fill {
            "#".custom_color(lerp_color(low_color, high_color, percentage))
        } else {
            String::from(".")
        };

        result.push_str(&char);
    }

    result.push_str(&format!("] {}%", (fill * 100.0) as usize));
    result
}

pub fn seconds_to_string(seconds: u128) -> String {
    let hh = seconds / 3600;
    let mm = (seconds - (hh * 3600)) / 60;
    let ss = seconds - (hh * 3600) - (mm * 60);

    let mut result = String::new();

    if hh > 0 {
        result.push_str(format!("{}h ", hh).as_str());
    }

    if hh > 0 || mm > 0 {
        result.push_str(format!("{}m ", mm).as_str());
    }

    result.push_str(format!("{}s", ss).as_str());

    result.trim().to_string()
}

pub fn time_to_string(miliseconds: u128) -> String {
    let mm = miliseconds / 60000;
    let ss = (miliseconds - (mm * 60000)) as f32 / 1000.0;

    let mut result = String::new();

    if mm > 0 {
        result.push_str(format!("{}m ", mm).as_str());
    }

    if ss >= 1.0 || mm > 0 {
        result.push_str(format!("{:.2}s", ss).as_str());
    } else {
        result.push_str(format!("{:.0}ms", ss * 1000.0).as_str());
    }

    result.trim().to_string()
}

pub fn number_to_string(number: u128) -> String {
    if number < 1000 {
        return format!("{number}");
    }

    let number = round(number as f64 / 1000.0);

    if number < 1000.0 {
        return format!("{:.2}K", number);
    }

    let number = round(number as f64 / 1000.0);

    if number < 1000.0 {
        return format!("{:.2}M", number);
    }

    format!("{:.2}B", round(number as f64 / 1000.0))
}

pub fn bytes_to_string(number: u128) -> String {
    if number < 1024 {
        return format!("{number}");
    }

    let number = round(number as f64 / 1024.0);

    if number < 1024.0 {
        return format!("{:.2}K", number);
    }

    let number = round(number as f64 / 1024.0);

    if number < 1024.0 {
        return format!("{:.2}M", number);
    }

    format!("{:.2}G", round(number as f64 / 1024.0))
}

fn round(number: f64) -> f64 {
    (number * 100.0).round() / 100.0
}

pub trait AlignString {
    fn align_to_right(&self, len: usize) -> String;
    fn align_to_left(&self, len: usize) -> String;
    fn align_to_center(&self, len: usize) -> String;
}

impl AlignString for String {
    fn align_to_right(&self, len: usize) -> String {
        let string_len = self.len();

        if string_len >= len {
            return self.clone();
        }

        let space = " ".repeat(len - string_len);
        format!("{space}{self}")
    }

    fn align_to_left(&self, len: usize) -> String {
        let string_len = self.len();

        if string_len >= len {
            return self.clone();
        }

        let space = " ".repeat(len - string_len);
        format!("{self}{space}")
    }

    fn align_to_center(&self, len: usize) -> String {
        let string_len = self.len();

        if string_len >= len {
            return self.clone();
        }

        let space_size = len - string_len;
        let modulo = " ".repeat(space_size % 2);
        let space = " ".repeat(space_size / 2);
        format!("{space}{self}{space}{modulo}")
    }
}

impl AlignString for &str {
    fn align_to_right(&self, len: usize) -> String {
        self.to_string().align_to_right(len)
    }

    fn align_to_left(&self, len: usize) -> String {
        self.to_string().align_to_left(len)
    }

    fn align_to_center(&self, len: usize) -> String {
        self.to_string().align_to_center(len)
    }
}
