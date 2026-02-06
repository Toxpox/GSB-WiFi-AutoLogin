from __future__ import annotations

import re
import json
import socket
from pathlib import Path
from urllib.parse import urlparse

import requests
import urllib3
from bs4 import BeautifulSoup
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

DEFAULT_LOGIN_URL = "https://wifi.gsb.gov.tr/j_spring_security_check"
USER_CONFIG_PATH = Path(__file__).with_name("user_config.json")
APP_VERSION = "0.9.5"

def resolve_login_ip(login_url: str) -> str:
    parsed = urlparse(login_url)
    hostname = parsed.hostname
    if not hostname:
        raise ValueError("Login URL içinde hostname bulunamadı.")
    try:
        return socket.gethostbyname(hostname)
    except socket.gaierror as exc:
        raise RuntimeError(f"DNS çözümlemesi başarısız: {hostname}") from exc
    
def perform_login(login_url: str, username: str, password: str) -> str:
    session = requests.Session()
    payload = {
        "j_username": username,
        "j_password": password,
        "submit": "Giriş",
    }
    response = session.post(
        login_url,
        data=payload,
        allow_redirects=True,
        verify=False,
        timeout=10,
    )
    response.raise_for_status()
    return response.text


def extract_portal_info(html: str) -> str:
    soup = BeautifulSoup(html, 'lxml')
    entries: list[str] = []
    seen: set[str] = set()

    def clean_text(text: str) -> str:
        clean = ' '.join(text.split())
        clean = re.sub(r'\s*:\s*', ': ', clean)
        return clean

    name_node = soup.find('span', class_='myinfo')
    if name_node:
        name = clean_text(name_node.get_text(strip=True))
        if name:
            entries.append(name)

    for label in soup.select('label.myinfo'):
        text = clean_text(label.get_text(strip=True))
        if 'Last Login' in text:
            entries.append(text)
            break

    for node in soup.select('label[id*="j_idt95"]'):
        text = clean_text(node.get_text(strip=True))
        if text and ('quota' in text.lower() or 'expired' in text.lower()):
            if text not in seen:
                entries.append(text)
                seen.add(text)

    return "\n".join(entries) if entries else "Bilgi bloğu boş."

def load_saved_username() -> str:
    if not USER_CONFIG_PATH.exists():
        return ""
    try:
        content = USER_CONFIG_PATH.read_text(encoding="utf-8")
        data = json.loads(content)
    except (OSError, json.JSONDecodeError):
        return ""
    username = data.get("username", "")
    return username if isinstance(username, str) else ""


def save_username(username: str) -> None:
    if not username:
        return
    payload = json.dumps({"username": username})
    try:
        USER_CONFIG_PATH.write_text(payload, encoding="utf-8")
    except OSError:
        pass

class LoginRequest(BaseModel):
    username: str = Field(min_length=1, description="GSB WiFi kullanıcı adı")
    password: str = Field(min_length=1, description="GSB WiFi parolası")
    login_url: str | None = Field(default=None, description="Opsiyonel özel giriş URL'si") #?
    remember: bool = Field(default=True, description="Kullanıcı adını yerel dosyada sakla")


class LoginResponse(BaseModel):
    login_url: str
    portal_info: str
    resolved_ip: str | None = None
    dns_error: str | None = None


class UsernameResponse(BaseModel):
    username: str


class UsernameUpdateRequest(BaseModel):
    username: str = Field(min_length=1, description="Kaydedilecek kullanıcı adı")


app = FastAPI(
    title="GSB WiFi AutoLogin API",
    version=APP_VERSION,
    description=(
        "GSB WiFi için otomatik giriş işlevlerini sağlayan REST API. "
            ),
)


@app.get("/health")
def health_check() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/config/username", response_model=UsernameResponse)
def get_saved_username() -> UsernameResponse:
    return UsernameResponse(username=load_saved_username())


@app.post("/config/username", response_model=UsernameResponse)
def update_saved_username(payload: UsernameUpdateRequest) -> UsernameResponse:
    save_username(payload.username)
    return UsernameResponse(username=payload.username)


@app.post("/login", response_model=LoginResponse)
def login(payload: LoginRequest) -> LoginResponse:
    login_url = payload.login_url.strip() if payload.login_url else DEFAULT_LOGIN_URL

    resolved_ip: str | None = None
    dns_error: str | None = None
    try:
        resolved_ip = resolve_login_ip(login_url)
    except Exception as exc:
        dns_error = str(exc)

    try:
        html = perform_login(login_url, payload.username, payload.password)
    except requests.HTTPError as exc:
        raise HTTPException(status_code=502, detail=f"HTTP hatası: {exc}") from exc
    except Exception as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc

    info = extract_portal_info(html)
    if payload.remember:
        save_username(payload.username)

    return LoginResponse(
        login_url=login_url,
        portal_info=info,
        resolved_ip=resolved_ip,
        dns_error=dns_error,
    )


def run() -> None:
    import uvicorn

    uvicorn.run(
        "gsb-autologin-core:app",
        host="127.0.0.1",
        port=8000,
        reload=False,
    )


if __name__ == "__main__":
    run()
