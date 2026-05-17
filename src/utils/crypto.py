import os
import base64
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC


_KEY = None


def _get_key() -> bytes:
    global _KEY
    if _KEY is not None:
        return _KEY
    salt = b"EyeForge_salt_2026"
    kdf = PBKDF2HMAC(algorithm=hashes.SHA256(), length=32, salt=salt, iterations=100000)
    _KEY = base64.urlsafe_b64encode(kdf.derive(b"EyeForge_secret"))
    return _KEY


def encrypt(plain: str) -> str:
    if not plain:
        return ""
    f = Fernet(_get_key())
    return f.encrypt(plain.encode("utf-8")).decode("utf-8")


def decrypt(token: str) -> str:
    if not token:
        return ""
    try:
        f = Fernet(_get_key())
        return f.decrypt(token.encode("utf-8")).decode("utf-8")
    except Exception:
        return token
