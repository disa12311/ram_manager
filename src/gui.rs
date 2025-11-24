use eframe::egui;
use crate::ram_manager::{ProcessInfo, RamManager};

pub struct RamManagerApp {
    manager: RamManager,
    processes: Vec<ProcessInfo>,
    filter: String,
    selected_pid: Option<u32>,
    status_message: String,
    auto_refresh: bool,
    pin_working_set_mb: usize,
    limit_max_ws_mb: usize,
    sort_by: SortBy,
    show_stats: bool,
    theme: Theme,
}

#[derive(PartialEq)]
enum Theme {
    Dark,
    Light,
}

#[derive(PartialEq)]
enum SortBy {
    Memory,
    Name,
    Status,
    CPU,
}

impl Default for RamManagerApp {
    fn default() -> Self {
        let mut manager = RamManager::new();
        let processes = manager.list_processes();

        Self {
            manager,
            processes,
            filter: String::new(),
            selected_pid: None,
            status_message: "🟢 Sẵn sàng - Tool đang chạy".to_string(),
            auto_refresh: true,
            pin_working_set_mb: 512,
            limit_max_ws_mb: 256,
            sort_by: SortBy::Memory,
            show_stats: false,
            theme: Theme::Dark,
        }
    }
}

impl eframe::App for RamManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        match self.theme {
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
        }

        // Auto refresh
        if self.auto_refresh {
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
            self.processes = self.manager.list_processes();
        }

        // Top panel - System info
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("🖥️ Advanced RAM Manager v1.0.0").size(18.0));
                ui.separator();

                let sys_info = self.manager.get_system_info();
                let usage_percent = (sys_info.used_ram_gb / sys_info.total_ram_gb) * 100.0;

                // RAM progress bar với màu động
                let ram_color = if usage_percent > 90.0 {
                    egui::Color32::from_rgb(231, 76, 60)
                } else if usage_percent > 75.0 {
                    egui::Color32::from_rgb(230, 126, 34)
                } else {
                    egui::Color32::from_rgb(46, 204, 113)
                };

                ui.label("💾 RAM:");
                ui.add(
                    egui::ProgressBar::new(usage_percent as f32 / 100.0)
                        .text(format!("{:.2} / {:.2} GB ({:.1}%)", 
                            sys_info.used_ram_gb, sys_info.total_ram_gb, usage_percent))
                        .fill(ram_color)
                );

                ui.separator();
                ui.label(format!("📊 Tiến trình: {}", sys_info.process_count));

                ui.separator();
                ui.checkbox(&mut self.auto_refresh, "🔄 Auto");

                if ui.button("🔃").on_hover_text("Refresh ngay").clicked() {
                    self.processes = self.manager.list_processes();
                    self.status_message = "✅ Đã làm mới danh sách".to_string();
                }

                ui.separator();
                if ui.button(match self.theme {
                    Theme::Dark => "🌙",
                    Theme::Light => "☀️",
                }).on_hover_text("Đổi theme").clicked() {
                    self.theme = match self.theme {
                        Theme::Dark => Theme::Light,
                        Theme::Light => Theme::Dark,
                    };
                }

                if ui.button("📈").on_hover_text("Thống kê").clicked() {
                    self.show_stats = !self.show_stats;
                }
            });
            ui.add_space(5.0);
        });

        // Bottom panel - Status
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                let color = if self.status_message.starts_with("✅") {
                    egui::Color32::from_rgb(46, 204, 113)
                } else if self.status_message.starts_with("❌") {
                    egui::Color32::from_rgb(231, 76, 60)
                } else {
                    egui::Color32::from_rgb(52, 152, 219)
                };

                ui.colored_label(color, &self.status_message);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("⚠️ Chạy với quyền Administrator");
                });
            });
            ui.add_space(3.0);
        });

        // Left panel - Process list
        egui::SidePanel::left("process_list").min_width(650.0).show(ctx, |ui| {
            ui.heading("📋 Danh sách tiến trình");
            ui.add_space(5.0);

            // Filter and sort controls
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.filter)
                    .on_hover_text("Tìm theo tên hoặc PID");

                if ui.button("❌").on_hover_text("Xóa filter").clicked() {
                    self.filter.clear();
                }

                ui.separator();
                ui.label("Sắp xếp:");
                ui.selectable_value(&mut self.sort_by, SortBy::Memory, "💾 RAM");
                ui.selectable_value(&mut self.sort_by, SortBy::CPU, "⚙️ CPU");
                ui.selectable_value(&mut self.sort_by, SortBy::Name, "📝 Tên");
                ui.selectable_value(&mut self.sort_by, SortBy::Status, "🏷️ Trạng thái");
            });

            ui.separator();

            // Statistics panel
            if self.show_stats {
                ui.collapsing("📊 Thống kê", |ui| {
                    let stats = self.manager.get_statistics();
                    ui.horizontal(|ui| {
                        ui.label(format!("📌 Pinned: {}", stats.pinned_count));
                        ui.separator();
                        ui.label(format!("🗜️ Trimmed: {}", stats.trimmed_count));
                        ui.separator();
                        ui.label(format!("⚠️ Limited: {}", stats.limited_count));
                    });
                });
                ui.separator();
            }

            // Process table
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let mut filtered_processes = self.processes.clone();

                    // Apply filter
                    if !self.filter.is_empty() {
                        filtered_processes.retain(|p| {
                            p.name.to_lowercase().contains(&self.filter.to_lowercase())
                                || p.pid.to_string().contains(&self.filter)
                        });
                    }

                    // Apply sort
                    match self.sort_by {
                        SortBy::Memory => filtered_processes
                            .sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap()),
                        SortBy::CPU => filtered_processes
                            .sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()),
                        SortBy::Name => filtered_processes.sort_by(|a, b| a.name.cmp(&b.name)),
                        SortBy::Status => filtered_processes
                            .sort_by(|a, b| a.status.as_str().cmp(b.status.as_str())),
                    }

                    // Grid
                    egui::Grid::new("process_grid")
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .min_col_width(60.0)
                        .show(ui, |ui| {
                            // Header
                            ui.label(egui::RichText::new("PID").strong());
                            ui.label(egui::RichText::new("Tên tiến trình").strong());
                            ui.label(egui::RichText::new("RAM (MB)").strong());
                            ui.label(egui::RichText::new("CPU %").strong());
                            ui.label(egui::RichText::new("Trạng thái").strong());
                            ui.end_row();

                            // Rows
                            for proc in filtered_processes.iter() {
                                let is_selected = self.selected_pid == Some(proc.pid);

                                let response = ui.selectable_label(
                                    is_selected,
                                    egui::RichText::new(proc.pid.to_string())
                                        .color(if is_selected {
                                            egui::Color32::from_rgb(52, 152, 219)
                                        } else {
                                            egui::Color32::GRAY
                                        })
                                );

                                if response.clicked() {
                                    self.selected_pid = Some(proc.pid);
                                    self.status_message = format!(
                                        "🎯 Đã chọn: {} (PID: {})",
                                        proc.name, proc.pid
                                    );
                                }

                                ui.label(&proc.name);
                                ui.label(format!("{:.1}", proc.memory_mb));
                                ui.label(format!("{:.1}", proc.cpu_usage));

                                let color = proc.status.color();
                                ui.horizontal(|ui| {
                                    ui.label(proc.status.icon());
                                    ui.colored_label(
                                        egui::Color32::from_rgb(color[0], color[1], color[2]),
                                        proc.status.as_str(),
                                    );
                                });

                                ui.end_row();
                            }
                        });

                    // Total count
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label(format!(
                        "📊 Hiển thị {} / {} tiến trình",
                        filtered_processes.len(),
                        self.processes.len()
                    ));
                });
        });

        // Central panel - Control panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("⚙️ Bảng điều khiển");
            ui.add_space(10.0);

            if let Some(pid) = self.selected_pid {
                if let Some(proc) = self.processes.iter().find(|p| p.pid == pid) {
                    // Process info
                    ui.group(|ui| {
                        ui.set_min_height(80.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Tiến trình đã chọn:").strong().size(14.0));
                            ui.label(egui::RichText::new(format!("{} (PID: {})", proc.name, proc.pid)).size(14.0));
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(format!("💾 RAM: {:.1} MB", proc.memory_mb));
                            ui.separator();
                            ui.label(format!("⚙️ CPU: {:.1}%", proc.cpu_usage));
                        });
                        ui.horizontal(|ui| {
                            ui.label("🏷️ Trạng thái:");
                            let color = proc.status.color();
                            ui.label(proc.status.icon());
                            ui.colored_label(
                                egui::Color32::from_rgb(color[0], color[1], color[2]),
                                proc.status.as_str(),
                            );
                        });
                    });

                    ui.add_space(15.0);

                    // Controls
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Pin to RAM
                        ui.group(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(46, 204, 113),
                                egui::RichText::new("🔒 1. Ưu tiên giữ trong RAM vật lý").strong().size(14.0),
                            );
                            ui.label("Tăng working set và priority để giữ tiến trình trong bộ nhớ thật");

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label("Working Set:");
                                ui.add(egui::Slider::new(&mut self.pin_working_set_mb, 128..=4096)
                                    .suffix(" MB"));
                            });

                            ui.add_space(8.0);
                            if ui.button("🔒 Ghim vào RAM").clicked() {
                                match self.manager.pin_to_ram(pid, self.pin_working_set_mb) {
                                    Ok(msg) => self.status_message = msg,
                                    Err(e) => self.status_message = format!("❌ {}", e),
                                }
                            }
                        });

                        ui.add_space(10.0);

                        // Trim
                        ui.group(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(52, 152, 219),
                                egui::RichText::new("🗜️ 2. Giảm working set / Đẩy ra nền").strong().size(14.0),
                            );
                            ui.label("Thu nhỏ bộ nhớ đang dùng và hạ priority");

                            ui.add_space(8.0);
                            if ui.button("🗜️ Trim Working Set").clicked() {
                                match self.manager.trim_working_set(pid) {
                                    Ok(msg) => self.status_message = msg,
                                    Err(e) => self.status_message = format!("❌ {}", e),
                                }
                            }
                        });

                        ui.add_space(10.0);

                        // Limit
                        ui.group(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(230, 126, 34),
                                egui::RichText::new("⚠️ 3. Giới hạn tài nguyên").strong().size(14.0),
                            );
                            ui.label("Đặt giới hạn working set tối đa");

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label("Giới hạn:");
                                ui.add(egui::Slider::new(&mut self.limit_max_ws_mb, 64..=2048)
                                    .suffix(" MB"));
                            });

                            ui.add_space(8.0);
                            if ui.button("⚠️ Áp dụng giới hạn").clicked() {
                                match self.manager.limit_resources(pid, self.limit_max_ws_mb) {
                                    Ok(msg) => self.status_message = msg,
                                    Err(e) => self.status_message = format!("❌ {}", e),
                                }
                            }
                        });

                        ui.add_space(10.0);

                        // Restore
                        ui.group(|ui| {
                            ui.colored_label(
                                egui::Color32::GRAY,
                                egui::RichText::new("♻️ Khôi phục về bình thường").strong().size(14.0),
                            );
                            ui.label("Reset tất cả cài đặt về mặc định");

                            ui.add_space(8.0);
                            if ui.button("♻️ Khôi phục").clicked() {
                                match self.manager.restore_process(pid) {
                                    Ok(msg) => self.status_message = msg,
                                    Err(e) => self.status_message = format!("❌ {}", e),
                                }
                            }
                        });
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(egui::RichText::new("❌ Tiến trình không tồn tại").size(16.0));
                        if ui.button("🔄 Làm mới").clicked() {
                            self.selected_pid = None;
                            self.processes = self.manager.list_processes();
                        }
                    });
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("👈 Chọn một tiến trình từ danh sách");
                    ui.label("Click vào PID để bắt đầu");
                });
            }

            ui.add_space(20.0);
            ui.separator();
            
            // Help
            ui.collapsing("❓ Hướng dẫn", |ui| {
                ui.label("🔒 Pin: Giữ tiến trình trong RAM vật lý");
                ui.label("🗜️ Trim: Giải phóng RAM không dùng");
                ui.label("⚠️ Limit: Giới hạn RAM tối đa");
                ui.label("♻️ Restore: Đưa về trạng thái ban đầu");
            });
        });
    }
}