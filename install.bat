@echo off
chcp 65001 >nul
title EyeForge 环境安装

echo ========================================
echo   EyeForge 一键环境安装
echo ========================================
echo.

where python >nul 2>nul
if %errorlevel% neq 0 (
    echo [错误] 未检测到 Python
    echo.
    echo 请先安装 Python 3.10+：
    echo   官网 https://www.python.org/downloads/
    echo   安装时务必勾选 [Add Python to PATH]
    echo.
    pause
    exit /b 1
)

python --version | find "3." >nul
if %errorlevel% neq 0 (
    echo [错误] Python 版本过低，需要 Python 3.10+
    pause
    exit /b 1
)

if not exist "venv\" (
    echo [1/3] 创建虚拟环境...
    python -m venv venv
    if %errorlevel% neq 0 (
        echo [错误] 虚拟环境创建失败
        pause
        exit /b 1
    )
) else (
    echo [1/3] 虚拟环境已存在，跳过
)

echo [2/3] 升级 pip...
call venv\Scripts\python.exe -m pip install --upgrade pip -q

echo [3/3] 安装依赖...
call venv\Scripts\pip.exe install -r requirements.txt

if %errorlevel% equ 0 (
    echo.
    echo ===== 安装完成 =====
    echo 运行 start.bat 启动程序
) else (
    echo.
    echo [警告] 部分依赖安装失败，请查看上方错误信息
)

pause
