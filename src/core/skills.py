import os
import re
import subprocess
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


class OpenClawSkill(Skill):
    """Skill loaded from an OpenClaw-compatible directory.

    Structure:
      skills/<name>/
        SKILL.md         # Required: YAML frontmatter + instructions
        reference.md     # Optional: detailed reference docs
        README.md        # Optional: skill overview
        examples/        # Optional: example files
        scripts/         # Optional: executable scripts
    """

    def __init__(self, skill_dir: str):
        self._skill_dir = skill_dir
        self.name = ""
        self.description = ""
        self.parameters = []
        self.instructions = ""
        self.reference = ""
        self.examples: list[str] = []
        self.scripts: list[str] = []
        self._load()

    def _load(self):
        md_path = os.path.join(self._skill_dir, "SKILL.md")
        if not os.path.isfile(md_path):
            raise FileNotFoundError(f"Missing SKILL.md in {self._skill_dir}")
        with open(md_path, "r", encoding="utf-8") as f:
            content = f.read()
        meta, body = self._parse_frontmatter(content)
        self.name = meta.get("name", os.path.basename(self._skill_dir))
        self.description = meta.get("description", "")
        self.parameters = meta.get("parameters", [])
        self.instructions = body

        ref_path = os.path.join(self._skill_dir, "reference.md")
        if os.path.isfile(ref_path):
            with open(ref_path, "r", encoding="utf-8") as f:
                self.reference = f.read()

        examples_dir = os.path.join(self._skill_dir, "examples")
        if os.path.isdir(examples_dir):
            for fname in sorted(os.listdir(examples_dir)):
                fpath = os.path.join(examples_dir, fname)
                if os.path.isfile(fpath) and fname.endswith((".md", ".txt", ".json", ".yaml", ".yml")):
                    try:
                        with open(fpath, "r", encoding="utf-8") as f:
                            self.examples.append(f.read())
                    except Exception:
                        pass

        scripts_dir = os.path.join(self._skill_dir, "scripts")
        if os.path.isdir(scripts_dir):
            for fname in sorted(os.listdir(scripts_dir)):
                if fname.endswith(".py") or fname.endswith(".ps1") or fname.endswith(".bat") or fname.endswith(".sh"):
                    self.scripts.append(os.path.join(scripts_dir, fname))

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

    def get_instructions_block(self) -> str:
        """Returns formatted instructions for system prompt injection."""
        blocks = [f"## 技能: {self.name}", f"{self.description}\n", self.instructions]
        if self.reference:
            blocks.append(f"### 参考文档\n{self.reference}")
        if self.examples:
            blocks.append("### 示例")
            for i, ex in enumerate(self.examples):
                blocks.append(f"--- 示例 {i+1} ---\n{ex}")
        if self.scripts:
            blocks.append("### 可用脚本")
            for s in self.scripts:
                blocks.append(f"- `{os.path.basename(s)}`")
            blocks.append("调用方式: `{\"type\": \"skill\", \"name\": \"<技能名>\", \"script\": \"<脚本名>\", ...参数}`")
        return "\n\n".join(blocks)

    def run(self, **params) -> str:
        script_name = params.pop("script", None)
        if script_name:
            script_path = os.path.join(self._skill_dir, "scripts", script_name)
            if not os.path.isfile(script_path):
                return f"脚本 '{script_name}' 不存在"
            return self._run_script(script_path, params)
        return self.instructions[:500]

    def _run_script(self, script_path: str, params: dict) -> str:
        ext = os.path.splitext(script_path)[1].lower()
        try:
            if ext == ".py":
                return self._run_python(script_path, params)
            elif ext == ".ps1":
                return self._run_powershell(script_path, params)
            elif ext in (".bat", ".cmd", ".sh"):
                return self._run_shell(script_path, params)
            else:
                return f"不支持的脚本类型: {ext}"
        except subprocess.TimeoutExpired:
            return "脚本执行超时"
        except Exception as e:
            logger.error(f"Script error: {e}")
            return f"脚本执行失败: {e}"

    def _run_python(self, path: str, params: dict) -> str:
        mod_name = f"_skill_script_{os.path.basename(path)}_{id(self)}"
        spec = importlib.util.spec_from_file_location(mod_name, path)
        if not spec or not spec.loader:
            return f"无法加载脚本 {path}"
        mod = importlib.util.module_from_spec(spec)
        sys.modules[mod_name] = mod
        spec.loader.exec_module(mod)
        if hasattr(mod, "run"):
            return mod.run(**params)
        result = subprocess.run(
            [sys.executable, path] + [f"--{k}={v}" for k, v in params.items()],
            capture_output=True, text=True, timeout=60,
        )
        return (result.stdout or result.stderr)[:3000]

    def _run_powershell(self, path: str, params: dict) -> str:
        args = [f"-{k} '{v}'" for k, v in params.items()]
        cmd = ["powershell", "-NoProfile", "-File", path] + args
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        return (result.stdout or result.stderr)[:3000]

    def _run_shell(self, path: str, params: dict) -> str:
        result = subprocess.run([path], capture_output=True, text=True, timeout=60)
        return (result.stdout or result.stderr)[:3000]


class BuiltinSkill(Skill):
    """Base for built-in class-based skills."""


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
        """Returns skill instructions for system prompt injection."""
        skills = self.get_enabled_skills()
        if not skills:
            return ""
        blocks = ["## User-Defined Skills (自定义技能)"]
        for s in skills:
            if isinstance(s, OpenClawSkill):
                blocks.append(s.get_instructions_block())
            else:
                params = ", ".join(f"`{p['name']}`" for p in s.parameters) if s.parameters else "无"
                blocks.append(
                    f"### {s.name}\n{s.description}\n参数: {params}\n"
                    f"调用: {{\"type\": \"skill\", \"name\": \"{s.name}\", ...参数}}"
                )
        return "\n\n".join(blocks)

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
        """Load all skill directories under skills_root."""
        if not os.path.isdir(skills_root):
            return
        for entry in sorted(os.listdir(skills_root)):
            skill_dir = os.path.join(skills_root, entry)
            if not os.path.isdir(skill_dir):
                continue
            if not os.path.isfile(os.path.join(skill_dir, "SKILL.md")):
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
        os.makedirs(os.path.join(skill_dir, "scripts"), exist_ok=True)
        os.makedirs(os.path.join(skill_dir, "examples"), exist_ok=True)
        md = f"""---
name: {name}
description: My custom skill
parameters: []
---

Instructions for the AI about when and how to use this skill.
"""
        with open(os.path.join(skill_dir, "SKILL.md"), "w", encoding="utf-8") as f:
            f.write(md)
        with open(os.path.join(skill_dir, "reference.md"), "w", encoding="utf-8") as f:
            f.write("# Reference\n\nDetailed reference documentation for this skill.\n")
        handler = """def run(**params) -> str:
    return f"Executed with: {params}"
"""
        with open(os.path.join(skill_dir, "scripts", "main.py"), "w", encoding="utf-8") as f:
            f.write(handler)
        return skill_dir

    @staticmethod
    def delete_skill_dir(skill_dir: str):
        import shutil
        if os.path.isdir(skill_dir):
            shutil.rmtree(skill_dir)

    @staticmethod
    def read_file(path: str) -> Optional[str]:
        if os.path.isfile(path):
            with open(path, "r", encoding="utf-8") as f:
                return f.read()
        return None

    @staticmethod
    def write_file(path: str, content: str):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)

    @staticmethod
    def list_skill_files(skill_dir: str) -> list[str]:
        """List all files in a skill directory."""
        result = []
        for root, dirs, files in os.walk(skill_dir):
            for fname in files:
                rel = os.path.relpath(os.path.join(root, fname), skill_dir)
                result.append(rel)
        return sorted(result)


class BuiltinShellSkill(BuiltinSkill):
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


class BuiltinOpenAppSkill(BuiltinSkill):
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


class BuiltinWriteFileSkill(BuiltinSkill):
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


class BuiltinReadFileSkill(BuiltinSkill):
    name = "read_file"
    description = "读取文件内容"
    parameters = [{"name": "path", "type": "string", "required": True, "description": "文件路径"}]

    def run(self, path: str = "") -> str:
        try:
            with open(path, "r", encoding="utf-8") as f:
                return f.read()[:2000]
        except Exception as e:
            return f"读取失败: {e}"


class BuiltinGetClipboardSkill(BuiltinSkill):
    name = "get_clipboard"
    description = "获取剪贴板内容"
    parameters = []

    def run(self) -> str:
        try:
            import pyperclip
            return pyperclip.paste()
        except ImportError:
            try:
                result = subprocess.run(["powershell", "-NoProfile", "Get-Clipboard"], capture_output=True, text=True)
                return result.stdout.strip() or "(empty)"
            except Exception as e:
                return f"获取剪贴板失败: {e}"


class BuiltinSetClipboardSkill(BuiltinSkill):
    name = "set_clipboard"
    description = "设置剪贴板内容"
    parameters = [{"name": "text", "type": "string", "required": True, "description": "要设置的文本"}]

    def run(self, text: str = "") -> str:
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
