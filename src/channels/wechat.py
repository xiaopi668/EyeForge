import json
import os
import uuid
import time
import base64
import struct
import hashlib
import hmac
import logging
import threading
import queue
from typing import Optional, Callable
from urllib.request import Request, urlopen
from urllib.error import URLError

logger = logging.getLogger(__name__)

ILINK_BASE = "https://ilink.weixin.qq.com"
_running = False
_thread: Optional[threading.Thread] = None
_on_message: Optional[Callable] = None

_outgoing_queue: queue.Queue = queue.Queue()
_context_tokens: dict = {}
_cond = threading.Condition()

token: str = ""
uin: str = ""
cursor: str = ""


def _base_info():
    return {
        "app_id": "",
        "bot_id": "",
        "channel_version": "1.0.3",
        "env": "release",
        "platform": "weixin",
        "sdk_version": "1.0.0",
    }


def _headers():
    return {
        "Content-Type": "application/json",
        "AuthorizationType": "ilink_bot_token",
        "Authorization": f"Bearer {token}",
        "X-WECHAT-UIN": uin,
    }


def _call(endpoint: str, body: dict, timeout: int = 30) -> Optional[dict]:
    url = f"{ILINK_BASE}/ilink/bot/{endpoint}"
    data = json.dumps({**body, "base_info": _base_info()}, ensure_ascii=False).encode("utf-8")
    req = Request(url, data=data, headers=_headers(), method="POST")
    try:
        resp = urlopen(req, timeout=timeout)
        raw = resp.read().decode("utf-8").strip()
        return json.loads(raw) if raw else None
    except URLError as e:
        logger.warning(f"iLink API error ({endpoint}): {e}")
        return None
    except Exception as e:
        logger.warning(f"iLink API exception ({endpoint}): {e}")
        return None


def _poll_loop():
    global cursor
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

        msg = None
        with _cond:
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
                "client_id": str(uuid.uuid4()),
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


def start(bot_token: str = "", bot_uin: str = ""):
    global _running, _thread, token, uin
    if _running:
        return
    if not bot_token:
        logger.error("WeChat: bot_token is required")
        return
    token = bot_token
    uin = bot_uin or base64.b64encode(struct.pack("<I", 12345678)).decode()
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
