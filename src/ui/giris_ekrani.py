"""Giris formu - TC ve sifre girisi, durum gostergesi."""

from __future__ import annotations

import customtkinter as ctk
from PIL import Image

from config import WIFI_IMG_YOLU, renkler
from ui.widgets import AnaButon, GirisAlani, DurumRozeti


class GirisEkrani(ctk.CTkFrame):
    def __init__(self, parent, uygulama, **kw):
        super().__init__(parent, fg_color="transparent", **kw)
        self.uyg = uygulama
        self._olustur()

    def _olustur(self):
        # Baslik
        ust = ctk.CTkFrame(self, fg_color="transparent")
        ust.pack(fill="x", pady=(0, 20))

        ikon_kutu = ctk.CTkFrame(
            ust, width=64, height=64, corner_radius=18,
            fg_color=renkler["kart"], border_width=1, border_color=renkler["cizgi"],
        )
        ikon_kutu.pack()
        ikon_kutu.pack_propagate(False)
        try:
            self._logo_img = ctk.CTkImage(light_image=Image.open(WIFI_IMG_YOLU), size=(40, 40))
            ctk.CTkLabel(ikon_kutu, image=self._logo_img, text="").place(relx=0.5, rely=0.5, anchor="center")
        except Exception:
            ctk.CTkLabel(ikon_kutu, text="📶", font=ctk.CTkFont(size=28)).place(relx=0.5, rely=0.5, anchor="center")

        ctk.CTkLabel(
            ust, text="GSB WiFi",
            font=ctk.CTkFont(family="Segoe UI", size=26, weight="bold"),
            text_color=renkler["beyaz"],
        ).pack(pady=(12, 0))
        ctk.CTkLabel(
            ust, text="Otomatik Bağlantı Sistemi",
            font=ctk.CTkFont(family="Segoe UI", size=12),
            text_color=renkler["gri_koyu"],
        ).pack(pady=(2, 0))

        # Form karti
        kart = ctk.CTkFrame(
            self, fg_color=renkler["kart"], corner_radius=18,
            border_width=1, border_color=renkler["cizgi"],
        )
        kart.pack(fill="x", pady=(0, 16))

        kart_ic = ctk.CTkFrame(kart, fg_color="transparent")
        kart_ic.pack(fill="x", padx=20, pady=24)

        # Kullanici adi
        ctk.CTkLabel(
            kart_ic, text="👤  Kullanıcı Adı",
            font=ctk.CTkFont(family="Segoe UI Semibold", size=12),
            text_color=renkler["gri"], anchor="w",
        ).pack(fill="x")
        self.kullanici_gir = GirisAlani(kart_ic, ipucu="TC Kimlik No", textvariable=self.uyg.kullanici_var)
        self.kullanici_gir.pack(fill="x", pady=(6, 16))

        # Sifre
        ctk.CTkLabel(
            kart_ic, text="🔐  Şifre",
            font=ctk.CTkFont(family="Segoe UI Semibold", size=12),
            text_color=renkler["gri"], anchor="w",
        ).pack(fill="x")
        self.sifre_gir = GirisAlani(kart_ic, ipucu="••••••••", gizli=True, textvariable=self.uyg.sifre_var)
        self.sifre_gir.pack(fill="x", pady=(6, 20))

        # Enter tusu ile giris
        self.sifre_gir.bind("<Return>", lambda e: self.uyg._giris_baslat())
        self.kullanici_gir.bind("<Return>", lambda e: self.sifre_gir.focus())

        # Giris butonu
        self.giris_btn = AnaButon(kart_ic, text="Bağlan", command=self.uyg._giris_baslat)
        self.giris_btn.pack(fill="x")

        # Durum
        self.durum = DurumRozeti(kart_ic)
        self.durum.pack(pady=(16, 0))

        # SSL uyarisi
        ctk.CTkLabel(
            self,
            text="🔓 SSL doğrulaması kapalı (captive portal gereksinimi)",
            font=ctk.CTkFont(family="Segoe UI", size=10),
            text_color=renkler["sari"],
        ).pack(pady=(0, 4))

        # Log butonu
        log_btn = ctk.CTkButton(
            self, text="📋 Sistem Günlüğü",
            font=ctk.CTkFont(size=12), height=36, corner_radius=10,
            fg_color=renkler["kart"], hover_color=renkler["kart2"],
            text_color=renkler["gri_koyu"],
            border_width=1, border_color=renkler["cizgi"],
            command=self.uyg._log_goster,
        )
        log_btn.pack(fill="x")

        # Uyari
        ctk.CTkLabel(
            self, text="⚠ Bu uygulama eğitim amaçlıdır.",
            font=ctk.CTkFont(family="Segoe UI", size=11),
            text_color=renkler["gri_koyu"],
        ).pack(pady=(16, 0))
