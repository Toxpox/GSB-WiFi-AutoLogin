#!/usr/bin/env python3
"""Surum numarasinin tum kaynaklarda tutarli oldugunu dogrular.

Tek gercek kaynak `src-tauri/Cargo.toml`. Bu betik saf metin kontrolu
yaptigi icin platform bagimsizdir ve CI matrisinin her isletim sisteminde
ayni sekilde calisir.

Kontrol edilenler:
  - src-tauri/Cargo.lock  : proje paketinin kilitli surumu
  - frontend/js/app.js    : VERSION fallback degeri
  - README.md             : surum rozeti
  - CHANGELOG.md          : ilgili surum basligi (release notu buradan uretilir)
  - git tag               : etiketli kosumda vX.Y.Z eslesmesi
"""

import argparse
import os
import re
import sys
from pathlib import Path

KOK = Path(__file__).resolve().parent.parent


def cargo_surumu() -> str:
    metin = (KOK / "src-tauri" / "Cargo.toml").read_text(encoding="utf-8")
    eslesme = re.search(r'(?m)^version\s*=\s*"([^"]+)"', metin)
    if not eslesme:
        raise SystemExit("src-tauri/Cargo.toml icinde version bulunamadi.")
    return eslesme.group(1)


def kontrol_et(surum: str) -> list[str]:
    hatalar: list[str] = []

    kilit = (KOK / "src-tauri" / "Cargo.lock").read_text(encoding="utf-8")
    kilit_deseni = (
        r'\[\[package\]\]\s+name = "gsb-wifi-autologin"\s+version = "'
        + re.escape(surum)
        + '"'
    )
    if not re.search(kilit_deseni, kilit):
        hatalar.append(
            f"Cargo.lock proje paketi surumu {surum} ile uyusmuyor "
            "(cargo build calistirip lock dosyasini guncelleyin)."
        )

    app_js = (KOK / "frontend" / "js" / "app.js").read_text(encoding="utf-8")
    if f'let VERSION = "{surum}";' not in app_js:
        hatalar.append(f"frontend/js/app.js VERSION fallback degeri {surum} degil.")

    readme = (KOK / "README.md").read_text(encoding="utf-8")
    if f"version-{surum}-blue" not in readme:
        hatalar.append(f"README.md surum rozeti {surum} ile uyusmuyor.")

    changelog = (KOK / "CHANGELOG.md").read_text(encoding="utf-8")
    if not re.search(r"(?m)^##\s*\[" + re.escape(surum) + r"\]", changelog):
        hatalar.append(
            f"CHANGELOG.md icinde [{surum}] basligi yok. "
            "Release notu CHANGELOG'dan uretildigi icin bu zorunludur."
        )

    ref = os.environ.get("GITHUB_REF", "")
    if ref.startswith("refs/tags/v"):
        etiket = ref.removeprefix("refs/tags/")
        if etiket != f"v{surum}":
            hatalar.append(f"Git etiketi ({etiket}) proje surumu (v{surum}) ile uyusmuyor.")

    return hatalar


def main() -> int:
    ayristirici = argparse.ArgumentParser(description=__doc__)
    ayristirici.add_argument(
        "--yaz-github-output",
        action="store_true",
        help="Surumu GITHUB_OUTPUT dosyasina 'version' anahtariyla yazar",
    )
    args = ayristirici.parse_args()

    surum = cargo_surumu()
    hatalar = kontrol_et(surum)

    if hatalar:
        print(f"Surum tutarlilik kontrolu BASARISIZ (beklenen: {surum})", file=sys.stderr)
        for hata in hatalar:
            print(f"  - {hata}", file=sys.stderr)
        return 1

    print(f"Surum tutarli: {surum}")

    if args.yaz_github_output:
        cikti = os.environ.get("GITHUB_OUTPUT")
        if cikti:
            with open(cikti, "a", encoding="utf-8") as dosya:
                dosya.write(f"version={surum}\n")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
