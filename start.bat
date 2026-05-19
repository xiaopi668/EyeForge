@echo off
chcp 65001 >nul
title EyeForge

if not exist "venv\" (
    echo [错误] 未找到虚拟环境，请先运行 install.bat
    pause
    exit /b 1
)

if not exist "venv\Scripts\pythonw.exe" (
    echo [错误] 虚拟环境已损坏，请删除 venv 文件夹后重新运行 install.bat
    pause
    exit /b 1
)

start "" /b venv\Scripts\pythonw.exe main.py
exit
