#!/usr/bin/env python3
"""scripts/ altindaki release yardimcilarinin regresyon testleri.

Bu betikler release yolunun kritik parcasi: yanlis calisirlarsa ya release
yayinlanmaz ya da (daha kotusu) bozuk bir `latest.json` yayinlanip tum
istemcilerin guncelleme kontrolunu kirar. Bu yuzden testleri var.

Kosum:  python3 scripts/test_release_araclari.py
"""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

KOK = Path(__file__).resolve().parent.parent
BETIKLER = KOK / "scripts"

ORNEK_CHANGELOG = """# Changelog

Aciklama satiri.

## [1.10.0] - 2026-08-23

### Guvenlik
- TLS dogrulamasi acildi.

### Duzeltmeler
- Pencere boyutu duzeltildi.

## [1.9.1] - 2026-06-15

### Duzeltmeler
- Eski surum notu.

[1.10.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.9.1...v1.10.0
[1.9.1]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.9.0...v1.9.1
"""


def calistir(betik: str, *argumanlar: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(BETIKLER / betik), *argumanlar],
        capture_output=True,
        text=True,
    )


class ChangelogBolumTesti(unittest.TestCase):
    def setUp(self) -> None:
        self.gecici = tempfile.TemporaryDirectory()
        self.changelog = Path(self.gecici.name) / "CHANGELOG.md"
        self.changelog.write_text(ORNEK_CHANGELOG, encoding="utf-8")
        self.addCleanup(self.gecici.cleanup)

    def test_istenen_surum_govdesini_dondurur(self) -> None:
        sonuc = calistir("changelog-bolum.py", "1.10.0", "--dosya", str(self.changelog))
        self.assertEqual(sonuc.returncode, 0, sonuc.stderr)
        self.assertIn("TLS dogrulamasi acildi.", sonuc.stdout)
        self.assertIn("Pencere boyutu duzeltildi.", sonuc.stdout)

    def test_sonraki_surume_tasmaz(self) -> None:
        sonuc = calistir("changelog-bolum.py", "1.10.0", "--dosya", str(self.changelog))
        self.assertNotIn("Eski surum notu.", sonuc.stdout)
        self.assertNotIn("## [1.9.1]", sonuc.stdout)

    def test_v_oneki_kabul_edilir(self) -> None:
        sonuc = calistir("changelog-bolum.py", "v1.10.0", "--dosya", str(self.changelog))
        self.assertEqual(sonuc.returncode, 0, sonuc.stderr)
        self.assertIn("TLS dogrulamasi acildi.", sonuc.stdout)

    def test_link_tanimlari_nota_sizmaz(self) -> None:
        sonuc = calistir("changelog-bolum.py", "1.9.1", "--dosya", str(self.changelog))
        self.assertEqual(sonuc.returncode, 0, sonuc.stderr)
        self.assertNotIn("https://github.com", sonuc.stdout)

    def test_eksik_surum_hata_verir(self) -> None:
        sonuc = calistir("changelog-bolum.py", "9.9.9", "--dosya", str(self.changelog))
        self.assertNotEqual(sonuc.returncode, 0)
        self.assertIn("bulunamadi", sonuc.stderr)


class ReleaseManifestTesti(unittest.TestCase):
    def setUp(self) -> None:
        self.gecici = tempfile.TemporaryDirectory()
        self.dizin = Path(self.gecici.name)
        self.addCleanup(self.gecici.cleanup)

    def paket_yaz(self, ad: str, imzali: bool = True, imza: str = "IMZA") -> None:
        (self.dizin / ad).write_text("ikili icerik", encoding="utf-8")
        if imzali:
            (self.dizin / (ad + ".sig")).write_text(imza, encoding="utf-8")

    def tam_set_yaz(self) -> None:
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_setup.exe")
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_amd64.AppImage")
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_aarch64.app.tar.gz")
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_x86_64.app.tar.gz")
        # Updater kapsaminda olmayanlar:
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_portable.exe", imzali=False)
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_amd64.deb", imzali=False)
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_x86_64.rpm", imzali=False)

    def uret(self) -> subprocess.CompletedProcess:
        return calistir(
            "release-manifest.py", "--version", "1.10.0", "--dizin", str(self.dizin)
        )

    def test_tum_platformlar_manifeste_yazilir(self) -> None:
        self.tam_set_yaz()
        sonuc = self.uret()
        self.assertEqual(sonuc.returncode, 0, sonuc.stderr)

        manifest = json.loads((self.dizin / "latest.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest["version"], "v1.10.0")
        self.assertEqual(
            sorted(manifest["platforms"]),
            ["darwin-aarch64", "darwin-x86_64", "linux-x86_64", "windows-x86_64"],
        )

    def test_deb_ve_rpm_manifeste_alinmaz(self) -> None:
        # deb/rpm updater ile guncellenemez; manifeste girerlerse istemci
        # indirip kuramaz ve guncelleme kirilir.
        self.tam_set_yaz()
        self.uret()
        manifest = json.loads((self.dizin / "latest.json").read_text(encoding="utf-8"))
        urller = " ".join(p["url"] for p in manifest["platforms"].values())
        self.assertNotIn(".deb", urller)
        self.assertNotIn(".rpm", urller)

    def test_url_ve_imza_dogru_eslenir(self) -> None:
        self.tam_set_yaz()
        self.uret()
        manifest = json.loads((self.dizin / "latest.json").read_text(encoding="utf-8"))
        linux = manifest["platforms"]["linux-x86_64"]
        self.assertTrue(linux["url"].endswith("_amd64.AppImage"))
        self.assertIn("/releases/download/v1.10.0/", linux["url"])
        self.assertEqual(linux["signature"], "IMZA")

    def test_imzasiz_paket_basarisiz_olur(self) -> None:
        self.tam_set_yaz()
        (self.dizin / "GSB_WiFi_AutoLogin_1.10.0_amd64.AppImage.sig").unlink()
        sonuc = self.uret()
        self.assertNotEqual(sonuc.returncode, 0)
        self.assertIn("imzasi yok", sonuc.stderr)

    def test_bos_imza_basarisiz_olur(self) -> None:
        self.tam_set_yaz()
        (self.dizin / "GSB_WiFi_AutoLogin_1.10.0_setup.exe.sig").write_text(
            "   \n", encoding="utf-8"
        )
        sonuc = self.uret()
        self.assertNotEqual(sonuc.returncode, 0)

    def test_windows_eksikse_basarisiz_olur(self) -> None:
        # Tum Windows istemcileri tek endpoint'e bagli: windows anahtari
        # olmayan bir manifest yayinlanirsa hepsi guncelleme alamaz.
        self.paket_yaz("GSB_WiFi_AutoLogin_1.10.0_amd64.AppImage")
        sonuc = self.uret()
        self.assertNotEqual(sonuc.returncode, 0)
        self.assertIn("windows-x86_64", sonuc.stderr)

    def test_sha256sums_tum_dosyalari_kapsar(self) -> None:
        self.tam_set_yaz()
        self.uret()
        satirlar = (self.dizin / "SHA256SUMS.txt").read_text(encoding="utf-8").strip().splitlines()
        adlar = {satir.split("  ", 1)[1] for satir in satirlar}

        self.assertIn("GSB_WiFi_AutoLogin_1.10.0_amd64.deb", adlar)
        self.assertIn("GSB_WiFi_AutoLogin_1.10.0_portable.exe", adlar)
        self.assertIn("latest.json", adlar)
        self.assertNotIn("SHA256SUMS.txt", adlar)
        for satir in satirlar:
            self.assertRegex(satir, r"^[0-9a-f]{64}  ")


class SurumKontrolTesti(unittest.TestCase):
    def test_gercek_depo_tutarli(self) -> None:
        # Depo her zaman tutarli olmali; degilse CI zaten kirmiziya doner.
        sonuc = calistir("surum-kontrol.py")
        self.assertEqual(sonuc.returncode, 0, sonuc.stderr)
        self.assertIn("Surum tutarli", sonuc.stdout)


if __name__ == "__main__":
    unittest.main(verbosity=2)
