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
    """Native QR code login. Returns {token, bot_id, user_id} or raises."""
    if progress_callback:
        progress_callback("获取二维码...")

    resp = urlopen(f"{ILINK_BASE}/ilink/bot/get_bot_qrcode?bot_type=3", timeout=15)
    data = json.loads(resp.read().decode("utf-8"))
    qrcode_key = data["qrcode"]
    qrcode_url = data["qrcode_img_content"]

    if progress_callback:
        progress_callback("qrcode_ready:" + qrcode_url)

    while True:
        try:
            status_resp = urlopen(
                Request(
                    f"{ILINK_BASE}/ilink/bot/get_qrcode_status?qrcode={qrcode_key}",
                    headers={"iLink-App-ClientVersion": "1"},
                ),
                timeout=40,
            )
            status = json.loads(status_resp.read().decode("utf-8"))
            s = status.get("status", "")
            if s == "scaned":
                if progress_callback:
                    progress_callback("已扫码，请在手机上确认...")
            elif s == "confirmed":
                if progress_callback:
                    progress_callback("登录成功！")
                return {
                    "token": status["bot_token"],
                    "bot_id": status.get("ilink_bot_id", ""),
                    "user_id": status.get("ilink_user_id", ""),
                }
            elif s == "expired":
                raise Exception("二维码已过期，请重新获取")
            else:
                time.sleep(1)
        except URLError:
            time.sleep(1)
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
