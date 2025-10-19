/*
 * @Author: grtsinry43
 * @Date: 2025-10-19
 * @FilePath: /processforlinux/src/status_window.rs
 */

use chrono::{DateTime, Utc};
use egui_overlay::EguiOverlay;
use std::sync::mpsc;
use std::time::Duration;

/// 应用运行状态数据
#[derive(Debug, Clone)]
pub struct AppStatus {
    pub session_type: String,
    pub current_window: String,
    pub window_status: WindowStatus,
    pub next_check_time: DateTime<Utc>,
    pub watch_interval: i64, // 检测间隔（秒）
    pub media_title: String,
    pub media_artist: String,
    pub media_thumbnail: String,
    pub stats: RunningStats,
    pub last_error: Option<String>,
}

/// 窗口获取状态
#[derive(Debug, Clone)]
pub enum WindowStatus {
    Success,
    Failed(String),
    Checking,
}

/// 运行统计信息
#[derive(Debug, Clone)]
pub struct RunningStats {
    pub start_time: DateTime<Utc>,
    pub success_count: u64,
    pub failure_count: u64,
    pub total_checks: u64,
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            session_type: "检测中...".to_string(),
            current_window: "无".to_string(),
            window_status: WindowStatus::Checking,
            next_check_time: Utc::now(),
            watch_interval: 5, // 默认5秒
            media_title: String::new(),
            media_artist: String::new(),
            media_thumbnail: String::new(),
            stats: RunningStats {
                start_time: Utc::now(),
                success_count: 0,
                failure_count: 0,
                total_checks: 0,
            },
            last_error: None,
        }
    }
}

/// 状态窗口应用
pub struct StatusWindow {
    pub status: AppStatus,
    pub receiver: mpsc::Receiver<AppStatus>,
    font_loaded: bool,
}

impl StatusWindow {
    pub fn new(receiver: mpsc::Receiver<AppStatus>) -> Self {
        Self {
            status: AppStatus::default(),
            receiver,
            font_loaded: false,
        }
    }

    fn format_duration(&self, duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}秒", secs)
        } else if secs < 3600 {
            format!("{}分{}秒", secs / 60, secs % 60)
        } else {
            format!("{}时{}分", secs / 3600, (secs % 3600) / 60)
        }
    }

    fn get_status_emoji(&self) -> &str {
        match &self.status.window_status {
            WindowStatus::Success => "✅",
            WindowStatus::Failed(_) => "❌",
            WindowStatus::Checking => "🔍",
        }
    }

    fn get_session_emoji(&self) -> &str {
        match self.status.session_type.as_str() {
            "wayland" => "W",
            "x11" => "X",
            _ => "?",
        }
    }

    fn time_until_next_check(&self) -> i64 {
        (self.status.next_check_time - Utc::now()).num_seconds().max(0)
    }

    fn setup_fonts(&mut self, ctx: &egui::Context) {
        if self.font_loaded {
            return;
        }

        let mut fonts = egui::FontDefinitions::default();

        // 尝试加载系统中的中文字体
        let font_paths = vec![
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/OTF/NotoSansCJK-Regular.ttc",
        ];

        for font_path in font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "noto_sans_cjk".to_owned(),
                    egui::FontData::from_owned(font_data),
                );

                // 将中文字体添加到所有字体族
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "noto_sans_cjk".to_owned());

                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("noto_sans_cjk".to_owned());

                ctx.set_fonts(fonts);
                self.font_loaded = true;
                eprintln!("成功加载中文字体: {}", font_path);
                return;
            }
        }

        eprintln!("警告: 未找到中文字体，中文显示可能不正常");
        self.font_loaded = true;
    }
}

impl EguiOverlay for StatusWindow {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut egui_overlay::egui_render_three_d::ThreeDBackend,
        glfw_backend: &mut egui_overlay::egui_window_glfw_passthrough::GlfwBackend,
    ) {
        // 设置中文字体（仅第一次调用时）
        self.setup_fonts(egui_context);

        // 接收状态更新
        if let Ok(new_status) = self.receiver.try_recv() {
            self.status = new_status;
        }

        // 使用 Area 而不是 Window，这样可以自由拖拽
        egui::Area::new(egui::Id::new("status_area"))
            .movable(true)
            .default_pos(egui::pos2(20.0, 20.0))
            .show(egui_context, |ui| {
                // 创建一个带圆角的半透明背景框
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(20, 20, 25, 220)) // 更深的背景
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);
                        ui.spacing_mut().item_spacing.y = 4.0;

                // 标题区域
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("● Process Monitor")
                            .size(10.0)
                            .color(egui::Color32::from_rgba_premultiplied(100, 116, 139, 200))
                    );
                });

                ui.add_space(6.0);

                // 第一行：会话类型 + 状态
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // 会话类型 - 无背景
                    ui.label(
                        egui::RichText::new(format!("{} {}", self.get_session_emoji(), &self.status.session_type))
                            .size(9.0)
                            .color(egui::Color32::from_rgba_premultiplied(147, 197, 253, 180))
                    );

                    // 状态 - 无背景
                    let (status_text, status_fg) = match &self.status.window_status {
                        WindowStatus::Success => (
                            format!("{} 运行中", self.get_status_emoji()),
                            egui::Color32::from_rgba_premultiplied(134, 239, 172, 180)
                        ),
                        WindowStatus::Failed(_) => (
                            format!("{} 失败", self.get_status_emoji()),
                            egui::Color32::from_rgba_premultiplied(252, 165, 165, 200)
                        ),
                        WindowStatus::Checking => (
                            format!("{} 检测中", self.get_status_emoji()),
                            egui::Color32::from_rgba_premultiplied(253, 224, 71, 200)
                        ),
                    };

                    ui.label(
                        egui::RichText::new(status_text)
                            .size(9.0)
                            .color(status_fg)
                    );
                });

                // 当前窗口信息
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label(
                        egui::RichText::new("▶")
                            .size(9.0)
                            .color(egui::Color32::from_rgba_premultiplied(139, 92, 246, 200))
                    );
                    let window_name = if self.status.current_window.is_empty() {
                        "无".to_string()
                    } else {
                        let name = &self.status.current_window;
                        if name.len() > 18 {
                            format!("{}...", &name[..15])
                        } else {
                            name.clone()
                        }
                    };
                    ui.label(
                        egui::RichText::new(window_name)
                            .size(9.0)
                            .color(egui::Color32::from_rgba_premultiplied(203, 213, 225, 255))
                    );
                });

                // 检测倒计时进度条 - 小而隐秘
                ui.add_space(6.0);
                let countdown = self.time_until_next_check();
                let total_time = self.status.watch_interval;
                let progress = 1.0 - (countdown as f32 / total_time as f32).clamp(0.0, 1.0);

                let bar_width = 120.0; // 固定宽度，不撑满
                let bar_height = 1.5; // 更细
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(bar_width, bar_height),
                    egui::Sense::hover()
                );

                // 背景 - 更透明
                ui.painter().rect_filled(
                    rect,
                    0.5,
                    egui::Color32::from_rgba_premultiplied(71, 85, 105, 40)
                );

                // 进度 - 更隐蔽的颜色
                let progress_width = rect.width() * progress;
                let progress_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(progress_width, rect.height())
                );
                ui.painter().rect_filled(
                    progress_rect,
                    0.5,
                    egui::Color32::from_rgba_premultiplied(139, 92, 246, 80)
                );

                // 媒体信息
                if !self.status.media_title.is_empty() || !self.status.media_artist.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    if !self.status.media_title.is_empty() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.label(
                                egui::RichText::new("♬")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgba_premultiplied(251, 146, 60, 220))
                            );
                            let title = if self.status.media_title.len() > 22 {
                                format!("{}...", &self.status.media_title[..19])
                            } else {
                                self.status.media_title.clone()
                            };
                            ui.label(
                                egui::RichText::new(title)
                                    .size(9.0)
                                    .color(egui::Color32::from_rgba_premultiplied(226, 232, 240, 255))
                            );
                        });
                    }
                    if !self.status.media_artist.is_empty() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.label(
                                egui::RichText::new("♪")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgba_premultiplied(251, 146, 60, 180))
                            );
                            let artist = if self.status.media_artist.len() > 22 {
                                format!("{}...", &self.status.media_artist[..19])
                            } else {
                                self.status.media_artist.clone()
                            };
                            ui.label(
                                egui::RichText::new(artist)
                                    .size(9.0)
                                    .color(egui::Color32::from_rgba_premultiplied(148, 163, 184, 255))
                            );
                        });
                    }
                }

                // 统计信息（可折叠）
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                egui::CollapsingHeader::new(
                    egui::RichText::new("📊 统计")
                        .size(9.0)
                        .color(egui::Color32::from_rgba_premultiplied(100, 116, 139, 200))
                )
                .default_open(false)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;

                    // 运行时间和检测次数
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let running_time = Utc::now() - self.status.stats.start_time;
                        ui.label(
                            egui::RichText::new(format!("运行 {}", self.format_duration(running_time.to_std().unwrap_or_default())))
                                .size(8.0)
                                .color(egui::Color32::from_rgba_premultiplied(148, 163, 184, 255))
                        );

                        ui.label(
                            egui::RichText::new(format!("检测 {}", self.status.stats.total_checks))
                                .size(8.0)
                                .color(egui::Color32::from_rgba_premultiplied(148, 163, 184, 255))
                        );
                    });

                    // 成功失败统计
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.label(
                            egui::RichText::new(format!("✓ {}", self.status.stats.success_count))
                                .size(8.0)
                                .color(egui::Color32::from_rgba_premultiplied(134, 239, 172, 255))
                        );
                        ui.label(
                            egui::RichText::new(format!("✗ {}", self.status.stats.failure_count))
                                .size(8.0)
                                .color(egui::Color32::from_rgba_premultiplied(252, 165, 165, 255))
                        );

                        if self.status.stats.total_checks > 0 {
                            let success_rate =
                                (self.status.stats.success_count as f64 / self.status.stats.total_checks as f64) * 100.0;

                            ui.label(
                                egui::RichText::new(format!("{:.0}%", success_rate))
                                    .size(8.0)
                                    .color(egui::Color32::from_rgba_premultiplied(203, 213, 225, 255))
                            );
                        }
                    });

                    // 成功率进度条
                    if self.status.stats.total_checks > 0 {
                        let success_rate =
                            (self.status.stats.success_count as f64 / self.status.stats.total_checks as f64) * 100.0;

                        ui.add_space(2.0);
                        let progress_bar_height = 3.0;
                        let (rect, _response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), progress_bar_height),
                            egui::Sense::hover()
                        );

                        let progress_color = if success_rate > 90.0 {
                            egui::Color32::from_rgba_premultiplied(34, 197, 94, 200)
                        } else if success_rate > 70.0 {
                            egui::Color32::from_rgba_premultiplied(234, 179, 8, 200)
                        } else {
                            egui::Color32::from_rgba_premultiplied(239, 68, 68, 200)
                        };

                        // 背景
                        ui.painter().rect_filled(
                            rect,
                            1.5,
                            egui::Color32::from_rgba_premultiplied(71, 85, 105, 100)
                        );

                        // 进度
                        let progress_width = rect.width() * (success_rate / 100.0) as f32;
                        let progress_rect = egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(progress_width, rect.height())
                        );
                        ui.painter().rect_filled(
                            progress_rect,
                            1.5,
                            progress_color
                        );
                    }
                });

                // 错误信息
                if let Some(error) = &self.status.last_error {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("⚠ {}", error))
                            .size(8.0)
                            .color(egui::Color32::from_rgba_premultiplied(252, 165, 165, 255))
                    );
                }
            });
        });

        // 关键：根据 egui 是否需要鼠标输入来动态设置穿透
        // 如果 egui 不需要鼠标输入（即鼠标不在任何控件上），启用穿透
        let wants_pointer = egui_context.wants_pointer_input();
        glfw_backend.set_passthrough(!wants_pointer);
    }
}

/// 启动状态窗口
pub fn run_status_window(receiver: mpsc::Receiver<AppStatus>) {
    use egui_overlay::egui_window_glfw_passthrough::{GlfwBackend, GlfwConfig};
    use egui_overlay::egui_render_three_d::ThreeDBackend;

    let mut glfw_backend = GlfwBackend::new(GlfwConfig {
        glfw_callback: Box::new(|glfw| {
            glfw.window_hint(egui_overlay::egui_window_glfw_passthrough::glfw::WindowHint::TransparentFramebuffer(true));
            glfw.window_hint(egui_overlay::egui_window_glfw_passthrough::glfw::WindowHint::Decorated(false));
            glfw.window_hint(egui_overlay::egui_window_glfw_passthrough::glfw::WindowHint::Floating(true));
            glfw.window_hint(egui_overlay::egui_window_glfw_passthrough::glfw::WindowHint::Maximized(true));
        }),
        opengl_window: Some(true),
        transparent_window: Some(true),
        ..Default::default()
    });

    // 设置窗口属性
    glfw_backend.window.set_floating(true);
    glfw_backend.window.set_decorated(false);

    // 全屏化窗口（无边框全屏）
    let screen_size = glfw_backend.glfw.with_primary_monitor(|_, monitor| {
        monitor.and_then(|m| m.get_video_mode().map(|vm| (vm.width as i32, vm.height as i32)))
    });

    if let Some((width, height)) = screen_size {
        glfw_backend.window.set_size(width, height);
        glfw_backend.window.set_pos(0, 0);
    }

    let latest_size = glfw_backend.window.get_framebuffer_size();
    let latest_size = [latest_size.0 as _, latest_size.1 as _];

    let default_gfx_backend = ThreeDBackend::new(
        egui_overlay::egui_render_three_d::ThreeDConfig::default(),
        |s| glfw_backend.get_proc_address(s),
        latest_size,
    );

    let overlap_app = egui_overlay::OverlayApp {
        user_data: StatusWindow::new(receiver),
        egui_context: Default::default(),
        default_gfx_backend,
        glfw_backend,
    };

    overlap_app.enter_event_loop();
}
