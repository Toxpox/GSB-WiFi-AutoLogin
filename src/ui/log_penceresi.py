"""Renkli sistem gunlugu penceresi."""

from __future__ import annotations

import customtkinter as ctk

from config import renkler


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

        ctk.CTkLabel(
            ust,
            text="📋 Sistem Günlüğü",
            font=ctk.CTkFont(family="Segoe UI Semibold", size=14),
            text_color=renkler["gri"],
        ).pack(side="left")

        ctk.CTkButton(
            ust,
            text="Temizle",
            font=ctk.CTkFont(size=11),
            width=64,
            height=26,
            corner_radius=8,
            fg_color="transparent",
            hover_color=renkler["kart2"],
            text_color=renkler["gri_koyu"],
            border_width=1,
            border_color=renkler["cizgi"],
            command=self.temizle,
        ).pack(side="right")

        self.alan = ctk.CTkTextbox(
            self,
            corner_radius=12,
            border_width=1,
            border_color=renkler["cizgi"],
            fg_color=renkler["kart"],
            text_color=renkler["gri"],
            font=ctk.CTkFont(family="Cascadia Code", size=12),
            wrap="word",
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
