"""Uygulama sinifi - ekran gecisleri, giris akisi ve thread yonetimi."""

from __future__ import annotations

import contextlib
import threading
import time
from tkinter import messagebox

import customtkinter as ctk
import requests

from config import (
    GIRIS_URL,
    WIFI_IMG_YOLU,
    __version__,
    kayitli_kullanici_al,
    kullanici_kaydet,
    renkler,
    tc_maskele,
)
from errors import DNSHatasi, GirisBasarisiz, GSBHata, MaksimumCihaz
from network import cikis_yap, giris_yap, ip_bul, onceki_oturumu_kapat
from parser import bilgi_cek
from ui.giris_ekrani import GirisEkrani
from ui.hosgeldin_ekrani import HosgeldinEkrani
from ui.log_penceresi import LogPenceresi


class Uygulama:
    def __init__(self, ana):
        self.ana = ana

        # Global thread exception handler
        threading.excepthook = self._thread_hata_yakala

        # Pencere ayarlari
        self.ana.title("GSB WiFi AutoLogin")
        self.ana.geometry("420x680")
        self.ana.resizable(False, False)
        self.ana.configure(fg_color=renkler["arkaplan"])

        # Pencere ikonu (taskbar + sol ust kose)
        with contextlib.suppress(Exception):
            self.ana.iconbitmap(str(WIFI_IMG_YOLU))

        ctk.set_appearance_mode("dark")

        self.url_var = ctk.StringVar(value=GIRIS_URL)
        self.kullanici_var = ctk.StringVar()
        self.sifre_var = ctk.StringVar()
        self.kullanici_bilgi: dict = {}
        self.oturum: requests.Session | None = None
        self._giris_aktif = False  # Cift tiklama korumasi
        self._maksimum_bekliyor = False  # Maksimum cihaz akisi aktif mi

        # Ana cerceve
        self.cerceve = ctk.CTkFrame(self.ana, fg_color="transparent")
        self.cerceve.pack(fill="both", expand=True, padx=24, pady=24)

        # Log penceresi
        self.log = LogPenceresi(self.ana)

        # Ekranlar
        self.giris_ekrani = GirisEkrani(self.cerceve, self)
        self.hosgeldin_ekrani = HosgeldinEkrani(self.cerceve, self)

        # Baslangic
        self._giris_goster()
        self._kullanici_yukle()

        # Ilk mesaj
        self.log.yaz("┌─────────────────────────────────┐", "bilgi")
        self.log.yaz(f"│  GSB WiFi AutoLogin v{__version__}      │", "bilgi")
        self.log.yaz("└─────────────────────────────────┘", "bilgi")
        self.log.yaz("")

    def _thread_hata_yakala(self, args):
        """Thread icerisinde yakalanmamis hatalari ele al"""
        self._sonra(lambda: self.log.yaz(f"✗ Beklenmeyen hata: {args.exc_type.__name__}: {args.exc_value}", "hata"))

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
        kayitli_k, kayitli_s = kayitli_kullanici_al()
        if kayitli_k:
            self.kullanici_var.set(kayitli_k)
        if kayitli_s:
            self.sifre_var.set(kayitli_s)

    def _cikis_yap(self):
        self.log.yaz("")
        self.log.yaz("━━━ Çıkış yapılıyor... ━━━", "uyari")

        s = self.oturum

        def islem():
            basarili = cikis_yap(s)
            if basarili:
                self._sonra(lambda: self.log.yaz("✓ Oturum sonlandırıldı", "basarili"))
            else:
                self._sonra(lambda: self.log.yaz("⚠ Çıkış isteği gönderilemedi", "uyari"))
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
        # Thread-safe: cift tiklama korumasi
        if self._giris_aktif:
            return

        url = self.url_var.get().strip()
        k = self.kullanici_var.get().strip()
        s = self.sifre_var.get()

        if not k or not s:
            messagebox.showwarning("Eksik Bilgi", "Kullanıcı adı ve şifre gerekli.")
            return

        kullanici_kaydet(k)
        self._giris_aktif = True
        self.giris_ekrani.giris_btn.configure(state="disabled", text="Bağlanıyor...")
        self.giris_ekrani.durum.guncelle("Bağlanıyor...", "yukleniyor")
        self.log.yaz("")
        self.log.yaz("━━━ Giriş başlatılıyor ━━━", "bilgi")
        self.log.yaz(f"  Kullanıcı: {tc_maskele(k)}", "soluk")

        t = threading.Thread(target=self._giris_islem, args=(url, k, s), daemon=True)
        t.start()

    def _giris_islem(self, url, k, s, _maksimum_sonrasi=False):
        try:
            try:
                ip = ip_bul(url)
                self._sonra(lambda: self.log.yaz(f"🌐 Sunucu: {ip}", "soluk"))
            except DNSHatasi:
                self._sonra(
                    lambda: self.log.yaz(
                        "⚠ DNS hatası: Sunucuya ulaşılamıyor. "
                        "VPN aktifse devre dışı bırakın veya ağ yapılandırmanızı kontrol edin.",
                        "uyari",
                    )
                )
            except Exception as e:
                hata_msg = str(e)
                self._sonra(lambda: self.log.yaz(f"⚠ IP alınamadı: {hata_msg}", "uyari"))

            # Eski oturumu kapat (session leak onlemi)
            if self.oturum:
                with contextlib.suppress(Exception):
                    self.oturum.close()

            def deneme_bildir(deneme, toplam):
                self._sonra(lambda: self.log.yaz(f"  ↻ {deneme}. deneme... ({deneme}/{toplam})", "uyari"))
                self._sonra(
                    lambda: self.giris_ekrani.durum.guncelle(f"Yeniden deneniyor ({deneme}/{toplam})...", "yukleniyor")
                )

            oturum, html = giris_yap(url, k, s, deneme_callback=deneme_bildir)
            self.oturum = oturum
            bilgi = bilgi_cek(html)
            self.kullanici_bilgi = bilgi

            kullanici_kaydet(k, s)
            self._sonra(lambda: self.log.yaz("✓ Bağlantı başarılı!", "basarili"))
            self._sonra(lambda: self.log.yaz(f"  Kullanıcı: {bilgi.get('isim', 'N/A')}", "soluk"))
            konum = bilgi.get("konum", "")
            if konum:
                self._sonra(lambda: self.log.yaz(f"  Konum: {konum}", "soluk"))
            kota = bilgi.get("kota", {})
            kalan_str = kota.get("kalan_mb", "")
            toplam_str = kota.get("toplam_mb", "")
            if kalan_str and toplam_str:
                try:
                    kalan_gb = float(kalan_str) / 1024
                    toplam_gb = float(toplam_str) / 1024
                    self._sonra(lambda: self.log.yaz(f"  Kota: {kalan_gb:.1f} / {toplam_gb:.1f} GB", "soluk"))
                except ValueError:
                    pass
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Bağlı", "basarili"))
            self._sonra(lambda: self.ana.after(600, self._hosgeldin_goster))

        except MaksimumCihaz as e:
            bilgi = e.cihaz_bilgisi
            _oturum = e.session
            _html = e.html
            if _maksimum_sonrasi:
                # Disconnect denendikten sonra hâlâ max device — döngüyü kır
                hata = "Önceki cihazın bağlantısı düşürülemedi. Lütfen cihazdan manuel çıkış yapın."
                self._sonra(lambda: self.log.yaz(f"✗ {hata}", "hata"))
                self._sonra(lambda: self.giris_ekrani.durum.guncelle("Hata", "hata"))
                self._sonra(lambda: messagebox.showerror("Hata", hata))
            else:
                self._maksimum_bekliyor = True
                baslik = bilgi.get("baslangic", "?")
                mac = bilgi.get("mac", "?")
                konum = bilgi.get("konum", "?")
                mesaj = (
                    f"Bu hesapla başka bir cihaz zaten bağlı.\n\n"
                    f"  Bağlantı Başlangıcı: {baslik}\n"
                    f"  MAC Adresi: {mac}\n"
                    f"  Konum: {konum}\n\n"
                    f"Önceki cihazın bağlantısını düşürüp tekrar bağlanılsın mı?"
                )
                self._sonra(lambda: self._maksimum_onay_sor(mesaj, url, k, s, _oturum, _html))

        except GirisBasarisiz as e:
            mesaj = e.kullanici_mesaji
            self._sonra(lambda: self.log.yaz(f"✗ {mesaj}", "hata"))
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Giriş Başarısız", "hata"))
            self._sonra(lambda: messagebox.showerror("Giriş Başarısız", mesaj))

        except GSBHata as e:
            mesaj = e.kullanici_mesaji
            self._sonra(lambda: self.log.yaz(f"✗ {mesaj}", "hata"))
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Hata", "hata"))
            self._sonra(lambda: messagebox.showerror("Hata", mesaj))

        except requests.HTTPError as e:
            hata_msg = str(e)
            self._sonra(lambda: self.log.yaz(f"✗ HTTP Hatası: {hata_msg}", "hata"))
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Hata", "hata"))
            self._sonra(lambda: messagebox.showerror("Hata", f"HTTP: {hata_msg}"))

        except Exception as e:
            hata_msg = str(e)
            self._sonra(lambda: self.log.yaz(f"✗ Hata: {hata_msg}", "hata"))
            self._sonra(lambda: self.giris_ekrani.durum.guncelle("Hata", "hata"))
            self._sonra(lambda: messagebox.showerror("Hata", hata_msg))

        finally:
            self._giris_aktif = False
            if not self._maksimum_bekliyor:
                self._sonra(lambda: self.giris_ekrani.giris_btn.configure(state="normal", text="Bağlan"))

    def _maksimum_onay_sor(self, mesaj, url, k, s, oturum, html):
        """Maksimum cihaz durumunda kullaniciya onay sor; evet ise onceki oturumu kaldir ve tekrar baglan."""
        self._maksimum_bekliyor = False
        onay = messagebox.askyesno("Maksimum Cihaz Limiti", mesaj)
        if onay:
            self._giris_aktif = True
            self.giris_ekrani.giris_btn.configure(state="disabled", text="Bağlanıyor...")
            self.giris_ekrani.durum.guncelle("Önceki oturum kapatılıyor...", "yukleniyor")
            self.log.yaz("↻ Önceki cihazın bağlantısı kapatılıyor...", "uyari")
            t = threading.Thread(target=self._maksimum_isle, args=(url, k, s, oturum, html), daemon=True)
            t.start()
        else:
            self.giris_ekrani.giris_btn.configure(state="normal", text="Bağlan")
            self.giris_ekrani.durum.guncelle("İptal edildi", "bekle")

    def _maksimum_isle(self, url, k, s, oturum, html):
        """Onceki oturumu kapat ve giris denemesini tekrarla."""
        basarili = onceki_oturumu_kapat(oturum, html, url)
        if basarili:
            self._sonra(lambda: self.log.yaz("✓ Önceki oturum kapatıldı", "basarili"))
        else:
            self._sonra(lambda: self.log.yaz("⚠ Oturum kapatma belirsiz, yeniden deneniyor...", "uyari"))
        time.sleep(2)
        self._giris_islem(url, k, s, _maksimum_sonrasi=True)

    def _sonra(self, fn):
        self.ana.after(0, fn)
