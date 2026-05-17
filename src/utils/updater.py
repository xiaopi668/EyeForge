import re
import logging
import requests

from src.version import VERSION

logger = logging.getLogger(__name__)

GITHUB_RELEASES = "https://github.com/xiaopi668/EyeForge/releases"
GITHUB_LATEST = "https://github.com/xiaopi668/EyeForge/releases/latest"
GITCODE_RELEASES = "https://gitcode.com/xiaopi668/EyeForge/releases"
GITCODE_LATEST = "https://gitcode.com/xiaopi668/EyeForge/releases/latest"


def parse_version(tag: str) -> tuple:
    cleaned = tag.lstrip("vV")
    parts = re.findall(r"\d+", cleaned)
    return tuple(int(p) for p in parts[:3]) if parts else (0, 0, 0)


def _check_url(url: str, pattern: str) -> str:
    resp = requests.get(url, timeout=10, allow_redirects=True)
    resp.raise_for_status()
    m = re.search(pattern, resp.url)
    return m.group(1) if m else ""


def check_update() -> dict:
    latest = ""
    source = ""
    try:
        latest = _check_url(GITHUB_LATEST, r'/releases/tag/v?([^"/]+)')
        if latest:
            source = "GitHub"
    except Exception as e:
        logger.warning(f"GitHub check failed: {e}")

    if not latest:
        try:
            latest = _check_url(GITCODE_LATEST, r'/releases/tag/v?([^"/]+)')
            if latest:
                source = "GitCode"
        except Exception as e:
            logger.warning(f"GitCode check failed: {e}")

    latest_ver = parse_version(latest) if latest else (0, 0, 0)
    current_ver = parse_version(VERSION)

    return {
        "current": VERSION,
        "latest": latest or "Unknown",
        "update_available": latest_ver > current_ver if latest else False,
        "source": source or None,
        "github_url": GITHUB_RELEASES,
        "gitcode_url": GITCODE_RELEASES,
    }
