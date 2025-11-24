#![windows_subsystem = "windows"] // Ẩn console window khi release

mod ram_manager;
mod gui;

use eframe::egui;
use std::env;

fn main() -> Result<(), eframe::Error> {
    // Check for admin privileges
    if !is_elevated() {
        show_admin_warning();
    }

    // Setup logging
    env::set_var("RUST_LOG", "info");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Advanced RAM Manager v1.0.0",
        options,
        Box::new(|cc| {
            // Configure fonts
            configure_fonts(&cc.egui_ctx);
            Ok(Box::new(gui::RamManagerApp::default()))
        }),
    )
}

fn is_elevated() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use windows::Win32::Security::*;
        use windows::Win32::Foundation::*;
        use windows::Win32::System::Threading::*;

        unsafe {
            let mut token: HANDLE = HANDLE::default();
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY,
                &mut token
            ).is_ok() {
                let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
                let mut size = 0u32;
                
                if GetTokenInformation(
                    token,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut _),
                    mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut size
                ).is_ok() {
                    let _ = CloseHandle(token);
                    return elevation.TokenIsElevated != 0;
                }
                let _ = CloseHandle(token);
            }
        }
    }
    false
}

fn show_admin_warning() {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd")
            .args(&[
                "/C",
                "echo Tool cần quyền Administrator! && echo. && echo Vui lòng: && echo 1. Đóng tool && echo 2. Chuột phải - Run as Administrator && pause"
            ])
            .spawn();
    }
}

fn load_icon() -> egui::IconData {
    // Load icon từ file PNG
    // Đặt file icon.png trong thư mục assets/
    let icon_bytes = include_bytes!("../assets/icon.png");
    
    match image::load_from_memory(icon_bytes) {
        Ok(image) => {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width: width as u32,
                height: height as u32,
            }
        }
        Err(_) => {
            // Fallback: Tạo icon đơn giản 32x32 màu xanh
            let size = 32;
            let mut rgba = vec![0u8; size * size * 4];
            for i in 0..size * size {
                let idx = i * 4;
                rgba[idx] = 52;      // R
                rgba[idx + 1] = 152; // G
                rgba[idx + 2] = 219; // B
                rgba[idx + 3] = 255; // A
            }
            egui::IconData {
                rgba,
                width: size as u32,
                height: size as u32,
            }
        }
    }
}

fn configure_fonts(_ctx: &egui::Context) {
    // Cấu hình fonts tùy chỉnh nếu cần
    // Ví dụ: thêm font Vietnamese
}