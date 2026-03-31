# Ozel hata tipleri - kullaniciya gosterilecek mesajlar buradan geliyor

from __future__ import annotations


class GSBHata(Exception):
    """Tum uygulama hatalarinin tabanı"""

    def __init__(self, mesaj: str, kullanici_mesaji: str | None = None):
        super().__init__(mesaj)
        self.kullanici_mesaji = kullanici_mesaji or mesaj


class AgHatasi(GSBHata):
    def __init__(self, mesaj: str = "Ag hatasi"):
        super().__init__(mesaj, "GSB WiFi agina bagli oldugunuzdan emin olun.")


class DNSHatasi(AgHatasi):
    """DNS cozumleme hatasi"""

    def __init__(self, host: str = ""):
        mesaj = f"DNS hatasi: {host}" if host else "DNS hatasi"
        super().__init__(mesaj)
        self.kullanici_mesaji = (
            "Sunucuya ulaşılamıyor. VPN aktifse devre dışı bırakın veya ağ yapılandırmanızı kontrol edin."
        )


class ZamanAsimi(AgHatasi):
    """Zaman asimi hatasi"""

    def __init__(self) -> None:
        super().__init__("Zaman asimi")
        self.kullanici_mesaji = "Sunucu yanitlamiyor. Ag yogunlugu nedeniyle gecikmeli olabilir."


class GirisBasarisiz(GSBHata):
    """Yanlis kimlik bilgileri"""

    def __init__(self, mesaj: str = "Giris basarisiz"):
        super().__init__(mesaj, "Kullanici adi veya sifrenizi kontrol edin.")


class MaksimumCihaz(GSBHata):
    """Maksimum cihaz limitine ulasildi"""

    def __init__(self, cihaz_bilgisi: dict | None = None, session=None, html: str = ""):
        super().__init__(
            "Maksimum cihaz limitine ulasildi",
            "Bu hesapla başka bir cihaz zaten bağlı.",
        )
        self.cihaz_bilgisi: dict = cihaz_bilgisi or {}
        self.session = session
        self.html = html


class PortalDegisti(GSBHata):
    """Portal HTML yapisi degismis"""

    def __init__(self) -> None:
        super().__init__(
            "Portal yapisi degismis",
            "Portal yapisi degismis olabilir. Uygulamayi guncellemeyi deneyin.",
        )
