"""Basarili giris sonrasi gelen ekran - kota ve kullanici bilgileri."""

from __future__ import annotations

import customtkinter as ctk

from config import renkler
from ui.widgets import IkinciButon


class HosgeldinEkrani(ctk.CTkFrame):
    def __init__(self, parent, uygulama, **kw):
        super().__init__(parent, fg_color="transparent", **kw)
        self.uyg = uygulama
        self._olustur()

    def _olustur(self):
        # Scrollable content
        self.icerik = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.icerik.pack(fill="both", expand=True)

        # Basari basligi
        ust = ctk.CTkFrame(self.icerik, fg_color="transparent")
        ust.pack(fill="x", pady=(10, 0))

        ikon_kutu = ctk.CTkFrame(
            ust, width=70, height=70, corner_radius=35,
            fg_color=renkler["yesil_koyu"], border_width=3, border_color=renkler["yesil"],
        )
        ikon_kutu.pack()
        ikon_kutu.pack_propagate(False)
        ctk.CTkLabel(
            ikon_kutu, text="✓",
            font=ctk.CTkFont(size=36, weight="bold"),
            text_color=renkler["yesil"],
        ).place(relx=0.5, rely=0.5, anchor="center")

        ctk.CTkLabel(
            ust, text="Hoş Geldiniz!",
            font=ctk.CTkFont(family="Segoe UI", size=26, weight="bold"),
            text_color=renkler["beyaz"],
        ).pack(pady=(14, 4))

        self.isim_lbl = ctk.CTkLabel(
            ust, text="",
            font=ctk.CTkFont(family="Segoe UI", size=16),
            text_color=renkler["mor_acik"],
        )
        self.isim_lbl.pack()

        # Baglanti durumu
        durum_kutu = ctk.CTkFrame(ust, fg_color=renkler["yesil_koyu"], corner_radius=20, height=32)
        durum_kutu.pack(pady=(12, 0))
        ctk.CTkLabel(
            durum_kutu, text="● Bağlantı Aktif",
            font=ctk.CTkFont(family="Segoe UI Semibold", size=12),
            text_color=renkler["yesil"],
        ).pack(padx=16, pady=6)

        # Kullanici bilgi karti
        self.bilgi_kart = ctk.CTkFrame(
            self.icerik, fg_color=renkler["kart"], corner_radius=16,
            border_width=1, border_color=renkler["cizgi"],
        )

        self.bilgi_ic = ctk.CTkFrame(self.bilgi_kart, fg_color="transparent")
        self.bilgi_ic.pack(fill="x", padx=20, pady=14)

        ctk.CTkLabel(
            self.bilgi_ic, text="👤 Kullanıcı Bilgileri",
            font=ctk.CTkFont(family="Segoe UI Semibold", size=13),
            text_color=renkler["gri"], anchor="w",
        ).pack(fill="x")

        self.son_giris_lbl = ctk.CTkLabel(
            self.bilgi_ic, text="",
            font=ctk.CTkFont(family="Segoe UI", size=13),
            text_color=renkler["beyaz"], anchor="w", justify="left",
        )

        self.konum_lbl = ctk.CTkLabel(
            self.bilgi_ic, text="",
            font=ctk.CTkFont(family="Segoe UI", size=13),
            text_color=renkler["beyaz"], anchor="w", justify="left",
        )

        # Kota karti
        self.kota_kart = ctk.CTkFrame(
            self.icerik, fg_color=renkler["kart"], corner_radius=16,
            border_width=1, border_color=renkler["cizgi"],
        )

        self.kota_ic = ctk.CTkFrame(self.kota_kart, fg_color="transparent")
        self.kota_ic.pack(fill="x", padx=20, pady=14)

        ctk.CTkLabel(
            self.kota_ic, text="📊 Kota Bilgileri",
            font=ctk.CTkFont(family="Segoe UI Semibold", size=13),
            text_color=renkler["gri"], anchor="w",
        ).pack(fill="x")

        self.kota_bar = ctk.CTkProgressBar(
            self.kota_ic, height=14, corner_radius=7,
            fg_color=renkler["input_bg"], progress_color=renkler["mor"],
        )

        self.kota_yuzde_lbl = ctk.CTkLabel(
            self.kota_ic, text="",
            font=ctk.CTkFont(family="Segoe UI Semibold", size=13),
            text_color=renkler["beyaz"], anchor="w",
        )

        self.kota_detay_lbl = ctk.CTkLabel(
            self.kota_ic, text="",
            font=ctk.CTkFont(family="Segoe UI", size=12),
            text_color=renkler["gri"], anchor="w", justify="left",
        )

        # Butonlar
        btn_alan = ctk.CTkFrame(self, fg_color="transparent")
        btn_alan.pack(fill="x", side="bottom", pady=(10, 0))

        ctk.CTkButton(
            btn_alan, text="📋 Sistem Günlüğü",
            font=ctk.CTkFont(size=12), height=36, corner_radius=10,
            fg_color="transparent", hover_color=renkler["kart"],
            text_color=renkler["gri_koyu"],
            command=self.uyg._log_goster,
        ).pack(fill="x", pady=(0, 8))

        IkinciButon(btn_alan, text="Çıkış Yap", command=self.uyg._cikis_yap).pack(fill="x")

    def bilgi_goster(self, bilgi):
        isim = bilgi.get("isim", "Kullanıcı")
        self.isim_lbl.configure(text=isim)

        # Kullanici bilgi karti
        son_giris = bilgi.get("son_giris", "")
        konum = bilgi.get("konum", "")

        bilgi_var = son_giris or konum
        if bilgi_var:
            self.bilgi_kart.pack(fill="x", pady=(16, 0), in_=self.icerik)

            if son_giris:
                self.son_giris_lbl.configure(text=f"Son Giriş:  {son_giris}")
                self.son_giris_lbl.pack(fill="x", pady=(8, 0))
            else:
                self.son_giris_lbl.pack_forget()

            if konum:
                self.konum_lbl.configure(text=f"Konum:  {konum}")
                self.konum_lbl.pack(fill="x", pady=(4, 0))
            else:
                self.konum_lbl.pack_forget()
        else:
            self.bilgi_kart.pack_forget()

        # Kota karti (normalize edilmis anahtarlar)
        kota = bilgi.get("kota", {})
        toplam_str = kota.get("toplam_mb", "")
        kalan_str = kota.get("kalan_mb", "")

        if toplam_str and kalan_str:
            self.kota_kart.pack(fill="x", pady=(10, 0), in_=self.icerik)

            try:
                toplam = float(toplam_str)
                kalan = float(kalan_str)
                kullanilan = toplam - kalan
                oran = kalan / toplam if toplam > 0 else 0
            except ValueError:
                toplam, kalan, kullanilan, oran = 0, 0, 0, 0

            # Progress bar
            self.kota_bar.pack(fill="x", pady=(10, 0))
            self.kota_bar.set(oran)

            # Renk: yesil > %50, sari > %20, kirmizi <= %20
            if oran > 0.5:
                self.kota_bar.configure(progress_color=renkler["yesil"])
            elif oran > 0.2:
                self.kota_bar.configure(progress_color=renkler["sari"])
            else:
                self.kota_bar.configure(progress_color=renkler["kirmizi"])

            # Kalan / Toplam
            kalan_gb = kalan / 1024
            toplam_gb = toplam / 1024
            self.kota_yuzde_lbl.configure(
                text=f"{kalan_gb:.1f} GB / {toplam_gb:.1f} GB  ({oran:.0%})"
            )
            self.kota_yuzde_lbl.pack(fill="x", pady=(6, 0))

            # Detaylar
            detay_satirlar = []
            yenilenme = kota.get("yenilenme", "")
            if yenilenme:
                detay_satirlar.append(f"Yenilenme:  {yenilenme}")
            kullanilan_gb = kullanilan / 1024
            detay_satirlar.append(f"Kullanılan:  {kullanilan_gb:.1f} GB")

            self.kota_detay_lbl.configure(text="\n".join(detay_satirlar))
            self.kota_detay_lbl.pack(fill="x", pady=(4, 0))
        else:
            self.kota_kart.pack_forget()
