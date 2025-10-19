/*
 * @Author: timochan
 * @Date: 2023-07-17 11:48:02
 * @LastEditors: timochan
 * @LastEditTime: 2023-10-30 22:22:22
 * @FilePath: /processforlinux/src/get_active_window.rs
*/

/*
 * It seems that 'xprop' can get the title directly.
 */
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

enum WindowTitle {
    Code,
    WebStorm,
    Telegram,
    WeChat,
    Discord,
    Mail,
    QQ,
    Chrome,
    QQMusic,
    NetEaseMusic,
    ITerm2,
    Typora,
    Firefox,
    Spotify,
    Slack,
    Idea,
    PyCharm,
    GoLand,
    CLion,
    AndroidStudio,
    RustRover,
    SublimeText,
    Atom,
    LibreOffice,
    VLC,
    OBS,
    None,
}

impl std::fmt::Display for WindowTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WindowTitle::Code => write!(f, "Code"),
            WindowTitle::WebStorm => write!(f, "WebStorm"),
            WindowTitle::Telegram => write!(f, "Telegram"),
            WindowTitle::CLion => write!(f, "CLion"),
            WindowTitle::WeChat => write!(f, "WeChat"),
            WindowTitle::Discord => write!(f, "Discord"),
            WindowTitle::Mail => write!(f, "Mail"),
            WindowTitle::QQ => write!(f, "QQ"),
            WindowTitle::Chrome => write!(f, "Chrome"),
            WindowTitle::QQMusic => write!(f, "QQ音乐"),
            WindowTitle::NetEaseMusic => write!(f, "NetEaseMusic"),
            WindowTitle::ITerm2 => write!(f, "iTerm2"),
            WindowTitle::Typora => write!(f, "Typora"),
            WindowTitle::Firefox => write!(f, "Firefox"),
            WindowTitle::Spotify => write!(f, "Spotify"),
            WindowTitle::Slack => write!(f, "Slack"),
            WindowTitle::Idea => write!(f, "IDEA"),
            WindowTitle::PyCharm => write!(f, "PyCharm"),
            WindowTitle::GoLand => write!(f, "GoLand"),
            WindowTitle::AndroidStudio => write!(f, "Android Studio"),
            WindowTitle::RustRover => write!(f, "RustRover"),
            WindowTitle::SublimeText => write!(f, "Sublime Text"),
            WindowTitle::Atom => write!(f, "Atom"),
            WindowTitle::LibreOffice => write!(f, "LibreOffice"),
            WindowTitle::VLC => write!(f, "VLC"),
            WindowTitle::OBS => write!(f, "OBS"),
            WindowTitle::None => write!(f, "None"),
        }
    }
}

impl WindowTitle {
    fn from_string(s: &str) -> WindowTitle {
        match s {
            "code" => WindowTitle::Code,
            "jetbrains-webstorm" => WindowTitle::WebStorm,
            "telegram" => WindowTitle::Telegram, // TODO: Can't get the title of Telegram
            "wechat" => WindowTitle::WeChat,
            "discord" => WindowTitle::Discord, // TODO: Can't test
            "thunderbird" => WindowTitle::Mail,
            "kmail" => WindowTitle::Mail, // TODO: Can't get the title of KMail
            "qq" => WindowTitle::QQ,
            "google-chrome" => WindowTitle::Chrome,
            "chromium" => WindowTitle::Chrome,
            "thorium" => WindowTitle::Chrome,
            "firefox" => WindowTitle::Firefox,
            "qqmusic" => WindowTitle::QQMusic,
            "music" => WindowTitle::NetEaseMusic,
            "yesplaymusic" => WindowTitle::NetEaseMusic,
            "spotify" => WindowTitle::Spotify,
            "yakuake" => WindowTitle::ITerm2, // TODO: Can't get the title of Yakuake
            "konsole" => WindowTitle::ITerm2, // TODO: Can't get the title of Konsole
            "gnome-terminal" => WindowTitle::ITerm2,
            "kitty" => WindowTitle::ITerm2,
            "alacritty" => WindowTitle::ITerm2,
            "typora" => WindowTitle::Typora,
            "slack" => WindowTitle::Slack,
            "jetbrains-idea" => WindowTitle::Idea,
            "jetbrains-clion" => WindowTitle::CLion,
            "jetbrains-pycharm" => WindowTitle::PyCharm,
            "jetbrains-goland" => WindowTitle::GoLand,
            "jetbrains-studio" => WindowTitle::AndroidStudio,
            "jetbrains-rustrover" => WindowTitle::RustRover,
            "sublime_text" => WindowTitle::SublimeText,
            "atom" => WindowTitle::Atom,
            "libreoffice" => WindowTitle::LibreOffice,
            "vlc" => WindowTitle::VLC,
            "obs" => WindowTitle::OBS,
            _ => WindowTitle::None,
        }
    }
}

pub fn get_active_window_process_and_title() -> Result<String, Box<dyn Error>> {
    let mut failure_count = 0;
    let max_attempts = 5; // 最大尝试次数

    loop {
        let xprop_output = Command::new("xprop")
            .arg("-root")
            .arg("_NET_ACTIVE_WINDOW")
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or("Failed to capture xprop stdout")?;

        let xprop_reader = BufReader::new(xprop_output);
        let mut window_id = String::new();
        for line in xprop_reader.lines() {
            let line = line?;
            if line.contains("_NET_ACTIVE_WINDOW(WINDOW)") {
                window_id = line.split_whitespace().nth(4).unwrap_or("").to_string();
                break;
            }
        }

        if window_id.is_empty() {
            failure_count += 1;
            if failure_count >= max_attempts {
                println!("无法获取活动窗口，返回空字符串");
                return Ok(String::new());
            }
            thread::sleep(Duration::from_millis(200));
            continue;
        }

        let xprop_output = Command::new("xprop")
            .arg("-id")
            .arg(&window_id)
            .arg("WM_CLASS")
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or("Failed to capture xprop stdout")?;

        let xprop_reader = BufReader::new(xprop_output);
        for line in xprop_reader.lines() {
            let line = line?;
            if line.contains("WM_CLASS(STRING)") {
                let class_name = line.split('"').nth(1).unwrap_or("");
                println!("class_name: {}", class_name);
                let window_title_enum = WindowTitle::from_string(class_name);

                // 如果是未知进程，直接返回空字符串而不是 "None"
                if matches!(window_title_enum, WindowTitle::None) {
                    return Ok(String::new());
                }

                return Ok(window_title_enum.to_string());
            }
        }

        failure_count += 1;
        if failure_count >= max_attempts {
            println!("达到最大尝试次数，返回空字符串");
            return Ok(String::new());
        }
        thread::sleep(Duration::from_millis(200));
    }
}
