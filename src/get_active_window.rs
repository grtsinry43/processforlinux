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
use std::fs;
use std::env;

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

/// 检测当前会话类型
pub fn detect_session_type() -> String {
    // 首先检查 XDG_SESSION_TYPE 环境变量
    if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
        return session_type;
    }

    // 检查 WAYLAND_DISPLAY 环境变量
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return "wayland".to_string();
    }

    // 检查 DISPLAY 环境变量
    if env::var("DISPLAY").is_ok() {
        return "x11".to_string();
    }

    // 默认返回未知
    "unknown".to_string()
}

/// 检测可用的 D-Bus 工具
fn detect_dbus_tool() -> Option<String> {
    // 按优先级尝试不同的 D-Bus 工具
    let tools = ["qdbus6", "qdbus"];

    for tool in &tools {
        if Command::new(tool).arg("--version").output().is_ok() {
            return Some(tool.to_string());
        }
    }

    None
}

/// 在 Wayland KDE Plasma 环境下获取活动窗口
fn get_active_window_wayland_kde() -> Result<String, Box<dyn Error>> {
    // 检测可用的 D-Bus 工具
    let dbus_tool = detect_dbus_tool()
        .ok_or("未找到可用的 D-Bus 工具 (qdbus6/qdbus)")?;

    println!("使用 D-Bus 工具: {}", dbus_tool);

    // 创建临时的 KWin 脚本文件
    let script_content = r#"
// 获取当前活动窗口
try {
    const activeWindow = workspace.activeWindow;
    if (activeWindow && activeWindow.resourceClass) {
        print("ACTIVE_WINDOW:" + activeWindow.resourceClass.toString());
    } else {
        print("ACTIVE_WINDOW:");
    }
} catch (e) {
    print("ERROR:" + e.toString());
}
"#;

    let temp_dir = env::temp_dir();
    let script_path = temp_dir.join("kwin_get_active_window.js");

    // 写入脚本内容
    fs::write(&script_path, script_content)?;

    // 记录当前时间，用于过滤 journalctl 输出
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // 使用检测到的 D-Bus 工具加载脚本
    let load_output = Command::new(&dbus_tool)
        .arg("org.kde.KWin")
        .arg("/Scripting")
        .arg("org.kde.kwin.Scripting.loadScript")
        .arg(script_path.to_string_lossy().as_ref())
        .output()?;

    if !load_output.status.success() {
        let error_msg = String::from_utf8_lossy(&load_output.stderr);
        return Err(format!("Failed to load KWin script with {}: {}", dbus_tool, error_msg).into());
    }

    // 获取脚本编号
    let script_id = String::from_utf8(load_output.stdout)?
        .trim()
        .to_string();

    if script_id.is_empty() {
        return Err("Failed to get script ID".into());
    }

    println!("KWin 脚本 ID: {}", script_id);

    // 运行脚本
    let run_output = Command::new(&dbus_tool)
        .arg("org.kde.KWin")
        .arg(format!("/Scripting/Script{}", script_id))
        .arg("org.kde.kwin.Script.run")
        .output()?;

    if !run_output.status.success() {
        let error_msg = String::from_utf8_lossy(&run_output.stderr);
        return Err(format!("Failed to run KWin script: {}", error_msg).into());
    }

    // 停止脚本
    let _ = Command::new(&dbus_tool)
        .arg("org.kde.KWin")
        .arg(format!("/Scripting/Script{}", script_id))
        .arg("org.kde.kwin.Script.stop")
        .output();

    // 等待一小段时间让脚本输出到日志
    thread::sleep(Duration::from_millis(200));

    // 从 journalctl 获取输出
    let journal_output = Command::new("journalctl")
        .arg("_COMM=kwin_wayland")
        .arg("-o")
        .arg("cat")
        .arg("--since")
        .arg(format!("@{}", current_time))
        .arg("--no-pager")
        .output()?;

    // 清理临时文件
    let _ = fs::remove_file(&script_path);

    if !journal_output.status.success() {
        return Err("Failed to get journal output".into());
    }

    let journal_content = String::from_utf8_lossy(&journal_output.stdout);
    println!("Journal 输出:\n{}", journal_content);

    // 解析输出
    for line in journal_content.lines() {
        if line.contains("ACTIVE_WINDOW:") {
            let class_name = line.split("ACTIVE_WINDOW:").nth(1).unwrap_or("").trim();
            if !class_name.is_empty() {
                println!("class_name (Wayland): {}", class_name);
                let window_title_enum = WindowTitle::from_string(class_name);

                // 如果是未知进程，直接返回空字符串
                if matches!(window_title_enum, WindowTitle::None) {
                    return Ok(String::new());
                }

                return Ok(window_title_enum.to_string());
            }
        }
        if line.contains("ERROR:") {
            let error_msg = line.split("ERROR:").nth(1).unwrap_or("").trim();
            return Err(format!("KWin script error: {}", error_msg).into());
        }
    }

    Ok(String::new())
}

/// 在 X11 环境下获取活动窗口 (原有实现)
fn get_active_window_x11() -> Result<String, Box<dyn Error>> {
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
                println!("class_name (X11): {}", class_name);
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

pub fn get_active_window_process_and_title() -> Result<String, Box<dyn Error>> {
    let session_type = detect_session_type();
    println!("检测到的会话类型: {}", session_type);

    match session_type.as_str() {
        "wayland" => {
            // 检查是否是 KDE Plasma
            if env::var("KDE_SESSION_VERSION").is_ok() ||
               env::var("DESKTOP_SESSION").unwrap_or_default().contains("plasma") ||
               env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().contains("KDE") {
                println!("检测到 KDE Plasma Wayland 环境，使用 KWin 脚本方法");
                get_active_window_wayland_kde()
            } else {
                println!("Wayland 环境下非 KDE，暂不支持");
                Ok(String::new())
            }
        }
        "x11" => {
            println!("检测到 X11 环境，使用 xprop 方法");
            get_active_window_x11()
        }
        _ => {
            println!("未知的会话类型 ({}), 尝试 X11 方法", session_type);
            get_active_window_x11()
        }
    }
}
