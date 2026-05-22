import json
import os
import uuid
import time
import base64
import struct
import random
import logging
import threading
import queue
from typing import Optional, Callable
from urllib.request import Request, urlopen
from urllib.error import URLError

logger = logging.getLogger(__name__)

ILINK_BASE = "https://ilinkai.weixin.qq.com"
WX_BASE = "https://weknora.weixin.qq.com"
_running = False
_thread: Optional[threading.Thread] = None
_on_message: Optional[Callable] = None

_outgoing_queue: queue.Queue = queue.Queue()
_context_tokens: dict = {}
_cond = threading.Condition()

token: str = ""
cache_dir: str = ""
cursor: str = ""


def _headers():
    uin = base64.b64encode(struct.pack("<I", random.randint(0, 0xFFFFFFFF))).decode()
    return {
        "Content-Type": "application/json",
        "AuthorizationType": "ilink_bot_token",
        "Authorization": f"Bearer {token}",
        "X-WECHAT-UIN": uin,
    }


def _call(endpoint: str, body: dict, timeout: int = 30) -> Optional[dict]:
    url = f"{ILINK_BASE}/ilink/bot/{endpoint}"
    body = {**body, "base_info": {"channel_version": "1.0.3"}}
    data = json.dumps(body, ensure_ascii=False).encode("utf-8")
    headers = _headers()
    headers["Content-Length"] = str(len(data))
    req = Request(url, data=data, headers=headers, method="POST")
    try:
        resp = urlopen(req, timeout=timeout)
        raw = resp.read().decode("utf-8").strip()
        return json.loads(raw) if raw else {"ret": 0}
    except URLError as e:
        logger.warning(f"iLink API error ({endpoint}): {e}")
        return None
    except Exception as e:
        logger.warning(f"iLink API exception ({endpoint}): {e}")
        return None


def qr_login(progress_callback: Optional[Callable[[str], None]] = None) -> dict:
    """Native QR code login via WeChat clawbot API. Returns {token, bot_id, user_id, base_url} or raises."""
    if progress_callback:
        progress_callback("获取二维码...")

    req = Request(
        f"{WX_BASE}/api/v1/wechat/qrcode",
        data=b"{}",
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    resp = urlopen(req, timeout=15)
    result = json.loads(resp.read().decode("utf-8"))
    if not result.get("success"):
        raise Exception(f"获取二维码失败: {result}")

    qrcode_url = result["data"]["qrcode_url"]
    qrcode_token = result["data"]["qrcode"]

    if progress_callback:
        progress_callback("qrcode_ready:" + qrcode_url)

    while True:
        try:
            status_data = json.dumps({"qrcode": qrcode_token}).encode("utf-8")
            status_req = Request(
                f"{WX_BASE}/api/v1/wechat/qrcode/status",
                data=status_data,
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            status_resp = urlopen(status_req, timeout=40)
            status_result = json.loads(status_resp.read().decode("utf-8"))

            if not status_result.get("success"):
                if progress_callback:
                    progress_callback("轮询出错，重试...")
                time.sleep(2)
                continue

            s = status_result["data"]["status"]
            if s == "wait":
                if progress_callback:
                    progress_callback("等待扫描...")
                time.sleep(1.5)
            elif s == "scaned":
                if progress_callback:
                    progress_callback("已扫码，请在手机上确认...")
                time.sleep(1)
            elif s == "confirmed":
                creds = status_result["data"]["credentials"]
                if progress_callback:
                    progress_callback("登录成功！")
                return {
                    "token": creds["bot_token"],
                    "bot_id": creds.get("ilink_bot_id", ""),
                    "user_id": creds.get("ilink_user_id", ""),
                    "base_url": status_result["data"].get("baseurl", ILINK_BASE),
                }
            elif s == "expired":
                raise Exception("二维码已过期，请重新获取")
            else:
                time.sleep(1.5)
        except URLError:
            time.sleep(2)
            continue


def _poll_loop():
    global cursor, _context_tokens
    state_path = os.path.join(cache_dir, "wechat_ctx.json") if cache_dir else ""
    if state_path and os.path.exists(state_path):
        try:
            with open(state_path) as f:
                _context_tokens = json.load(f)
        except Exception:
            pass

    while _running:
        result = _call("getupdates", {"get_updates_buf": cursor}, timeout=35)
        if not result or result.get("ret") != 0:
            time.sleep(2)
            continue

        cursor = result.get("get_updates_buf", cursor)
        msgs = result.get("msgs", [])
        for msg in msgs:
            from_user = msg.get("from_user_id", "")
            ctx = msg.get("context_token", "")
            if ctx:
                _context_tokens[from_user] = ctx
            for item in msg.get("item_list", []):
                if item.get("type") == 1:
                    text = item.get("text_item", {}).get("text", "")
                    if text and _on_message:
                        _on_message(from_user, text, ctx)

        if state_path and _context_tokens:
            try:
                with open(state_path, "w") as f:
                    json.dump(_context_tokens, f)
            except Exception:
                pass

        try:
            msg = _outgoing_queue.get(timeout=1)
        except queue.Empty:
            continue

        if msg is None:
            continue

        to_user_id = msg["to_user_id"]
        text = msg["text"]
        ctx = _context_tokens.get(to_user_id, "")

        send_body = {
            "msg": {
                "from_user_id": "",
                "to_user_id": to_user_id,
                "client_id": f"ef-{uuid.uuid4().hex[:12]}",
                "message_type": 2,
                "message_state": 2,
                "item_list": [{"type": 1, "text_item": {"text": text}}],
            }
        }
        if ctx:
            send_body["msg"]["context_token"] = ctx

        _call("sendmessage", send_body)


def queue_outgoing(to_user_id: str, text: str):
    with _cond:
        _outgoing_queue.put({"to_user_id": to_user_id, "text": text})
        _cond.notify_all()


def set_on_message(callback: Callable[[str, str, str], None]):
    global _on_message
    _on_message = callback


def start(bot_token: str = "", log_dir: str = ""):
    global _running, _thread, token, cursor, cache_dir
    if _running:
        return
    if not bot_token:
        logger.error("WeChat: bot_token is required")
        return
    token = bot_token
    cache_dir = log_dir or os.path.join(os.path.dirname(__file__), "..", "..", "logs")
    cursor = ""
    _running = True
    _thread = threading.Thread(target=_poll_loop, daemon=True)
    _thread.start()
    logger.info("WeChat iLink client started")


def stop():
    global _running, _thread
    _running = False
    if _thread:
        _thread.join(timeout=3)
        _thread = None
    logger.info("WeChat iLink client stopped")


def is_running() -> bool:
    return _running
