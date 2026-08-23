#!/usr/bin/env python3
"""Release artefaktlarindan `latest.json` ve `SHA256SUMS.txt` uretir.

Onceki CI yalnizca `windows-x86_64` icin manifest yaziyordu; Linux ve macOS
istemcileri kendi platformlarini bulamadigi icin guncelleme kontrolu hata
veriyordu. Bu betik indirilen artefakt dizinini tarar ve bulunan her
updater-uyumlu paket icin dogru platform anahtarini yazar.

Updater notu: `.deb` ve `.rpm` uygulama ici updater ile guncellenemez, bu
yuzden manifeste alinmaz; onlarin guncellemesi paket yoneticisine aittir.
Linux'ta updater yalnizca AppImage uzerinden calisir.
"""

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

DEPO = "https://github.com/Toxpox/GSB-WiFi-AutoLogin"

# Dosya adi deseni -> Tauri updater platform anahtari.
# Sira onemli: ilk eslesen kazanir.
PLATFORM_DESENLERI = [
    (re.compile(r"_setup\.exe$", re.I), "windows-x86_64"),
    (re.compile(r"\.AppImage$", re.I), "linux-x86_64"),
    (re.compile(r"aarch64\.app\.tar\.gz$", re.I), "darwin-aarch64"),
    (re.compile(r"x86_64\.app\.tar\.gz$", re.I), "darwin-x86_64"),
]


def sha256(yol: Path) -> str:
    ozet = hashlib.sha256()
    with yol.open("rb") as dosya:
        for parca in iter(lambda: dosya.read(1024 * 1024), b""):
            ozet.update(parca)
    return ozet.hexdigest()


def platform_bul(ad: str) -> str | None:
    for desen, anahtar in PLATFORM_DESENLERI:
        if desen.search(ad):
            return anahtar
    return None


def main() -> int:
    ayristirici = argparse.ArgumentParser(description=__doc__)
    ayristirici.add_argument("--version", required=True, help="Surum, or. 1.10.0")
    ayristirici.add_argument("--dizin", required=True, help="Artefakt dizini")
    args = ayristirici.parse_args()

    surum = args.version.lstrip("vV")
    dizin = Path(args.dizin)
    if not dizin.is_dir():
        raise SystemExit(f"Dizin bulunamadi: {dizin}")

    platformlar: dict[str, dict[str, str]] = {}
    imzasiz: list[str] = []

    for dosya in sorted(dizin.iterdir()):
        if not dosya.is_file() or dosya.suffix == ".sig":
            continue

        anahtar = platform_bul(dosya.name)
        if anahtar is None:
            continue

        imza_yolu = dosya.with_name(dosya.name + ".sig")
        if not imza_yolu.is_file():
            imzasiz.append(dosya.name)
            continue

        imza = imza_yolu.read_text(encoding="utf-8").strip()
        if not imza:
            imzasiz.append(dosya.name)
            continue

        platformlar[anahtar] = {
            "signature": imza,
            "url": f"{DEPO}/releases/download/v{surum}/{dosya.name}",
        }

    if imzasiz:
        print(
            "HATA: su paketlerin updater imzasi yok veya bos: "
            + ", ".join(imzasiz)
            + "\nTAURI_SIGNING_PRIVATE_KEY secret'i tanimli mi?",
            file=sys.stderr,
        )
        return 1

    if "windows-x86_64" not in platformlar:
        # Windows kullanici tabaninin tamami tek bir endpoint'e bagli:
        # imzasiz/eksik bir manifest TUM Windows istemcilerinin guncelleme
        # kontrolunu kirar. Bu yuzden sert basarisizlik.
        print(
            "HATA: latest.json icinde windows-x86_64 yok. "
            "Yayinlanirsa mevcut Windows istemcileri guncelleme alamaz.",
            file=sys.stderr,
        )
        return 1

    manifest = {
        "version": f"v{surum}",
        "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "notes": f"GSB WiFi AutoLogin v{surum}. Ayrintilar icin CHANGELOG.md.",
        "platforms": platformlar,
    }

    manifest_yolu = dizin / "latest.json"
    # Updater BOM'suz UTF-8 bekler.
    manifest_yolu.write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(f"latest.json yazildi, platformlar: {', '.join(sorted(platformlar))}")

    satirlar = []
    for dosya in sorted(dizin.iterdir()):
        if dosya.is_file() and dosya.name != "SHA256SUMS.txt":
            satirlar.append(f"{sha256(dosya)}  {dosya.name}")

    (dizin / "SHA256SUMS.txt").write_text("\n".join(satirlar) + "\n", encoding="utf-8")
    print(f"SHA256SUMS.txt yazildi ({len(satirlar)} dosya).")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
