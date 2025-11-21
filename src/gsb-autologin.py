from __future__ import annotations
import json
import socket
import threading
from pathlib import Path
from urllib.parse import urlparse
import requests
import urllib3
from bs4 import BeautifulSoup
import tkinter as tk
from tkinter import ttk, messagebox
from tkinter.scrolledtext import ScrolledText

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

DEFAULT_LOGIN_URL = "https://wifi.gsb.gov.tr/j_spring_security_check"
USER_CONFIG_PATH = Path(__file__).with_name("user_config.json")

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
        timeout=15,
    )
    response.raise_for_status()
    return response.text

def extract_portal_info(html: str) -> str:
    soup = BeautifulSoup(html, "html.parser")
    center_block = soup.select_one("#content-div > center")
    if not center_block:
        return "3-Bilgi bloğu bulunamadı."

    entries: list[str] = []

    def append_clean_text(node) -> None:
        if not node:
            return
        text = node.get_text(separator=" ", strip=True)
        clean = " ".join(text.split())
        if clean:
            entries.append(clean)

    span_node = center_block.select_one("span.myinfo")
    append_clean_text(span_node)

    first_label = center_block.select_one("label:nth-child(3)")
    append_clean_text(first_label)

    second_label = center_block.select_one("label:nth-child(4)")
    append_clean_text(second_label)

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


class AutoLoginApp:
    def __init__(self, root: tk.Tk) -> None:
        self.root = root
        self.root.title("GSB WiFi AutoLogin v0.9.1")
        self.root.geometry("450x340")
        self.root.resizable(False, False)

        self.url_var = tk.StringVar(value=DEFAULT_LOGIN_URL)
        self.username_var = tk.StringVar()
        self.password_var = tk.StringVar()

        self._build_ui()
        self._load_username()
        messagebox.showwarning("Uyarı", "Bu uygulama eğitim amacıyla yapılmıştır.")
        messagebox.showinfo("Bilgilendirme", "Uygulama başlatıldı. GSB WiFi ağına bağlandıktan sonra giriş yapabilirsiniz.")

    def _build_ui(self) -> None:
        padding = {"padx": 12, "pady": 6}
        main = ttk.Frame(self.root, padding=12)
        main.grid(row=0, column=0, sticky="nsew")

        ttk.Label(main, text="Kullanıcı Adı").grid(row=1, column=0, sticky="w")
        self.username_entry = ttk.Entry(main, textvariable=self.username_var, width=30)
        self.username_entry.grid(row=1, column=1, sticky="ew", **padding)

        ttk.Label(main, text="Şifre").grid(row=2, column=0, sticky="w")
        self.password_entry = ttk.Entry(main, textvariable=self.password_var, show="*", width=30)
        self.password_entry.grid(row=2, column=1, sticky="ew", **padding)

        self.login_button = ttk.Button(main, text="Giriş Yap", command=self._start_login)
        self.login_button.grid(row=3, column=1, sticky="ew", **padding)

        ttk.Label(main, text="Log").grid(row=4, column=0, sticky="nw")
        self.log_text = ScrolledText(main, height=12, width=40, state="disabled")
        self.log_text.grid(row=4, column=1, sticky="nsew", **padding)

        main.columnconfigure(1, weight=1)

    def log(self, message: str) -> None:
        self.log_text.configure(state="normal")
        self.log_text.insert("end", message + "\n")
        self.log_text.see("end")
        self.log_text.configure(state="disabled")

    def _load_username(self) -> None:
        saved = load_saved_username()
        if saved:
            self.username_var.set(saved)

    def _start_login(self) -> None:
        login_url = self.url_var.get().strip()
        username = self.username_var.get().strip()
        password = self.password_var.get()

        if not login_url or not username or not password:
            messagebox.showwarning("Eksik Bilgi", "Lütfen URL, kullanıcı adı ve şifre alanlarını doldurun.")
            return

        save_username(username)
        self.login_button.state(["disabled"])
        self.log("--- Yeni giriş denemesi başlıyor ---")
        thread = threading.Thread(target=self._login_worker, args=(login_url, username, password), daemon=True)
        thread.start()

    def _login_worker(self, login_url: str, username: str, password: str) -> None:
        try:
            try:
                resolved_ip = resolve_login_ip(login_url)
                self._async(lambda: self.log(f"🌐 DNS Çözümleme: {resolved_ip}"))
            except Exception as exc:
                dns_error = str(exc)
                self._async(lambda msg=dns_error: self.log(f"⚠️ DNS çözümlemesi başarısız: {msg}"))

            html = perform_login(login_url, username, password)
            info = extract_portal_info(html)
            self._async(lambda: self.log("📄 Kullanıcı Bilgisi:\n" + info))
            self._async(lambda: messagebox.showinfo("GSB WiFi", "Giriş başarılı."))
        except requests.HTTPError as exc:
            http_error = str(exc)
            self._async(lambda msg=http_error: self.log(f"❌ HTTP hatası: {msg}"))
            self._async(lambda msg=http_error: messagebox.showerror("GSB WiFi", f"HTTP hatası: {msg}"))
        except Exception as exc:
            general_error = str(exc)
            self._async(lambda msg=general_error: self.log(f"❌ Hata: {msg}"))
            self._async(lambda msg=general_error: messagebox.showerror("GSB WiFi", f"Hata: {msg}"))
        finally:
            self._async(lambda: self.login_button.state(["!disabled"]))

    def _async(self, callback) -> None:
        self.root.after(0, callback)


def main() -> None:
    root = tk.Tk()
    app = AutoLoginApp(root)
    root.mainloop()


if __name__ == "__main__":
    main()
