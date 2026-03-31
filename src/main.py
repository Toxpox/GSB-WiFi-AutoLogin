"""
GSB WiFi AutoLogin - Giris Noktasi
====================================
Uygulamayi baslatir.
"""

import customtkinter as ctk

from ui.app import Uygulama


def main():
    ana = ctk.CTk()
    Uygulama(ana)
    ana.mainloop()


if __name__ == "__main__":
    main()
