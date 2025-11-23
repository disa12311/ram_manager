use eframe::egui;
use crate::ram_manager::{ProcessInfo, ProcessStatus, RamManager};

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
}

#[derive(PartialEq)]
enum SortBy {
    Memory,
    Name,
    Status,
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
            status_message: "Sẵn sàng".to_string(),
            auto_refresh: true,
            pin_working_set_mb: 512,
            limit_max_ws_mb: 256,
            sort_by: SortBy::Memory,
        }
    }
}

impl eframe::App for RamManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.auto_refresh {
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
            self.processes = self.manager.list_processes();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading("🖥️ Advanced RAM Manager");
                ui.separator();

                let sys_info = self.manager.get_system_info();
                let usage_percent = (sys_info.used_ram_gb / sys_info.total_ram_gb) * 100.0;

                ui.label(format!(
                    "💾 RAM: {:.2} / {:.2} GB ({:.1}%)",
                    sys_info.used_ram_gb, sys_info.total_ram_gb, usage_percent
                ));

                ui.separator();
                ui.checkbox(&mut self.auto_refresh, "🔄 Auto Refresh");

                if ui.button("🔃 Refresh Now").clicked() {
                    self.processes = self.manager.list_processes();
                    self.status_message = "Đã làm mới danh sách".to_string();
                }
            });
            ui.add_space(5.0);
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.label("📊 Trạng thái:");
                ui.colored_label(
                    egui::Color32::from_rgb(46, 204, 113),
                    &self.status_message,
                );
            });
            ui.add_space(3.0);
        });

        egui::SidePanel::left("process_list")
            .min_width(600.0)
            .show(ctx, |ui| {
                ui.heading("📋 Danh sách tiến trình");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("🔍 Tìm kiếm:");
                    ui.text_edit_singleline(&mut self.filter);

                    ui.separator();
                    ui.label("Sắp xếp:");
                    ui.selectable_value(&mut self.sort_by, SortBy::Memory, "RAM");
                    ui.selectable_value(&mut self.sort_by, SortBy::Name, "Tên");
                    ui.selectable_value(&mut self.sort_by, SortBy::Status, "Trạng thái");
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut filtered_processes = self.processes.clone();

                    if !self.filter.is_empty() {
                        filtered_processes.retain(|p| {
                            p.name.to_lowercase().contains(&self.filter.to_lowercase())
                                || p.pid.to_string().contains(&self.filter)
                        });
                    }

                    match self.sort_by {
                        SortBy::Memory => filtered_processes
                            .sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap()),
                        SortBy::Name => filtered_processes.sort_by(|a, b| a.name.cmp(&b.name)),
                        SortBy::Status => filtered_processes
                            .sort_by(|a, b| a.status.as_str().cmp(b.status.as_str())),
                    }

                    egui::Grid::new("process_grid")
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("PID").strong());
                            ui.label(egui::RichText::new("Tên tiến trình").strong());
                            ui.label(egui::RichText::new("RAM (MB)").strong());
                            ui.label(egui::RichText::new("Trạng thái").strong());
                            ui.end_row();

                            for proc in filtered_processes.iter() {
                                let is_selected = self.selected_pid == Some(proc.pid);

                                if ui
                                    .selectable_label(is_selected, proc.pid.to_string())
                                    .clicked()
                                {
                                    self.selected_pid = Some(proc.pid);
                                }

                                ui.label(&proc.name);
                                ui.label(format!("{:.1}", proc.memory_mb));

                                let color = proc.status.color();
                                ui.colored_label(
                                    egui::Color32::from_rgb(color[0], color[1], color[2]),
                                    proc.status.as_str(),
                                );

                                ui.end_row();
                            }
                        });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("⚙️ Bảng điều khiển");
            ui.add_space(10.0);

            if let Some(pid) = self.selected_pid {
                if let Some(proc) = self.processes.iter().find(|p| p.pid == pid) {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Tiến trình đã chọn:").strong());
                            ui.label(format!("{} (PID: {})", proc.name, proc.pid));
                        });
                        ui.label(format!("💾 RAM hiện tại: {:.1} MB", proc.memory_mb));
                        ui.horizontal(|ui| {
                            ui.label("Trạng thái:");
                            let color = proc.status.color();
                            ui.colored_label(
                                egui::Color32::from_rgb(color[0], color[1], color[2]),
                                proc.status.as_str(),
                            );
                        });
                    });

                    ui.add_space(15.0);

                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(46, 204, 113),
                            egui::RichText::new("🔒 1. Ưu tiên giữ trong RAM vật lý").strong(),
                        );
                        ui.label("Tăng working set và priority để giữ tiến trình trong bộ nhớ thật");

                        ui.horizontal(|ui| {
                            ui.label("Working Set (MB):");
                            ui.add(egui::Slider::new(
                                &mut self.pin_working_set_mb,
                                128..=4096,
                            ));
                        });

                        if ui.button("🔒 Ghim vào RAM").clicked() {
                            match self.manager.pin_to_ram(pid, self.pin_working_set_mb) {
                                Ok(msg) => self.status_message = msg,
                                Err(e) => self.status_message = format!("❌ {}", e),
                            }
                        }
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(52, 152, 219),
                            egui::RichText::new("🗜️ 2. Giảm working set / Đẩy ra nền").strong(),
                        );
                        ui.label("Thu nhỏ bộ nhớ đang dùng và hạ priority xuống thấp nhất");

                        if ui.button("🗜️ Trim Working Set").clicked() {
                            match self.manager.trim_working_set(pid) {
                                Ok(msg) => self.status_message = msg,
                                Err(e) => self.status_message = format!("❌ {}", e),
                            }
                        }
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(230, 126, 34),
                            egui::RichText::new("⚠️ 3. Giới hạn tài nguyên").strong(),
                        );
                        ui.label("Đặt giới hạn working set và giảm priority");

                        ui.horizontal(|ui| {
                            ui.label("Giới hạn tối đa (MB):");
                            ui.add(egui::Slider::new(&mut self.limit_max_ws_mb, 64..=2048));
                        });

                        if ui.button("⚠️ Áp dụng giới hạn").clicked() {
                            match self.manager.limit_resources(pid, self.limit_max_ws_mb) {
                                Ok(msg) => self.status_message = msg,
                                Err(e) => self.status_message = format!("❌ {}", e),
                            }
                        }
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::GRAY,
                            egui::RichText::new("♻️ Khôi phục về bình thường").strong(),
                        );
                        ui.label("Reset tất cả các cài đặt về mặc định");

                        if ui.button("♻️ Khôi phục").clicked() {
                            match self.manager.restore_process(pid) {
                                Ok(msg) => self.status_message = msg,
                                Err(e) => self.status_message = format!("❌ {}", e),
                            }
                        }
                    });
                } else {
                    ui.label("❌ Tiến trình không tồn tại");
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("👈 Chọn một tiến trình từ danh sách");
                    ui.label("Sau đó sử dụng các nút điều khiển bên dưới");
                });
            }

            ui.add_space(20.0);

            ui.separator();
            ui.collapsing("❓ Hướng dẫn sử dụng", |ui| {
                ui.label("🔒 Pin to RAM: Giữ tiến trình quan trọng trong bộ nhớ vật lý");
                ui.label("🗜️ Trim: Giải phóng RAM từ các tiến trình không quan trọng");
                ui.label("⚠️ Limit: Ngăn tiến trình tiêu thụ quá nhiều RAM");
                ui.label("♻️ Restore: Đưa tiến trình về trạng thái bình thường");
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("⚠️ Lưu ý: Cần chạy với quyền Administrator!")
                        .color(egui::Color32::from_rgb(231, 76, 60)),
                );
            });
        });
    }
}