# 🔨 Build Instructions - Advanced RAM Manager

Hướng dẫn build file `.exe` tối ưu cho Windows.

## 📋 Yêu cầu

- **Rust**: 1.70 trở lên
- **Windows 10/11**: 64-bit
- **Icon files**: `icon.png` và `icon.ico` trong thư mục `assets/`

## 📁 Cấu trúc thư mục

```
ram_manager/
├── assets/
│   ├── icon.png          # Icon 512x512 PNG (cho window)
│   └── icon.ico          # Icon 256x256 ICO (cho .exe file)
├── src/
│   ├── main.rs
│   ├── ram_manager.rs
│   └── gui.rs
├── build.rs              # Build script cho Windows
├── Cargo.toml
└── .gitignore
```

## 🎨 Chuẩn bị Icon

### 1. Tạo icon.png (512x512)
```bash
# Đặt file icon.png vào assets/
# Kích thước khuyến nghị: 512x512 hoặc 256x256
```

### 2. Convert PNG sang ICO
**Option A: Online converter**
- Truy cập: https://convertio.co/png-ico/
- Upload `icon.png`
- Download `icon.ico` với kích thước 256x256

**Option B: ImageMagick**
```bash
magick convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
```

**Option C: GIMP**
- Mở `icon.png` trong GIMP
- Export As → chọn `.ico`
- Select sizes: 256, 128, 64, 48, 32, 16

### 3. Đặt file vào thư mục assets/
```bash
mkdir assets
# Copy icon.png và icon.ico vào assets/
```

## 🔨 Build Commands

### Debug Build (Phát triển)
```bash
cargo build
```
- Build nhanh
- Có debug symbols
- File lớn (~50MB)
- Output: `target/debug/ram_manager.exe`

### Release Build (Production)
```bash
cargo build --release
```
- Build tối ưu
- Không có debug symbols
- File nhỏ (~5-10MB)
- Output: `target/release/ram_manager.exe`

### Release Build với optimization tối đa
```bash
# Set RUSTFLAGS cho optimization tối đa
set RUSTFLAGS=-C target-cpu=native -C opt-level=3
cargo build --release

# Hoặc trên Linux/Mac:
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release
```

## 📦 Build Script tự động

Tạo file `build.bat` (Windows):
```batch
@echo off
echo ========================================
echo  Advanced RAM Manager - Build Script
echo ========================================
echo.

echo [1/4] Checking Rust installation...
rustc --version
if errorlevel 1 (
    echo ERROR: Rust not found! Install from https://rustup.rs/
    pause
    exit /b 1
)

echo [2/4] Cleaning previous builds...
cargo clean

echo [3/4] Building release version...
cargo build --release
if errorlevel 1 (
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo [4/4] Copying executable...
copy "target\release\ram_manager.exe" "RAM_Manager_v1.0.0.exe"

echo.
echo ========================================
echo  Build completed successfully!
echo  Output: RAM_Manager_v1.0.0.exe
echo ========================================
pause
```

Chạy build script:
```bash
build.bat
```

## 🗜️ Giảm kích thước file .exe

### 1. Sử dụng UPX Compressor
```bash
# Download UPX từ https://upx.github.io/
upx --best --lzma target/release/ram_manager.exe

# Có thể giảm size từ 10MB xuống ~3MB
```

### 2. Strip symbols thủ công
```bash
strip target/release/ram_manager.exe
```

### 3. Optimize Cargo.toml
```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

## 📊 Kích thước file so sánh

| Build Type | Size | Description |
|------------|------|-------------|
| Debug | ~50MB | Có debug symbols |
| Release (default) | ~10MB | Optimized |
| Release (size opt) | ~8MB | opt-level = "z" |
| Release + UPX | ~3MB | Compressed |

## 🚀 Build cho Distribution

### 1. Build final release
```bash
cargo build --release
```

### 2. Test với quyền Administrator
```bash
# Chuột phải → Run as Administrator
target/release/ram_manager.exe
```

### 3. Rename và package
```bash
# Rename
copy target\release\ram_manager.exe RAM_Manager_v1.0.0.exe

# Tạo ZIP package
7z a RAM_Manager_v1.0.0.zip RAM_Manager_v1.0.0.exe README.md

# Hoặc tạo installer với NSIS, Inno Setup, WiX...
```

## 📝 Checklist trước khi release

- [ ] Kiểm tra icon hiển thị đúng
- [ ] Test trên Windows 10 và 11
- [ ] Test với và không có quyền admin
- [ ] Kiểm tra all features hoạt động
- [ ] Virus scan (Windows Defender, VirusTotal)
- [ ] Test trên máy sạch (không có Rust toolchain)
- [ ] Tạo README.md và changelog
- [ ] Tag version trong Git

## 🔍 Troubleshooting

### Lỗi: "icon.ico not found"
```bash
# Đảm bảo file tồn tại
dir assets\icon.ico

# Nếu không có, tạo thư mục và copy file
mkdir assets
copy path\to\your\icon.ico assets\
```

### Lỗi: "winres crate not found"
```bash
# Thêm vào Cargo.toml
[target.'cfg(windows)'.build-dependencies]
winres = "0.1"

# Rebuild
cargo clean
cargo build --release
```

### File .exe quá lớn
```bash
# Option 1: Strip symbols
strip target/release/ram_manager.exe

# Option 2: Use UPX
upx --best target/release/ram_manager.exe

# Option 3: Optimize for size
# Sửa Cargo.toml: opt-level = "z"
cargo build --release
```

### .exe bị Windows Defender block
```bash
# Add exclusion trong Windows Security
# Settings → Windows Security → Virus & threat protection
# → Manage settings → Exclusions → Add exclusion
```

## 🎯 Cross-compilation (Advanced)

Build từ Linux/Mac cho Windows:
```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install mingw-w64 (Linux)
sudo apt install mingw-w64

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

## 📦 Tạo Installer (Optional)

### Với NSIS
1. Download NSIS: https://nsis.sourceforge.io/
2. Tạo file `installer.nsi`
3. Build installer: `makensis installer.nsi`

### Với Inno Setup
1. Download Inno Setup: https://jrsoftware.org/isinfo.php
2. Tạo script với Inno Setup Compiler
3. Compile thành `.exe` installer

## 📈 Build Statistics

Xem chi tiết build:
```bash
cargo build --release --verbose

# Với timing info
cargo build --release --timings
```

## 💡 Tips

1. **Cache dependencies**: Sử dụng `sccache` để cache compilation
2. **Parallel builds**: Set `CARGO_BUILD_JOBS` để tăng tốc
3. **Clean builds**: Chạy `cargo clean` trước release builds
4. **Version bumping**: Dùng `cargo bump` để tự động tăng version

---

Happy Building! 🎉