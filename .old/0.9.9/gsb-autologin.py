"""
GSB WiFi AutoLogin
==================
Bu uygulama GSB WiFi ağına otomatik giriş yapar.
Yazar: Toxpox
Ver: 0.9.9
"""

from __future__ import annotations

import json
import socket
import threading
from pathlib import Path
from urllib.parse import urlparse

import requests
import urllib3
from bs4 import BeautifulSoup

import customtkinter as ctk
from tkinter import messagebox

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)


# Ayarlar
########################################################################

GIRIS_URL = "https://wifi.gsb.gov.tr/j_spring_security_check"
CIKIS_URL = "https://wifi.gsb.gov.tr/cikisSon.html?logout=1"
AYAR_DOSYASI = Path(__file__).with_name("user_config.json")


# Renkler
########################################################################

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


# Yardımcı Fonksiyonlar
########################################################################

def ip_bul(url):
    """URL'den IP bul"""
    parsed = urlparse(url)
    host = parsed.hostname
    if not host:
        raise ValueError("URL hatalı")
    try:
        return socket.gethostbyname(host)
    except:
        raise RuntimeError(f"DNS hatası: {host}")


def giris_yap(url, kullanici, sifre):
    """Giriş yap ve session döndür"""
    s = requests.Session()
    
    veri = {
        "j_username": kullanici,
        "j_password": sifre,
        "submit": "Giriş",
    }
    
    r = s.post(url, data=veri, allow_redirects=True, verify=False, timeout=15)
    r.raise_for_status()
    
    return s, r.text


def bilgi_cek(html):
    """Portal'dan kullanıcı bilgilerini çek"""
    soup = BeautifulSoup(html, "html.parser")
    blok = soup.select_one("#content-div > center")
    
    bilgi = {"isim": "Kullanıcı", "detaylar": [], "ham": ""}
    
    if not blok:
        bilgi["ham"] = "Bilgi yok"
        return bilgi

    span = blok.select_one("span.myinfo")
    if span:
        txt = span.get_text(separator=" ", strip=True)
        txt = " ".join(txt.split())
        if txt:
            bilgi["isim"] = txt
            bilgi["detaylar"].append(txt)

    lbl1 = blok.select_one("label:nth-child(3)")
    if lbl1:
        txt = lbl1.get_text(separator=" ", strip=True)
        txt = " ".join(txt.split())
        if txt:
            bilgi["detaylar"].append(txt)

    lbl2 = blok.select_one("label:nth-child(4)")
    if lbl2:
        txt = lbl2.get_text(separator=" ", strip=True)
        txt = " ".join(txt.split())
        if txt:
            bilgi["detaylar"].append(txt)

    bilgi["ham"] = "\n".join(bilgi["detaylar"]) if bilgi["detaylar"] else "Bilgi yok"
    return bilgi


# Kullanıcı Kayıt
########################################################################

def kayitli_kullanici_al():
    """Kayıtlı kullanıcıyı oku"""
    if not AYAR_DOSYASI.exists():
        return ""
    try:
        icerik = AYAR_DOSYASI.read_text(encoding="utf-8")
        veri = json.loads(icerik)
    except:
        return ""
    k = veri.get("username", "")
    return k if isinstance(k, str) else ""


def kullanici_kaydet(k):
    """Kullanıcıyı kaydet"""
    if not k:
        return
    try:
        AYAR_DOSYASI.write_text(json.dumps({"username": k}), encoding="utf-8")
    except:
        pass


# UI Bileşenleri
########################################################################

class AnaButon(ctk.CTkButton):
    def __init__(self, parent, **kw):
        super().__init__(
            parent,
            corner_radius=14,
            height=48,
            font=ctk.CTkFont(family="Segoe UI Semibold", size=14),
            fg_color=renkler["mor"],
            hover_color=renkler["mor_hover"],
            text_color=renkler["beyaz"],
            border_width=0,
            **kw
        )


class IkinciButon(ctk.CTkButton):
    def __init__(self, parent, **kw):
        super().__init__(
            parent,
            corner_radius=14,
            height=48,
            font=ctk.CTkFont(family="Segoe UI Semibold", size=14),
            fg_color="transparent",
            hover_color=renkler["kart2"],
            text_color=renkler["gri"],
            border_width=2,
            border_color=renkler["cizgi"],
            **kw
        )


class GirisAlani(ctk.CTkEntry):
    def __init__(self, parent, ipucu="", gizli=False, **kw):
        super().__init__(
            parent,
            height=48,
            corner_radius=12,
            border_width=2,
            border_color=renkler["cizgi"],
            fg_color=renkler["input_bg"],
            text_color=renkler["beyaz"],
            placeholder_text=ipucu,
            placeholder_text_color=renkler["gri_koyu"],
            font=ctk.CTkFont(family="Segoe UI", size=14),
            show="●" if gizli else "",
            **kw
        )
        self.bind("<FocusIn>", lambda e: self.configure(border_color=renkler["focus"]))
        self.bind("<FocusOut>", lambda e: self.configure(border_color=renkler["cizgi"]))


class DurumRozeti(ctk.CTkFrame):
    def __init__(self, parent, **kw):
        super().__init__(parent, fg_color="transparent", height=28, **kw)
        
        self.kutu = ctk.CTkFrame(self, fg_color=renkler["kart2"], corner_radius=14, height=28)
        self.kutu.pack(expand=True)
        
        self.ic = ctk.CTkFrame(self.kutu, fg_color="transparent")
        self.ic.pack(padx=14, pady=5)
        
        self.nokta = ctk.CTkLabel(self.ic, text="●", font=ctk.CTkFont(size=8), text_color=renkler["gri_koyu"], width=12)
        self.nokta.pack(side="left")
        
        self.yazi = ctk.CTkLabel(self.ic, text="Hazır", font=ctk.CTkFont(family="Segoe UI", size=11), text_color=renkler["gri_koyu"])
        self.yazi.pack(side="left", padx=(4, 0))
        
        self.animasyon = False
    
    def guncelle(self, metin, durum="bekle"):
        durumlar = {
            "bekle": (renkler["gri_koyu"], renkler["kart2"]),
            "yukleniyor": (renkler["sari"], renkler["sari_koyu"]),
            "basarili": (renkler["yesil"], renkler["yesil_koyu"]),
            "hata": (renkler["kirmizi"], renkler["kirmizi_koyu"]),
        }
        renk, bg = durumlar.get(durum, durumlar["bekle"])
        self.nokta.configure(text_color=renk)
        self.yazi.configure(text=metin, text_color=renk)
        self.kutu.configure(fg_color=bg)
        
        if durum == "yukleniyor":
            self.animasyon = True
            self._yanip_son()
        else:
            self.animasyon = False
    
    def _yanip_son(self):
        if not self.animasyon:
            return
        simdi = self.nokta.cget("text")
        self.nokta.configure(text="○" if simdi == "●" else "●")
        self.after(500, self._yanip_son)


class BilgiKutusu(ctk.CTkFrame):
    def __init__(self, parent, ikon, baslik, deger, **kw):
        super().__init__(parent, fg_color=renkler["kart"], corner_radius=12, border_width=1, border_color=renkler["cizgi"], **kw)
        
        ic = ctk.CTkFrame(self, fg_color="transparent")
        ic.pack(fill="both", expand=True, padx=12, pady=12)
        
        ctk.CTkLabel(ic, text=ikon, font=ctk.CTkFont(size=28)).pack(anchor="w")
        
        self.deger_lbl = ctk.CTkLabel(ic, text=deger, font=ctk.CTkFont(family="Segoe UI Semibold", size=14), text_color=renkler["beyaz"], anchor="w")
        self.deger_lbl.pack(fill="x", pady=(8, 2))
        
        ctk.CTkLabel(ic, text=baslik, font=ctk.CTkFont(family="Segoe UI", size=11), text_color=renkler["gri_koyu"], anchor="w").pack(fill="x")
    
    def deger_ayarla(self, d):
        self.deger_lbl.configure(text=d)


# Log Penceresi
########################################################################

class LogPenceresi(ctk.CTkToplevel):
    def __init__(self, parent, **kw):
        super().__init__(parent, **kw)
        
        self.title("Sistem Günlüğü")
        self.geometry("450x350")
        self.configure(fg_color=renkler["arkaplan"])
        self.resizable(True, True)
        self.minsize(350, 250)
        
        ust = ctk.CTkFrame(self, fg_color="transparent", height=40)
        ust.pack(fill="x", padx=16, pady=(16, 8))
        
        ctk.CTkLabel(ust, text="📋 Sistem Günlüğü", font=ctk.CTkFont(family="Segoe UI Semibold", size=14), text_color=renkler["gri"]).pack(side="left")
        
        ctk.CTkButton(
            ust, text="Temizle", font=ctk.CTkFont(size=11), width=64, height=26, corner_radius=8,
            fg_color="transparent", hover_color=renkler["kart2"], text_color=renkler["gri_koyu"],
            border_width=1, border_color=renkler["cizgi"], command=self.temizle
        ).pack(side="right")
        
        self.alan = ctk.CTkTextbox(
            self, corner_radius=12, border_width=1, border_color=renkler["cizgi"],
            fg_color=renkler["kart"], text_color=renkler["gri"],
            font=ctk.CTkFont(family="Cascadia Code", size=12), wrap="word"
        )
        self.alan.pack(fill="both", expand=True, padx=16, pady=(0, 16))
        self.alan.configure(state="disabled")
        
        self.alan._textbox.tag_configure("basarili", foreground=renkler["yesil"])
        self.alan._textbox.tag_configure("uyari", foreground=renkler["sari"])
        self.alan._textbox.tag_configure("hata", foreground=renkler["kirmizi"])
        self.alan._textbox.tag_configure("bilgi", foreground=renkler["mor_acik"])
        self.alan._textbox.tag_configure("soluk", foreground=renkler["gri_koyu"])
        
        self.protocol("WM_DELETE_WINDOW", self.withdraw)
        self.withdraw()
    
    def yaz(self, msg, tip=None):
        self.alan.configure(state="normal")
        if tip:
            self.alan._textbox.insert("end", msg + "\n", tip)
        else:
            self.alan._textbox.insert("end", msg + "\n")
        self.alan.see("end")
        self.alan.configure(state="disabled")
    
    def temizle(self):
        self.alan.configure(state="normal")
        self.alan.delete("1.0", "end")
        self.alan.configure(state="disabled")
    
    def goster(self):
        self.deiconify()
        self.lift()
        self.focus_force()


# Ekranlar
########################################################################

class GirisEkrani(ctk.CTkFrame):
    def __init__(self, parent, uygulama, **kw):
        super().__init__(parent, fg_color="transparent", **kw)
        self.uyg = uygulama
        self._olustur()
    
    def _olustur(self):
        # Başlık
        ust = ctk.CTkFrame(self, fg_color="transparent")
        ust.pack(fill="x", pady=(0, 20))
        
        ikon_kutu = ctk.CTkFrame(ust, width=64, height=64, corner_radius=18, fg_color=renkler["kart"], border_width=1, border_color=renkler["cizgi"])
        ikon_kutu.pack()
        ikon_kutu.pack_propagate(False)
        ctk.CTkLabel(ikon_kutu, text="📶", font=ctk.CTkFont(size=28)).place(relx=0.5, rely=0.5, anchor="center")
        
        ctk.CTkLabel(ust, text="GSB WiFi", font=ctk.CTkFont(family="Segoe UI", size=26, weight="bold"), text_color=renkler["beyaz"]).pack(pady=(12, 0))
        ctk.CTkLabel(ust, text="Otomatik Bağlantı Sistemi", font=ctk.CTkFont(family="Segoe UI", size=12), text_color=renkler["gri_koyu"]).pack(pady=(2, 0))
        
        # Form kartı
        kart = ctk.CTkFrame(self, fg_color=renkler["kart"], corner_radius=18, border_width=1, border_color=renkler["cizgi"])
        kart.pack(fill="x", pady=(0, 16))
        
        kart_ic = ctk.CTkFrame(kart, fg_color="transparent")
        kart_ic.pack(fill="x", padx=20, pady=24)
        
        # Kullanıcı adı
        ctk.CTkLabel(kart_ic, text="👤  Kullanıcı Adı", font=ctk.CTkFont(family="Segoe UI Semibold", size=12), text_color=renkler["gri"], anchor="w").pack(fill="x")
        self.kullanici_gir = GirisAlani(kart_ic, ipucu="TC Kimlik No", textvariable=self.uyg.kullanici_var)
        self.kullanici_gir.pack(fill="x", pady=(6, 16))
        
        # Şifre
        ctk.CTkLabel(kart_ic, text="🔐  Şifre", font=ctk.CTkFont(family="Segoe UI Semibold", size=12), text_color=renkler["gri"], anchor="w").pack(fill="x")
        self.sifre_gir = GirisAlani(kart_ic, ipucu="••••••••", gizli=True, textvariable=self.uyg.sifre_var)
        self.sifre_gir.pack(fill="x", pady=(6, 20))
        
        # Giriş butonu
        self.giris_btn = AnaButon(kart_ic, text="Bağlan", command=self.uyg._giris_baslat)
        self.giris_btn.pack(fill="x")
        
        # Durum
        self.durum = DurumRozeti(kart_ic)
        self.durum.pack(pady=(16, 0))
        
        # Log butonu
        log_btn = ctk.CTkButton(
            self, text="📋 Sistem Günlüğü", font=ctk.CTkFont(size=12), height=36, corner_radius=10,
            fg_color=renkler["kart"], hover_color=renkler["kart2"], text_color=renkler["gri_koyu"],
            border_width=1, border_color=renkler["cizgi"], command=self.uyg._log_goster
        )
        log_btn.pack(fill="x")
        
        # Uyarı
        ctk.CTkLabel(self, text="⚠ Bu uygulama eğitim amaçlıdır.", font=ctk.CTkFont(family="Segoe UI", size=11), text_color=renkler["gri_koyu"]).pack(pady=(16, 0))


class HosgeldinEkrani(ctk.CTkFrame):
    def __init__(self, parent, uygulama, **kw):
        super().__init__(parent, fg_color="transparent", **kw)
        self.uyg = uygulama
        self._olustur()
    
    def _olustur(self):
        # Başarı başlığı
        ust = ctk.CTkFrame(self, fg_color="transparent")
        ust.pack(fill="x", pady=(20, 0))
        
        ikon_kutu = ctk.CTkFrame(ust, width=80, height=80, corner_radius=40, fg_color=renkler["yesil_koyu"], border_width=3, border_color=renkler["yesil"])
        ikon_kutu.pack()
        ikon_kutu.pack_propagate(False)
        ctk.CTkLabel(ikon_kutu, text="✓", font=ctk.CTkFont(size=40, weight="bold"), text_color=renkler["yesil"]).place(relx=0.5, rely=0.5, anchor="center")
        
        ctk.CTkLabel(ust, text="Hoş Geldiniz!", font=ctk.CTkFont(family="Segoe UI", size=28, weight="bold"), text_color=renkler["beyaz"]).pack(pady=(20, 4))
        
        self.isim_lbl = ctk.CTkLabel(ust, text="", font=ctk.CTkFont(family="Segoe UI", size=16), text_color=renkler["mor_acik"])
        self.isim_lbl.pack()
        
        # Bağlantı durumu
        durum_kutu = ctk.CTkFrame(ust, fg_color=renkler["yesil_koyu"], corner_radius=20, height=36)
        durum_kutu.pack(pady=(16, 0))
        ctk.CTkLabel(durum_kutu, text="● Bağlantı Aktif", font=ctk.CTkFont(family="Segoe UI Semibold", size=13), text_color=renkler["yesil"]).pack(padx=20, pady=8)
        
        # Detay kartı
        self.detay_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.detay_frame.pack(fill="x", expand=True, pady=(24, 0))
        
        self.detay_kart = ctk.CTkFrame(self.detay_frame, fg_color=renkler["kart"], corner_radius=16, border_width=1, border_color=renkler["cizgi"])
        
        detay_ic = ctk.CTkFrame(self.detay_kart, fg_color="transparent")
        detay_ic.pack(fill="x", padx=20, pady=16)
        
        ctk.CTkLabel(detay_ic, text="👤 Kullanıcı Bilgileri", font=ctk.CTkFont(family="Segoe UI Semibold", size=13), text_color=renkler["gri"], anchor="w").pack(fill="x")
        
        self.detay_lbl = ctk.CTkLabel(detay_ic, text="", font=ctk.CTkFont(family="Segoe UI", size=14), text_color=renkler["beyaz"], anchor="w", justify="left")
        self.detay_lbl.pack(fill="x", pady=(8, 0))
        
        # Butonlar
        btn_alan = ctk.CTkFrame(self, fg_color="transparent")
        btn_alan.pack(fill="x", side="bottom", pady=(20, 0))
        
        ctk.CTkButton(
            btn_alan, text="📋 Sistem Günlüğü", font=ctk.CTkFont(size=12), height=36, corner_radius=10,
            fg_color="transparent", hover_color=renkler["kart"], text_color=renkler["gri_koyu"],
            command=self.uyg._log_goster
        ).pack(fill="x", pady=(0, 10))
        
        IkinciButon(btn_alan, text="Çıkış Yap", command=self.uyg._cikis_yap).pack(fill="x")
    
    def bilgi_goster(self, bilgi):
        isim = bilgi.get("isim", "Kullanıcı")
        self.isim_lbl.configure(text=isim)
        
        detaylar = bilgi.get("detaylar", [])
        if len(detaylar) > 1:
            self.detay_kart.pack(fill="x")
            self.detay_lbl.configure(text="\n".join(detaylar[1:]))
        else:
            self.detay_kart.pack_forget()


# Ana Uygulama
########################################################################

class Uygulama:
    def __init__(self, ana):
        self.ana = ana
        
        # Pencere ayarları
        self.ana.title("GSB WiFi AutoLogin")
        self.ana.geometry("420x560")
        self.ana.resizable(False, False)
        self.ana.configure(fg_color=renkler["arkaplan"])
        
        ctk.set_appearance_mode("dark")
        
        # Değişkenler
        self.url_var = ctk.StringVar(value=GIRIS_URL)
        self.kullanici_var = ctk.StringVar()
        self.sifre_var = ctk.StringVar()
        self.kullanici_bilgi = {}
        self.oturum = None
        
        # Ana çerçeve
        self.cerceve = ctk.CTkFrame(self.ana, fg_color="transparent")
        self.cerceve.pack(fill="both", expand=True, padx=24, pady=24)
        
        # Log penceresi
        self.log = LogPenceresi(self.ana)
        
        # Ekranlar
        self.giris_ekrani = GirisEkrani(self.cerceve, self)
        self.hosgeldin_ekrani = HosgeldinEkrani(self.cerceve, self)
        
        # Başlangıç
        self._giris_goster()
        self._kullanici_yukle()
        
        # İlk mesaj
        self.log.yaz("┌─────────────────────────────────┐", "bilgi")
        self.log.yaz("│  GSB WiFi AutoLogin v1.0        │", "bilgi")
        self.log.yaz("└─────────────────────────────────┘", "bilgi")
        self.log.yaz("")
    
    def _giris_goster(self):
        self.hosgeldin_ekrani.pack_forget()
        self.giris_ekrani.pack(fill="both", expand=True)
    
    def _hosgeldin_goster(self):
        self.giris_ekrani.pack_forget()
        self.hosgeldin_ekrani.bilgi_goster(self.kullanici_bilgi)
        self.hosgeldin_ekrani.pack(fill="both", expand=True)
    
    def _log_goster(self):
        self.log.goster()

    def _kullanici_yukle(self):
        kayitli = kayitli_kullanici_al()
        if kayitli:
            self.kullanici_var.set(kayitli)

    def _cikis_yap(self):
        self.log.yaz("")
        self.log.yaz("━━━ Çıkış yapılıyor... ━━━", "uyari")
        
        s = self.oturum
        
        def islem():
            try:
                if s:
                    s.get(CIKIS_URL, verify=False, timeout=5)
                else:
                    requests.get(CIKIS_URL, verify=False, timeout=5)
                self._sonra(lambda: self.log.yaz("✓ Oturum sonlandırıldı", "basarili"))
            except:
                pass
            finally:
                self._sonra(self._cikis_bitti)
        
        t = threading.Thread(target=islem, daemon=True)
        t.start()
    
    def _cikis_bitti(self):
        self.sifre_var.set("")
        self.kullanici_bilgi = {}
        self.oturum = None
        self._giris_goster()
        self.giris_ekrani.durum.guncelle("Hazır", "bekle")

    def _giris_baslat(self):
        url = self.url_var.get().strip()
        k = self.kullanici_var.get().strip()
        s = self.sifre_var.get()

        if not k or not s:
            messagebox.showwarning("Eksik Bilgi", "Kullanıcı adı ve şifre gerekli.")
            return

        kullanici_kaydet(k)
        self.giris_ekrani.giris_btn.configure(state="disabled", text="Bağlanıyor...")
        self.giris_ekrani.durum.guncelle("Bağlanıyor...", "yukleniyor")
        self.log.yaz("")
        self.log.yaz("━━━ Giriş başlatılıyor ━━━", "bilgi")
        
        t = threading.Thread(target=self._giris_islem, args=(url, k, s), daemon=True)
        t.start()

    def _giris_islem(self, url, k, s):
        try:
            try:
                ip = ip_bul(url)
                self._sonra(lambda: self.log.yaz(f"🌐 Sunucu: {ip}", "soluk"))
            except Exception as e:
                self._sonra(lambda: self.log.yaz(f"⚠ DNS hatası: {e}", "uyari"))

            oturum, html = giris_yap(url, k, s)
            self.oturum = oturum
            bilgi = bilgi_cek(html)
            self.kullanici_bilgi = bilgi
            
            self._sonra(lambda: self.log.yaz("✓ Bağlantı başarılı!", "basarili"))
            self._sonra(lambda: self.log.yaz(f"  Kullanıcı: {bilgi.get('isim', 'N/A')}", "soluk"))
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Bağlı", "basarili"))
            self._sonra(lambda: self.ana.after(600, self._hosgeldin_goster))
            
        except requests.HTTPError as e:
            self._sonra(lambda: self.log.yaz(f"✗ HTTP Hatası: {e}", "hata"))
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Hata", "hata"))
            self._sonra(lambda: messagebox.showerror("Hata", f"HTTP: {e}"))
            
        except Exception as e:
            self._sonra(lambda: self.log.yaz(f"✗ Hata: {e}", "hata"))
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Hata", "hata"))
            self._sonra(lambda: messagebox.showerror("Hata", str(e)))
            
        finally:
            self._sonra(lambda: self.giris_ekrani.giris_btn.configure(state="normal", text="Bağlan"))

    def _sonra(self, fn):
        self.ana.after(0, fn)


# Başlangıç
########################################################################

def main():
    ana = ctk.CTk()
    app = Uygulama(ana)
    ana.mainloop()


if __name__ == "__main__":
    main()
