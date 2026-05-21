import json
import logging
import threading
import asyncio
from typing import Callable, Optional

logger = logging.getLogger(__name__)

_loop: Optional[asyncio.AbstractEventLoop] = None
_thread: Optional[threading.Thread] = None
_server = None
_running = False
_on_message: Optional[Callable] = None


async def _handle(websocket, token: str):
    global _on_message
    try:
        async for raw in websocket:
            try:
                data = json.loads(raw)
            except json.JSONDecodeError:
                await websocket.send(json.dumps({"type": "error", "message": "invalid JSON"}))
                continue

            msg_type = data.get("type", "")

            if msg_type == "auth":
                ok = data.get("token", "") == token
                await websocket.send(json.dumps({
                    "type": "auth_result", "success": ok,
                    "message": "authenticated" if ok else "invalid token",
                }))
                if not ok:
                    await websocket.close()
                    return
                continue

            if msg_type == "task":
                task_text = data.get("task", "").strip()
                if not task_text:
                    await websocket.send(json.dumps({"type": "error", "message": "task is empty"}))
                    continue
                await websocket.send(json.dumps({"type": "status", "message": "task received"}))

                if _on_message:
                    try:
                        result = _on_message(task_text)
                        await websocket.send(json.dumps({
                            "type": "result",
                            "status": result.get("status", "error"),
                            "message": result.get("message", ""),
                            "data": result.get("data"),
                        }))
                    except Exception as e:
                        await websocket.send(json.dumps({
                            "type": "result", "status": "error", "message": str(e),
                        }))
                continue

            await websocket.send(json.dumps({"type": "error", "message": f"unknown type: {msg_type}"}))
    except Exception as e:
        logger.debug(f"WebSocket client disconnected: {e}")


async def _serve(host: str, port: int, token: str):
    global _server
    try:
        import websockets
        _server = await websockets.serve(
            lambda ws: _handle(ws, token),
            host, port,
            ping_interval=30,
        )
        logger.info(f"WebSocket server listening on ws://{host}:{port}")
        await _server.wait_closed()
    except Exception as e:
        logger.warning(f"WebSocket server error: {e}")


def _run_loop(host: str, port: int, token: str):
    global _loop
    _loop = asyncio.new_event_loop()
    asyncio.set_event_loop(_loop)
    try:
        _loop.run_until_complete(_serve(host, port, token))
    except Exception as e:
        logger.warning(f"WebSocket loop ended: {e}")


def start(host: str = "0.0.0.0", port: int = 8765, token: str = "",
          on_message: Callable[[str], dict] = None):
    global _running, _thread, _on_message
    if _running:
        return
    _on_message = on_message
    _running = True
    _thread = threading.Thread(
        target=_run_loop, args=(host, port, token), daemon=True
    )
    _thread.start()
    logger.info(f"WebSocket server started on ws://{host}:{port}")


def stop():
    global _running, _server, _loop, _thread
    _running = False
    if _server:
        _server.close()
        _server = None
    if _loop:
        _loop.call_soon_threadsafe(_loop.stop)
        _loop = None
    if _thread:
        _thread.join(timeout=3)
        _thread = None
    logger.info("WebSocket server stopped")


def is_available() -> bool:
    try:
        import websockets
        return True
    except ImportError:
        return False


def is_running() -> bool:
    return _running
