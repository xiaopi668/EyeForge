@echo off
chcp 65001 >nul
title EyeForge Rust Launcher

if exist "src-rs\target\debug\eye-forge-rs.exe" (
    start "" "src-rs\target\debug\eye-forge-rs.exe"
) else (
    echo [INFO] Rust binary not found. Building first...
    call install.bat
    if exist "src-rs\target\debug\eye-forge-rs.exe" (
        start "" "src-rs\target\debug\eye-forge-rs.exe"
    ) else (
        echo [ERROR] Failed to launch EyeForge.
        pause
        exit /b 1
    )
)
