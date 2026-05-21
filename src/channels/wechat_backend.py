import json
import os
import uuid
import time
import hmac
import hashlib
import base64
import struct
import logging
import threading
import queue
from http.server import HTTPServer, BaseHTTPRequestHandler
from typing import Optional, Callable
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)

_server: Optional[HTTPServer] = None
_thread: Optional[threading.Thread] = None
_running = False
_on_wechat_message: Optional[Callable] = None

_outgoing_queue: queue.Queue = queue.Queue()
_context_tokens: dict = {}
_cond = threading.Condition()

_cdn_base = "https://cdn.example.com"
_auth_tokens: set = set()

SUPPORTED_UIN = base64.b64encode(struct.pack("<I", 12345678)).decode()


class _Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        logger.debug(f"wechat: {fmt % args}")

    def _json_response(self, code: int, data: dict):
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Connection", "keep-alive")
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False).encode("utf-8"))

    def _read_body(self) -> dict:
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length) if length > 0 else b"{}"
        try:
            return json.loads(raw)
        except json.JSONDecodeError:
            return {}

    def _verify_auth(self) -> bool:
        auth = self.headers.get("Authorization", "")
        token = auth.replace("Bearer ", "").strip()
        if not token:
            return False
        return token in _auth_tokens

    def _build_base_info(self):
        return {
            "app_id": "",
            "bot_id": "",
            "channel_version": "1.0.3",
            "env": "release",
            "platform": "weixin",
            "sdk_version": "1.0.0",
        }

    def do_POST(self):
        path = self.path.strip("/")

        if not self._verify_auth():
            self._json_response(401, {"ret": -1, "err_msg": "unauthorized"})
            return

        body = self._read_body()
        uin = self.headers.get("X-WECHAT-UIN", SUPPORTED_UIN)

        if path == "sendmessage":
            self._handle_sendmessage(body, uin)
        elif path == "getupdates":
            self._handle_getupdates(body, uin)
        elif path == "getuploadurl":
            self._handle_getuploadurl(body, uin)
        elif path == "getconfig":
            self._handle_getconfig(body, uin)
        elif path == "sendtyping":
            self._handle_sendtyping(body, uin)
        else:
            self._json_response(404, {"ret": -1, "err_msg": f"unknown endpoint: {path}"})

    def _handle_sendmessage(self, body: dict, uin: str):
        msg = body.get("msg", {})
        to_user_id = msg.get("to_user_id", "")
        context_token = msg.get("context_token", "")
        item_list = msg.get("item_list", [])

        text = ""
        for item in item_list:
            if item.get("type") == 1:
                text_item = item.get("text_item", {})
                text = text_item.get("text", "")

        if not text:
            self._json_response(200, {"ret": 0})
            return

        if context_token:
            _context_tokens[to_user_id] = context_token

        self._json_response(200, {"ret": 0})

        if _on_wechat_message:
            threading.Thread(
                target=_on_wechat_message,
                args=(to_user_id, text, context_token),
                daemon=True,
            ).start()

    def _handle_getupdates(self, body: dict, uin: str):
        cursor = body.get("get_updates_buf", "")
        timeout = 25

        msg = None
        with _cond:
            end_time = time.time() + timeout
            while time.time() < end_time:
                try:
                    remaining = max(0.1, end_time - time.time())
                    msg = _outgoing_queue.get(timeout=remaining)
                    break
                except queue.Empty:
                    continue

        if msg is None:
            self._json_response(200, {
                "ret": 0, "msgs": [], "get_updates_buf": cursor or "buf_0",
            })
            return

        to_user_id = msg.get("to_user_id", "")
        text = msg.get("text", "")
        context_token = _context_tokens.get(to_user_id, "")

        resp_msg = {
            "from_user_id": "",
            "to_user_id": to_user_id,
            "client_id": str(uuid.uuid4()),
            "message_type": 2,
            "message_state": 2,
            "item_list": [
                {
                    "type": 1,
                    "text_item": {"text": text},
                }
            ],
            "context_token": context_token,
            "seq": 1,
        }
        if context_token:
            resp_msg["context_token"] = context_token

        self._json_response(200, {
            "ret": 0,
            "msgs": [resp_msg],
            "get_updates_buf": f"buf_{int(time.time())}",
        })

    def _handle_getuploadurl(self, body: dict, uin: str):
        media_type = body.get("media_type", 1)
        filekey = body.get("filekey", f"eyeforge_{uuid.uuid4().hex[:16]}")

        self._json_response(200, {
            "ret": 0,
            "upload_param": {
                "url": f"{_cdn_base}/{filekey}",
                "method": "PUT",
                "headers": {"Content-Type": "application/octet-stream"},
                "form_data": {},
            },
            "thumb_upload_param": {
                "url": f"{_cdn_base}/thumb_{filekey}",
                "method": "PUT",
                "headers": {"Content-Type": "application/octet-stream"},
                "form_data": {},
            },
            "encrypt_query_param": base64.b64encode(b"eyeforge_cdn").decode(),
            "aes_key": base64.b64encode(os.urandom(16)).decode(),
        })

    def _handle_getconfig(self, body: dict, uin: str):
        self._json_response(200, {
            "ret": 0,
            "typing_ticket": base64.b64encode(os.urandom(16)).decode(),
            "typing_gap_ms": 5000,
        })

    def _handle_sendtyping(self, body: dict, uin: str):
        self._json_response(200, {"ret": 0})


def add_token(token: str):
    _auth_tokens.add(token)


def queue_outgoing(to_user_id: str, text: str):
    with _cond:
        _outgoing_queue.put({
            "to_user_id": to_user_id,
            "text": text,
            "timestamp": time.time(),
        })
        _cond.notify_all()


def set_on_message(callback: Callable[[str, str, str], None]):
    global _on_wechat_message
    _on_wechat_message = callback


def _run(host: str, port: int):
    global _server
    _server = HTTPServer((host, port), _Handler)
    _server.serve_forever()


def start(host: str = "0.0.0.0", port: int = 8800, token: str = ""):
    global _running, _thread
    if _running:
        return
    _running = True
    if token:
        add_token(token)
    _thread = threading.Thread(target=_run, args=(host, port), daemon=True)
    _thread.start()
    logger.info(f"WeChat backend server started on http://{host}:{port}")


def stop():
    global _running, _server, _thread
    _running = False
    if _server:
        _server.shutdown()
        _server.server_close()
        _server = None
    if _thread:
        _thread.join(timeout=3)
        _thread = None
    logger.info("WeChat backend server stopped")


def is_running() -> bool:
    return _running
