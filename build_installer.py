import os
import io
import zipfile
import subprocess
import shutil

PROJECT_DIR = os.path.dirname(os.path.abspath(__file__))

# Read version from src/version.py
with open(os.path.join(PROJECT_DIR, "src", "version.py")) as f:
    exec(f.read())
VERSION = locals().get("VERSION", "1.5.0-beta.1")

OUTPUT = os.path.join(PROJECT_DIR, "EyeForge_Setup.exe")

EXCLUDE_DIRS = {"venv", ".git", "__pycache__", "logs", "skills"}
EXCLUDE_EXTS = {".pyc", ".log", ".old"}
EXCLUDE_FILES = {"config.json", "EyeForge_Setup.exe", "setup_stub.exe", ".gitignore"}
# Build artifacts not needed in installer
EXCLUDE_NAMES = {"build_installer.py", "setup_stub.cs", "start_launcher.cs", "setup_builder.cs"}


def build_payload_zip():
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        for root, dirs, names in os.walk(PROJECT_DIR):
            dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
            for name in names:
                if any(name.endswith(e) for e in EXCLUDE_EXTS):
                    continue
                if name in EXCLUDE_FILES:
                    continue
                if name in EXCLUDE_NAMES:
                    continue
                full = os.path.join(root, name)
                rel = os.path.relpath(full, PROJECT_DIR)
                z.write(full, rel)
    return buf.getvalue()


def build():
    stub_cs = os.path.join(PROJECT_DIR, "setup_stub.cs")
    stub_exe = os.path.join(PROJECT_DIR, "setup_stub.exe")

    print("[1/4] 编译安装程序核心...")
    csc = r"C:\Windows\Microsoft.NET\Framework\v4.0.30319\csc.exe"
    icon_path = os.path.join(PROJECT_DIR, "src", "logo.ico")
    cmd = [csc, "/target:winexe",
           "/reference:System.IO.Compression.dll",
           "/reference:System.IO.Compression.FileSystem.dll",
           f"/out:{stub_exe}", stub_cs]
    if os.path.exists(icon_path):
        cmd.append(f"/win32icon:{icon_path}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print("编译失败:", result.stderr)
        return

    print("[2/4] 打包项目文件...")
    zipped = build_payload_zip()
    print(f"       ZIP 大小: {len(zipped) / 1024:.1f} KB")

    print("[3/4] 合并安装程序...")
    with open(stub_exe, "rb") as f:
        stub_data = f.read()
    with open(OUTPUT, "wb") as f:
        f.write(stub_data)
        f.write(b"EYEFORGEZIP")
        f.write(zipped)

    os.remove(stub_exe)

    size = os.path.getsize(OUTPUT) / 1024
    print(f"[4/4] 完成: {OUTPUT} ({size:.1f} KB)")


if __name__ == "__main__":
    build()
