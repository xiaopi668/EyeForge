import os
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


class UserSkillFunc(Skill):
    """Wrapper for function-based user skills (simple format)."""

    def __init__(self, name: str, description: str, parameters: list, func):
        self.name = name
        self.description = description
        self.parameters = parameters
        self._func = func

    def run(self, **params) -> str:
        return self._func(**params)


class UserSkillClass(Skill):
    """Thin wrapper so any Skill subclass is usable directly."""


class SkillRegistry:
    def __init__(self):
        self._skills: dict[str, Skill] = {}
        self._enabled: set[str] = set()
        self._origins: dict[str, str] = {}  # skill_name -> "builtin" or filepath

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

    def load_from_disk(self, skills_dir: str):
        """Load all .py skill files from a directory."""
        if not os.path.isdir(skills_dir):
            return
        sys.path.insert(0, os.path.dirname(skills_dir))
        try:
            for fname in sorted(os.listdir(skills_dir)):
                if not fname.endswith(".py") or fname == "__init__.py":
                    continue
                fpath = os.path.join(skills_dir, fname)
                try:
                    self._load_skill_file(fpath, fname[:-3])
                except Exception as e:
                    logger.error(f"Failed to load skill {fname}: {e}")
        finally:
            if sys.path and sys.path[0] == os.path.dirname(skills_dir):
                sys.path.pop(0)

    def _load_skill_file(self, fpath: str, mod_name: str):
        spec = importlib.util.spec_from_file_location(mod_name, fpath)
        if not spec or not spec.loader:
            return
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)

        # Class-based: find Skill subclasses
        loaded = False
        for attr_name in dir(mod):
            attr = getattr(mod, attr_name)
            if isinstance(attr, type) and issubclass(attr, Skill) and attr is not Skill and attr is not UserSkillClass:
                inst = attr()
                if inst.name:
                    self.register(inst, origin=fpath)
                    loaded = True

        # Function-based: SKILL_NAME, SKILL_DESCRIPTION, SKILL_PARAMETERS + run()
        if not loaded and hasattr(mod, "SKILL_NAME") and hasattr(mod, "run"):
            name = mod.SKILL_NAME
            desc = getattr(mod, "SKILL_DESCRIPTION", "")
            params = getattr(mod, "SKILL_PARAMETERS", [])
            func = mod.run
            skill = UserSkillFunc(name, desc, params, func)
            self.register(skill, origin=fpath)
            loaded = True

    @staticmethod
    def save_user_skill(skills_dir: str, name: str, code: str):
        """Save a user skill .py file to disk."""
        os.makedirs(skills_dir, exist_ok=True)
        fpath = os.path.join(skills_dir, f"{name}.py")
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(code)
        return fpath

    @staticmethod
    def delete_user_skill(skills_dir: str, name: str):
        fpath = os.path.join(skills_dir, f"{name}.py")
        if os.path.exists(fpath):
            os.remove(fpath)

    @staticmethod
    def get_skill_code(skills_dir: str, name: str) -> Optional[str]:
        fpath = os.path.join(skills_dir, f"{name}.py")
        if os.path.exists(fpath):
            with open(fpath, "r", encoding="utf-8") as f:
                return f.read()
        return None


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


def create_registry(skills_dir: str = "") -> SkillRegistry:
    registry = SkillRegistry()
    for skill in BUILTIN_SKILLS:
        registry.register(skill)
    if skills_dir:
        registry.load_from_disk(skills_dir)
    return registry
