"""Sabitler, URL'ler, renkler ve kullanici kayit fonksiyonlari."""

from __future__ import annotations

import base64
import hashlib
import json
import socket
import sys
import uuid
from pathlib import Path

try:
    from cryptography.fernet import Fernet, InvalidToken
    _KRIPTO = True
except ImportError:
    _KRIPTO = False


def _asset_yolu(dosya: str) -> Path:
    """Dev, PyInstaller ve Nuitka standalone ortamlarinda asset dosyasini bulur."""
    # PyInstaller --onefile: gecici klasor
    if hasattr(sys, "_MEIPASS"):
        p = Path(sys._MEIPASS) / dosya  # type: ignore[attr-defined]
        if p.exists():
            return p
    # PyInstaller --onedir / Nuitka standalone: exe yaninda
    exe_dir = Path(sys.executable).resolve().parent
    p = exe_dir / dosya
    if p.exists():
        return p
    # Dev: kaynak dizin
    return Path(__file__).resolve().parent / dosya

# Versiyon - Tek kaynak
__version__ = "1.0.0"

# URL'ler
GIRIS_URL = "https://wifi.gsb.gov.tr/j_spring_security_check"
CIKIS_URL = "https://wifi.gsb.gov.tr/cikisSon.html?logout=1"

# Dosya yollari
AYAR_DOSYASI = Path(__file__).with_name("user_config.json")
WIFI_IMG_YOLU = _asset_yolu("wifi.png")

# Ag ayarlari
TIMEOUT = 15
MAX_DENEME = 3
BACKOFF_TABANI = 2
BACKOFF_CARPAN = 3

# User-Agent
USER_AGENT = f"GSB-WiFi-AutoLogin/{__version__}"

# Renkler
renkler = {
    "arkaplan": "#13111c",
    "kart": "#1a1825",
    "kart2": "#221f2e",
    "input_bg": "#0d0b14",
    "mor": "#7c3aed",
    "mor_hover": "#8b5cf6",
    "mor_acik": "#a78bfa",
    "yesil": "#10b981",
    "yesil_koyu": "#064e3b",
    "sari": "#f59e0b",
    "sari_koyu": "#78350f",
    "kirmizi": "#ef4444",
    "kirmizi_koyu": "#7f1d1d",
    "beyaz": "#f8fafc",
    "gri": "#a1a1aa",
    "gri_koyu": "#71717a",
    "cizgi": "#27272a",
    "focus": "#7c3aed",
}


def _sifrele_anahtar() -> bytes:
    """MAC + hostname ile makineye ozgu anahtar turet; baska makinede sifre okunamaz"""
    mac = str(uuid.getnode())
    hostname = socket.gethostname()
    girdi = f"gsb-wifi-autologin:{mac}:{hostname}".encode()
    # 100k iterasyon: brute-force maliyetini artirmak icin standart deger
    anahtar_bytes = hashlib.pbkdf2_hmac("sha256", girdi, b"gsb-salt-v1", 100_000, dklen=32)
    return base64.urlsafe_b64encode(anahtar_bytes)


def _sifrele(metin: str) -> str:
    """Fernet ile sifrele; lib yoksa plaintext kalir (eski kurulum uyumu)"""
    if not _KRIPTO or not metin:
        return metin
    f = Fernet(_sifrele_anahtar())  # type: ignore[name-defined]
    return f.encrypt(metin.encode()).decode()


def _coz(sifreli: str) -> str:
    """Sifrelenmis metni coz; hata durumunda veya eski formatta aynen dondur"""
    if not _KRIPTO or not sifreli:
        return sifreli
    try:
        f = Fernet(_sifrele_anahtar())  # type: ignore[name-defined]
        return f.decrypt(sifreli.encode()).decode()
    except (InvalidToken, Exception):  # type: ignore[name-defined]
        return sifreli  # Eski plaintext format


def tc_maskele(tc: str) -> str:
    """TC Kimlik No'yu maskeler: 12345678901 -> 123****901"""
    if len(tc) >= 7:
        return tc[:3] + "****" + tc[-3:]
    return "***"


def kayitli_kullanici_al() -> tuple[str, str]:
    """Kayitli kullanici bilgilerini oku, (kullanici, sifre) dondur"""
    if not AYAR_DOSYASI.exists():
        return "", ""
    try:
        icerik = AYAR_DOSYASI.read_text(encoding="utf-8")
        veri = json.loads(icerik)
    except (OSError, json.JSONDecodeError, ValueError):
        return "", ""

    k_raw = veri.get("username", "")
    s_raw = veri.get("password", "")

    if not isinstance(k_raw, str):
        return "", ""

    sifreli = veri.get("sifreli", False)
    k = _coz(k_raw) if sifreli else k_raw
    s = _coz(s_raw) if sifreli and isinstance(s_raw, str) else ""
    return k, s


def kullanici_kaydet(k: str, sifre: str = "") -> None:
    """Kullanici bilgilerini sifreli kaydet"""
    if not k:
        return
    try:
        veri = {
            "username": _sifrele(k),
            "password": _sifrele(sifre),
            "sifreli": _KRIPTO,
        }
        AYAR_DOSYASI.write_text(json.dumps(veri), encoding="utf-8")
    except OSError:
        pass
