import os
import re
import json
import importlib.util
import logging
import sys
from typing import Optional

logger = logging.getLogger(__name__)


class Skill:
    name: str = ""
    description: str = ""
    parameters: list[dict] = []

    def run(self, **params) -> str:
        raise NotImplementedError


class BuiltinSkill(Skill):
    """Wraps a Skill subclass as built-in."""


class OpenClawSkill(Skill):
    """Skill loaded from an OpenClaw-compatible directory (SKILL.md + scripts/)."""

    def __init__(self, skill_dir: str):
        self._skill_dir = skill_dir
        self.name = ""
        self.description = ""
        self.parameters = []
        self._func = None
        self._load()

    def _load(self):
        md_path = os.path.join(self._skill_dir, "SKILL.md")
        if not os.path.isfile(md_path):
            raise FileNotFoundError(f"Missing SKILL.md in {self._skill_dir}")
        with open(md_path, "r", encoding="utf-8") as f:
            content = f.read()
        meta, _ = self._parse_frontmatter(content)
        self.name = meta.get("name", os.path.basename(self._skill_dir))
        self.description = meta.get("description", "")
        self.parameters = meta.get("parameters", [])
        self._load_handler()

    @staticmethod
    def _parse_frontmatter(text: str):
        """Parse YAML frontmatter between --- markers. Returns (dict, body)."""
        text = text.strip()
        if not text.startswith("---"):
            return {}, text
        parts = text.split("---", 2)
        if len(parts) < 3:
            return {}, text
        yaml_block = parts[1].strip()
        body = parts[2].strip()
        meta = {}
        current_list_key = None
        for line in yaml_block.splitlines():
            line = line.strip()
            if not line:
                continue
            if line.startswith("- "):
                if current_list_key:
                    val = line[2:].strip().strip('"').strip("'")
                    meta.setdefault(current_list_key, []).append(val)
                continue
            if ":" in line:
                key, _, val = line.partition(":")
                key = key.strip()
                val = val.strip().strip('"').strip("'")
                if not val:
                    current_list_key = key
                    meta[key] = []
                else:
                    current_list_key = None
                    meta[key] = val
        return meta, body

    def _load_handler(self):
        scripts_dir = os.path.join(self._skill_dir, "scripts")
        if not os.path.isdir(scripts_dir):
            self._func = lambda **kw: f"技能 '{self.name}' 没有 handler"
            return
        candidates = ["main.py", "run.py"]
        found = None
        for c in candidates:
            p = os.path.join(scripts_dir, c)
            if os.path.isfile(p):
                found = p
                break
        if not found:
            self._func = lambda **kw: f"技能 '{self.name}' 没有 handler"
            return
        mod_name = f"_skill_{self.name}_{id(self)}"
        spec = importlib.util.spec_from_file_location(mod_name, found)
        if not spec or not spec.loader:
            self._func = lambda **kw: f"无法加载 {found}"
            return
        mod = importlib.util.module_from_spec(spec)
        sys.modules[mod_name] = mod
        try:
            spec.loader.exec_module(mod)
        except Exception as e:
            self._func = lambda **kw: f"加载 handler 失败: {e}"
            return
        if hasattr(mod, "run") and callable(mod.run):
            self._func = mod.run
        else:
            self._func = lambda **kw: f"未找到 run() 函数"

    def run(self, **params) -> str:
        return self._func(**params)


class SkillRegistry:
    def __init__(self):
        self._skills: dict[str, Skill] = {}
        self._enabled: set[str] = set()
        self._origins: dict[str, str] = {}

    def register(self, skill: Skill, origin: str = "builtin"):
        self._skills[skill.name] = skill
        self._enabled.add(skill.name)
        self._origins[skill.name] = origin

    def unregister(self, name: str):
        self._skills.pop(name, None)
        self._enabled.discard(name)
        self._origins.pop(name, None)

    def set_enabled(self, name: str, enabled: bool):
        if enabled:
            self._enabled.add(name)
        else:
            self._enabled.discard(name)

    def is_enabled(self, name: str) -> bool:
        return name in self._enabled

    def get_skill(self, name: str) -> Optional[Skill]:
        return self._skills.get(name)

    def get_enabled_skills(self) -> list[Skill]:
        return [s for n, s in self._skills.items() if n in self._enabled]

    def get_all_skills(self) -> list[Skill]:
        return list(self._skills.values())

    def get_origin(self, name: str) -> str:
        return self._origins.get(name, "builtin")

    def get_prompt_section(self) -> str:
        skills = self.get_enabled_skills()
        if not skills:
            return ""
        lines = [
            "",
            "### User-Defined Skills (自定义技能)",
            "调用方式: action type 为 `skill`，参数 `name` 为技能名，其余为技能参数",
            "",
            "| 技能 | 参数 | 说明 |",
            "|------|------|------|",
        ]
        for s in skills:
            params = ", ".join(f"`{p['name']}`" for p in s.parameters) if s.parameters else "无"
            lines.append(f"| `{s.name}` | {params} | {s.description} |")
        lines.append("")
        return "\n".join(lines)

    def execute(self, name: str, params: dict) -> tuple:
        skill = self._skills.get(name)
        if not skill:
            return False, f"Skill '{name}' not found"
        if name not in self._enabled:
            return False, f"Skill '{name}' is disabled"
        try:
            result = skill.run(**params)
            return True, result
        except Exception as e:
            logger.error(f"Skill '{name}' error: {e}")
            return False, f"Skill '{name}' error: {e}"

    def load_from_disk(self, skills_root: str):
        """Load all skill directories under skills_root (each dir = one OpenClaw skill)."""
        if not os.path.isdir(skills_root):
            return
        for entry in sorted(os.listdir(skills_root)):
            skill_dir = os.path.join(skills_root, entry)
            if not os.path.isdir(skill_dir):
                continue
            md_path = os.path.join(skill_dir, "SKILL.md")
            if not os.path.isfile(md_path):
                continue
            try:
                skill = OpenClawSkill(skill_dir)
                if skill.name:
                    self.register(skill, origin=skill_dir)
                    logger.info(f"Loaded skill '{skill.name}' from {skill_dir}")
            except Exception as e:
                logger.error(f"Failed to load skill from {skill_dir}: {e}")

    @staticmethod
    def create_skill_dir(skills_root: str, name: str) -> str:
        """Create a new skill directory with template files."""
        skill_dir = os.path.join(skills_root, name)
        os.makedirs(skill_dir, exist_ok=True)
        scripts_dir = os.path.join(skill_dir, "scripts")
        os.makedirs(scripts_dir, exist_ok=True)
        md_content = f"""---
name: {name}
description: My custom skill
parameters: []
---

Instructions for the AI about when to use this skill.
"""
        with open(os.path.join(skill_dir, "SKILL.md"), "w", encoding="utf-8") as f:
            f.write(md_content)
        handler_content = """def run(**params) -> str:
    # params contains the arguments defined in SKILL.md frontmatter
    return f"Executed with: {params}"
"""
        with open(os.path.join(scripts_dir, "main.py"), "w", encoding="utf-8") as f:
            f.write(handler_content)
        return skill_dir

    @staticmethod
    def delete_skill_dir(skill_dir: str):
        """Delete a skill directory and all its contents."""
        if os.path.isdir(skill_dir):
            import shutil
            shutil.rmtree(skill_dir)

    @staticmethod
    def read_skill_md(skill_dir: str) -> Optional[str]:
        md_path = os.path.join(skill_dir, "SKILL.md")
        if os.path.isfile(md_path):
            with open(md_path, "r", encoding="utf-8") as f:
                return f.read()
        return None

    @staticmethod
    def write_skill_md(skill_dir: str, content: str):
        md_path = os.path.join(skill_dir, "SKILL.md")
        with open(md_path, "w", encoding="utf-8") as f:
            f.write(content)

    @staticmethod
    def read_handler(skill_dir: str) -> Optional[str]:
        for c in ["main.py", "run.py"]:
            p = os.path.join(skill_dir, "scripts", c)
            if os.path.isfile(p):
                with open(p, "r", encoding="utf-8") as f:
                    return f.read()
        return None

    @staticmethod
    def write_handler(skill_dir: str, code: str):
        scripts_dir = os.path.join(skill_dir, "scripts")
        os.makedirs(scripts_dir, exist_ok=True)
        with open(os.path.join(scripts_dir, "main.py"), "w", encoding="utf-8") as f:
            f.write(code)


class BuiltinShellSkill(Skill):
    name = "run_shell"
    description = "执行系统 Shell 命令（PowerShell/cmd）并返回输出"
    parameters = [{"name": "command", "type": "string", "required": True, "description": "要执行的命令"}]

    def run(self, command: str = "") -> str:
        import subprocess
        result = subprocess.run(
            ["powershell", "-NoProfile", "-Command", command],
            capture_output=True, text=True, timeout=30,
        )
        output = result.stdout or result.stderr
        return output[:2000] if output else "(no output)"


class BuiltinOpenAppSkill(Skill):
    name = "open_app"
    description = "打开应用程序（按名称搜索并启动）"
    parameters = [{"name": "app_name", "type": "string", "required": True, "description": "应用程序名称"}]

    def run(self, app_name: str = "") -> str:
        import subprocess
        try:
            subprocess.Popen(["start", "", app_name], shell=True)
            return f"已启动 {app_name}"
        except Exception as e:
            return f"启动失败: {e}"


class BuiltinWriteFileSkill(Skill):
    name = "write_file"
    description = "写入内容到文件"
    parameters = [
        {"name": "path", "type": "string", "required": True, "description": "文件路径"},
        {"name": "content", "type": "string", "required": True, "description": "文件内容"},
    ]

    def run(self, path: str = "", content: str = "") -> str:
        try:
            os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
            with open(path, "w", encoding="utf-8") as f:
                f.write(content)
            return f"已写入 {path}"
        except Exception as e:
            return f"写入失败: {e}"


class BuiltinReadFileSkill(Skill):
    name = "read_file"
    description = "读取文件内容"
    parameters = [{"name": "path", "type": "string", "required": True, "description": "文件路径"}]

    def run(self, path: str = "") -> str:
        try:
            with open(path, "r", encoding="utf-8") as f:
                return f.read()[:2000]
        except Exception as e:
            return f"读取失败: {e}"


class BuiltinGetClipboardSkill(Skill):
    name = "get_clipboard"
    description = "获取剪贴板内容"
    parameters = []

    def run(self) -> str:
        try:
            import pyperclip
            return pyperclip.paste()
        except ImportError:
            try:
                import subprocess
                result = subprocess.run(["powershell", "-NoProfile", "Get-Clipboard"], capture_output=True, text=True)
                return result.stdout.strip() or "(empty)"
            except Exception as e:
                return f"获取剪贴板失败: {e}"


class BuiltinSetClipboardSkill(Skill):
    name = "set_clipboard"
    description = "设置剪贴板内容"
    parameters = [{"name": "text", "type": "string", "required": True, "description": "要设置的文本"}]

    def run(self, text: str = "") -> str:
        import subprocess
        try:
            import pyperclip
            pyperclip.copy(text)
            return "已复制到剪贴板"
        except ImportError:
            try:
                escaped = text.replace("'", "''")
                subprocess.run(["powershell", "-NoProfile", f"Set-Clipboard -Value '{escaped}'"], check=True)
                return "已复制到剪贴板"
            except Exception as e:
                return f"设置剪贴板失败: {e}"


BUILTIN_SKILLS = [
    BuiltinShellSkill(),
    BuiltinOpenAppSkill(),
    BuiltinWriteFileSkill(),
    BuiltinReadFileSkill(),
    BuiltinGetClipboardSkill(),
    BuiltinSetClipboardSkill(),
]


def create_registry(skills_root: str = "") -> SkillRegistry:
    registry = SkillRegistry()
    for skill in BUILTIN_SKILLS:
        registry.register(skill)
    if skills_root:
        registry.load_from_disk(skills_root)
    return registry
