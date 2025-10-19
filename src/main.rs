/*
 * @Author: timochan
 * @Date: 2023-07-17 11:48:02
 * @LastEditors: timochan
 * @LastEditTime: 2023-12-11 17:33:31
 * @FilePath: /processforlinux/src/main.rs
 */
mod get_active_window;
mod get_env_file;
mod get_media;
mod reportprocess;

use chrono::Utc;

use std::process::exit;
use std::{error::Error, time::Duration};
use tokio::time::sleep;
use std::collections::HashMap;

struct Config {
    api_url: String,
    api_key: String,
    watch_time: i64,
    media_enable: bool,
    log_enable: bool,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            api_url: String::new(),
            api_key: String::new(),
            watch_time: 5,
            media_enable: true,
            log_enable: true,
        }
    }
}

// 根据进程名称获取扩展信息的函数
fn get_extend_info(process_name: &str) -> String {
    let mut extend_map = HashMap::new();

    // 添加进程名称到扩展信息的映射（匹配get_active_window.rs中的枚举值）
    extend_map.insert("idea", "要么享受着kt的爽，要么就是面向Spring开发中");
    extend_map.insert("clion", "不会有人不喜欢C++吧？ 唉依赖，也是念起CMake vcpkg conan的好了");
    extend_map.insert("code", "ESLint和Prettier天天在我的配置文件里打架");
    extend_map.insert("firefox", "CSS调试唯一指定亲爹，但产品经理的电脑上没有它");
    extend_map.insert("chrome", "Lighthouse跑分专用浏览器，只要关掉插件，我的网站就天下第一");
    extend_map.insert("iterm2", "美化半天，结果99%的时间都在看 `npm install` 的进度条");
    extend_map.insert("webstorm", "自动导入一时爽，索引项目火葬场，专治各种 'any' 写法");
    extend_map.insert("pycharm", "后端同事的快乐老家，据说那里的缩进能决定项目死活");
    extend_map.insert("goland", "新潮后端们的圣杯，据说能用interface{}写出JavaScript的感觉");
    extend_map.insert("rustrover", "类型安全 无畏并发 Cargo 启动，编译慢到让人发指");
    extend_map.insert("discord", "React/Vue/Svelte 官方指定撕逼广场");
    extend_map.insert("spotify", "专注码字BGM生成器，一首歌的时间刚好够我命名一个CSS class");
    extend_map.insert("telegram", "Vite作者的日常茶馆，前端前沿资讯的第一手信源（如果你看得懂）");
    extend_map.insert("wechat", "前端兼容性噩梦的始作俑者，梦回IE6");
    extend_map.insert("qq", "内置浏览器比微信还离谱，上古前端技术展览馆");
    extend_map.insert("slack", "代码截图和部署机器人专用公告栏，以及Giphy斗图大赛主场");
    extend_map.insert("typora", "写README.md的唯一动力，毕竟它排版比我写的UI好看多了");
    extend_map.insert("vlc", "用来播放网上下载的付费教程，2倍速是基本操作");
    extend_map.insert("obs", "录制 Bug 复现视频专用，顺便幻想自己是 live-coding 大神");
    extend_map.insert("thunderbird", "GitHub和Vercel的通知轰炸区，专门用来接收构建失败的噩耗");
    extend_map.insert("kmail", "GitHub和Vercel的通知轰炸区 II：The Sequel");
    extend_map.insert("qqmusic", "当我的Babel编译卡住时，唯一能抚慰我心灵的东西");
    extend_map.insert("music", "修复IE兼容性问题时的专用BGM播放器，评论区里都是同道中人");
    extend_map.insert("yesplaymusic", "用Electron包装的听歌神器，充分体现了前端'万物皆可JS'的黑客精神");
    extend_map.insert("studio", "Gradle syncing... @OptIn(Experimental::class)");
    extend_map.insert("sublime_text", "上古前端大神们的信仰，打开速度比我的HMR（热更新）还快");
    extend_map.insert("atom", "Electron的亲儿子，VSCode的探路石，前端圈的活化石");
    extend_map.insert("libreoffice", "当产品经理发来一个.odt格式的需求文档时，我的内心就和打开它的样式一样崩溃");

    // 检查进程名称中是否包含映射的关键词（不区分大小写）
    let process_lower = process_name.to_lowercase();
    for (key, value) in extend_map.iter() {
        if process_lower.contains(key) {
            return value.to_string();
        }
    }

    // 如果没有找到匹配的，返回空字符串
    String::new()
}

async fn run_loop(config: Config) {
    let mut last_time = Utc::now();
    let mut previous_process_name = String::new();
    let mut previous_media_metadata: get_media::MediaMetadata = get_media::MediaMetadata::default();
    loop {
        let utc_now = Utc::now();
        let media_metadata = if config.media_enable {
            get_media::get_media_metadata().unwrap_or_default()
        } else {
            get_media::MediaMetadata::default()
        };

        let process_name = match get_active_window::get_active_window_process_and_title() {
            Ok(name) => name,
            Err(e) => {
                eprintln!("Failed to get active window: {}", e);
                continue;
            }
        };

        let prev_process_name = previous_process_name.clone();
        let prev_media_metadata = previous_media_metadata.clone();

        if prev_process_name != process_name
            || prev_media_metadata != media_metadata
            || (utc_now - last_time).num_seconds() > 20
        {
            let extend_info = get_extend_info(&process_name);

            // 添加调试信息
            println!("DEBUG: 检测到的进程名称: '{}'", process_name);
            println!("DEBUG: 扩展信息: '{}'", extend_info);

            // 始终发送原始的进程名称，extend 字段独立存在
            if let Err(e) = report(
                &process_name,  // 直接发送原始进程名称
                &media_metadata.title.clone().unwrap_or_default(),
                &media_metadata.artist.clone().unwrap_or_default(),
                &media_metadata.thumbnail.clone().unwrap_or_default(),
                &extend_info,   // extend 字段独立，不影响 process
                &config,
            )
            .await
            {
                eprintln!("Failed to report: {}", e);
            }

            previous_process_name = process_name;
            previous_media_metadata = media_metadata;
            last_time = utc_now;
        } else if config.log_enable {
            let next_watch_time = utc_now
                .checked_add_signed(chrono::Duration::seconds(config.watch_time))
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S");
            let utc_now = utc_now.format("%Y-%m-%d %H:%M:%S");
            println!("--------------------------------------------------");
            println!("This Watch Time : {}", utc_now);
            println!("No change in process or media metadata");
            println!("Next Watch Time : {}", next_watch_time);
            println!("--------------------------------------------------");
        }
        let sleep_interval_secs = config.watch_time.to_string().parse::<u64>().unwrap_or(5);
        sleep(Duration::from_secs(sleep_interval_secs)).await;
    }
}

async fn report(
    process_name: &str,
    media_title: &str,
    media_artist: &str,
    media_thumbnail: &str,
    extend: &str,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    reportprocess::process_report(
        process_name,
        media_title,
        media_artist,
        media_thumbnail,
        extend,
        &config.api_key,
        &config.api_url,
        config.watch_time,
        config.log_enable,
    )
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut config = Config::default();
    match get_env_file::get_env_file() {
        Ok((api_url, api_key, watch_time, media_enable, log_enable)) => {
            config.api_url = api_url;
            config.api_key = api_key;
            config.watch_time = watch_time;
            config.media_enable = media_enable;
            config.log_enable = log_enable;
        }
        Err(e) => {
            eprintln!("Failed to get env file: {}", e);
            exit(1);
        }
    };

    run_loop(config).await;
}
