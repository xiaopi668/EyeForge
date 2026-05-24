@echo off
chcp 65001 >nul
title EyeForge Rust Build

echo ========================================
echo   EyeForge Rust Build
echo ========================================
echo.

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust toolchain not found.
    echo Install Rust from https://rustup.rs/
    echo.
    pause
    exit /b 1
)

echo [1/2] Checking Rust toolchain...
cargo --version
if %errorlevel% neq 0 (
    echo [ERROR] Failed to run cargo.
    pause
    exit /b 1
)

echo [2/2] Building EyeForge...
pushd src-rs
cargo build
if %errorlevel% neq 0 (
    popd
    echo [ERROR] Rust build failed.
    pause
    exit /b 1
)
popd

echo.
echo ===== Build Complete =====
echo Run start.bat to launch EyeForge.
pause
