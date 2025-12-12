use std::fs;
use std::path::Path;
use std::process::Command;

pub const VALUE_NETWORK: &str = "monty_threats_with_pins.network";
pub const POLICY_NETWORK: &str = "p400exp4096pwsee007q.network";

fn main() {
    get_net(VALUE_NETWORK);
    get_net(POLICY_NETWORK);
}

fn get_net(name: &str) {
    let path_str = format!("../resources/networks/{name}");
    let path = Path::new(&path_str);
    let url = format!("https://huggingface.co/datasets/Snekkers/networks/resolve/main/{name}");

    if !path.exists() {
        println!("cargo:warning=Downloading {name} from HuggingFace...");
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let status = if cfg!(target_os = "windows") {
            if check_command("curl") {
                Command::new("curl")
                    .args(&["-L", "-o", &path_str, &url])
                    .status()
            } else {
                Command::new("powershell")
                    .args(&[
                        "-Command",
                        &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", url, path_str)
                    ])
                    .status()
            }
        } else {
            if check_command("curl") {
                Command::new("curl")
                    .args(&["-L", "-o", &path_str, &url])
                    .status()
            } else {
                Command::new("wget")
                    .args(&["-O", &path_str, &url])
                    .status()
            }
        };

        match status {
            Ok(s) if s.success() => {}, 
            _ => panic!("Failed to download file: {}. Check internet connection.", name),
        }
    }

    println!("cargo:rerun-if-changed=../resources/networks/stageD3.network");
}

fn check_command(cmd: &str) -> bool {
    if cfg!(target_os = "windows") {
        Command::new("where").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
    } else {
        Command::new("which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
    }
}