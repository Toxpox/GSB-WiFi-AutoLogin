"""Ozel widget'lar - buton, input ve durum rozeti."""

from __future__ import annotations

import customtkinter as ctk

from config import renkler


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
            **kw,
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
            **kw,
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
            **kw,
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

        self.yazi = ctk.CTkLabel(
            self.ic, text="Hazır", font=ctk.CTkFont(family="Segoe UI", size=11), text_color=renkler["gri_koyu"]
        )
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
        super().__init__(
            parent,
            fg_color=renkler["kart"],
            corner_radius=12,
            border_width=1,
            border_color=renkler["cizgi"],
            **kw,
        )

        ic = ctk.CTkFrame(self, fg_color="transparent")
        ic.pack(fill="both", expand=True, padx=12, pady=12)

        ctk.CTkLabel(ic, text=ikon, font=ctk.CTkFont(size=28)).pack(anchor="w")

        self.deger_lbl = ctk.CTkLabel(
            ic,
            text=deger,
            font=ctk.CTkFont(family="Segoe UI Semibold", size=14),
            text_color=renkler["beyaz"],
            anchor="w",
        )
        self.deger_lbl.pack(fill="x", pady=(8, 2))

        ctk.CTkLabel(
            ic,
            text=baslik,
            font=ctk.CTkFont(family="Segoe UI", size=11),
            text_color=renkler["gri_koyu"],
            anchor="w",
        ).pack(fill="x")

    def deger_ayarla(self, d):
        self.deger_lbl.configure(text=d)
