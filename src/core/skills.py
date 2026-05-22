import os
import json
import importlib.util
import logging
from typing import Optional

logger = logging.getLogger(__name__)


class Skill:
    name: str = ""
    description: str = ""
    parameters: list[dict] = []

    def run(self, **params) -> str:
        raise NotImplementedError


class SkillRegistry:
    def __init__(self):
        self._skills: dict[str, Skill] = {}
        self._enabled: set[str] = set()

    def register(self, skill: Skill):
        self._skills[skill.name] = skill
        self._enabled.add(skill.name)

    def unregister(self, name: str):
        self._skills.pop(name, None)
        self._enabled.discard(name)

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


def create_registry() -> SkillRegistry:
    registry = SkillRegistry()
    for skill in BUILTIN_SKILLS:
        registry.register(skill)
    return registry
