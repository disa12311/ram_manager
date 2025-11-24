@echo off
chcp 65001 >nul
color 0A

echo.
echo ╔════════════════════════════════════════════════════════╗
echo ║     Advanced RAM Manager - Build Script v1.0           ║
echo ╚════════════════════════════════════════════════════════╝
echo.

REM Check Rust installation
echo [1/6] 🔍 Checking Rust installation...
rustc --version >nul 2>&1
if errorlevel 1 (
    color 0C
    echo ❌ ERROR: Rust not found!
    echo.
    echo Please install Rust from: https://rustup.rs/
    echo.
    pause
    exit /b 1
)
rustc --version
echo ✅ Rust found!
echo.

REM Check cargo
echo [2/6] 📦 Checking Cargo...
cargo --version
echo ✅ Cargo ready!
echo.

REM Check icon files
echo [3/6] 🎨 Checking icon files...
if not exist "assets\icon.png" (
    color 0E
    echo ⚠️  WARNING: assets\icon.png not found!
    echo    Creating fallback icon...
    if not exist "assets" mkdir assets
)
if not exist "assets\icon.ico" (
    color 0E
    echo ⚠️  WARNING: assets\icon.ico not found!
    echo    App will use default icon
)
echo.

REM Clean previous builds
echo [4/6] 🧹 Cleaning previous builds...
cargo clean
if exist "RAM_Manager_v1.0.0.exe" del "RAM_Manager_v1.0.0.exe"
echo ✅ Cleaned!
echo.

REM Build release
echo [5/6] 🔨 Building release version...
echo This may take a few minutes...
echo.
cargo build --release
if errorlevel 1 (
    color 0C
    echo.
    echo ❌ ERROR: Build failed!
    echo.
    echo Please check the error messages above.
    pause
    exit /b 1
)
echo.
echo ✅ Build successful!
echo.

REM Copy and rename executable
echo [6/6] 📋 Creating final executable...
if exist "target\release\ram_manager.exe" (
    copy "target\release\ram_manager.exe" "RAM_Manager_v1.0.0.exe" >nul
    echo ✅ Executable created: RAM_Manager_v1.0.0.exe
) else (
    color 0C
    echo ❌ ERROR: Built executable not found!
    pause
    exit /b 1
)
echo.

REM Show file info
echo ╔════════════════════════════════════════════════════════╗
echo ║              Build Information                         ║
echo ╚════════════════════════════════════════════════════════╝
echo.
for %%F in (RAM_Manager_v1.0.0.exe) do (
    echo 📁 File: %%~nxF
    echo 📊 Size: %%~zF bytes ^(~%%~zF / 1048576 MB^)
    echo 📍 Path: %cd%\%%~nxF
)
echo.

REM Optional: UPX compression
where upx >nul 2>&1
if not errorlevel 1 (
    echo ╔════════════════════════════════════════════════════════╗
    echo ║           Optional: UPX Compression                    ║
    echo ╚════════════════════════════════════════════════════════╝
    echo.
    choice /C YN /M "Do you want to compress with UPX? (Y/N)"
    if errorlevel 2 goto skip_upx
    if errorlevel 1 (
        echo.
        echo 🗜️  Compressing with UPX...
        upx --best --lzma RAM_Manager_v1.0.0.exe
        echo ✅ Compressed!
        echo.
        for %%F in (RAM_Manager_v1.0.0.exe) do (
            echo 📊 New size: %%~zF bytes ^(~%%~zF / 1048576 MB^)
        )
    )
)
:skip_upx

echo.
echo ╔════════════════════════════════════════════════════════╗
echo ║          Build Completed Successfully! 🎉              ║
echo ╚════════════════════════════════════════════════════════╝
echo.
echo 📦 Output: RAM_Manager_v1.0.0.exe
echo ⚠️  Remember: Run as Administrator!
echo.
echo Next steps:
echo   1. Test the executable
echo   2. Right-click → Run as Administrator
echo   3. Check all features work correctly
echo.

pause