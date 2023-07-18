/*
 * @Author: timochan
 * @Date: 2023-07-17 11:48:02
 * @LastEditors: timochan
 * @LastEditTime: 2023-07-18 15:15:28
 * @FilePath: /processforlinux/src/get_active_window.rs
*/
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
enum WindowTitle {
    Code,
    WebStorm,
    Telgram,
    WeChat, // Linux not have it
    Discord,
    Mail,
    QQ,
    Chrome,
    QQ音乐,
    NetEaseMusic,
    iTerm2,
    Typora,
    None,
}
impl std::fmt::Display for WindowTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WindowTitle::Code => write!(f, "Code"),
            WindowTitle::WebStorm => write!(f, "WebStorm"),
            WindowTitle::Telgram => write!(f, "Telgram"),
            WindowTitle::WeChat => write!(f, "WeChat"),
            WindowTitle::Discord => write!(f, "Discord"),
            WindowTitle::Mail => write!(f, "Mail"),
            WindowTitle::QQ => write!(f, "QQ"),
            WindowTitle::Chrome => write!(f, "Chrome"),
            WindowTitle::QQ音乐 => write!(f, "QQ音乐"),
            WindowTitle::NetEaseMusic => write!(f, "NetEaseMusic"),
            WindowTitle::iTerm2 => write!(f, "iTerm2"),
            WindowTitle::Typora => write!(f, "Typora"),
            WindowTitle::None => write!(f, "None"),
        }
    }
}

pub fn get_active_window_process_and_title() -> Result<String, Box<dyn Error>> {
    let mut window_title = String::new();
    let xprop_output = Command::new("xprop")
        .arg("-root")
        .arg("_NET_ACTIVE_WINDOW")
        .stdout(Stdio::piped())
        .spawn()?;

    let xprop_stdout = xprop_output
        .stdout
        .ok_or("Failed to capture xprop stdout")?;

    let xprop_reader = BufReader::new(xprop_stdout);
    let mut active_window_id = String::new();
    for line in xprop_reader.lines() {
        let line = line?;
        if line.contains("_NET_ACTIVE_WINDOW(WINDOW)") {
            active_window_id = line.split_whitespace().nth(4).unwrap_or("").to_string();
            break;
        }
    }

    let xwininfo_output = Command::new("xwininfo")
        .arg("-id")
        .arg(active_window_id)
        .stdout(Stdio::piped())
        .spawn()?;

    let xwininfo_stdout = xwininfo_output
        .stdout
        .ok_or("Failed to capture xwininfo stdout")?;
    let xwininfo_reader = BufReader::new(xwininfo_stdout);

    for line in xwininfo_reader.lines() {
        let line = line?;
        if line.contains("xwininfo: Window id:") {
            let window_name_parts: Vec<&str> = line.split('"').collect();
            window_title = window_name_parts[1].to_string();
        }
    }
    let xwininfo_result = &window_title;
    let process_name = get_last_part(xwininfo_result).ok_or("Failed to get process name")?;
    let process_name = match process_name.as_str() {
        "Code" => WindowTitle::Code,
        "Telgram" => WindowTitle::None,  // can't get it
        "WebStorm" => WindowTitle::None, // can't get it
        "WeChat" => WindowTitle::None,   // can't get it
        "Discord" => WindowTitle::Discord, //TODO: not test
        "Thunderbird" => WindowTitle::Mail,
        "Kmail" => WindowTitle::Mail,
        "QQ" => WindowTitle::QQ,
        "Chrome" => WindowTitle::Chrome,
        "qqmusic" => WindowTitle::QQ音乐,
        "Cloud Music" => WindowTitle::NetEaseMusic,
        "Yakuake" => WindowTitle::iTerm2,
        "Konsole" => WindowTitle::iTerm2,
        "Typora" => WindowTitle::Typora,
        _ => WindowTitle::None,
    };

    Ok(process_name.to_string())
}
fn get_last_part(original_string: &str) -> Option<String> {
    let last_space_index = match original_string.rfind(' ') {
        Some(index) => index,
        None => {
            return Some(original_string.to_string());
        }
    };

    let result_string = &original_string[(last_space_index + 1)..];
    Some(result_string.to_string())
}
