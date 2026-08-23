#!/usr/bin/env python3
"""CHANGELOG.md icinden tek bir surumun govdesini cikarir.

Kullanim:
    changelog-bolum.py 1.10.0            -> govdeyi stdout'a yazar
    changelog-bolum.py v1.10.0 --kontrol -> yalnizca varligini dogrular

Release notu bu ciktidan uretilir, boylece surum notu tek kaynaktan
(CHANGELOG.md) yonetilir ve elle kopyalanmaz.
"""

import argparse
import re
import sys
from pathlib import Path

BASLIK = re.compile(r"^##\s*\[([^\]]+)\]")


def bolum_cikar(icerik: str, surum: str) -> str:
    hedef = surum.lstrip("vV")
    satirlar = icerik.splitlines()

    baslangic = None
    for i, satir in enumerate(satirlar):
        eslesme = BASLIK.match(satir)
        if eslesme and eslesme.group(1).lstrip("vV") == hedef:
            baslangic = i + 1
            break

    if baslangic is None:
        raise SystemExit(
            f"CHANGELOG.md icinde [{hedef}] basligi bulunamadi. "
            "Surum notu CHANGELOG'dan uretiliyor, once girdiyi ekleyin."
        )

    son = len(satirlar)
    for i in range(baslangic, len(satirlar)):
        if BASLIK.match(satirlar[i]):
            son = i
            break

    govde = "\n".join(satirlar[baslangic:son]).strip()
    if not govde:
        raise SystemExit(f"[{hedef}] basligi bos; surum notu uretilemez.")

    # Alt kisimdaki link tanimlarini ([1.9.0]: https://...) nota tasima.
    govde = re.sub(r"(?m)^\[[^\]]+\]:\s*http\S+\s*$", "", govde).strip()
    return govde


def main() -> int:
    ayristirici = argparse.ArgumentParser(description=__doc__)
    ayristirici.add_argument("surum", help="Surum numarasi, or. 1.10.0 veya v1.10.0")
    ayristirici.add_argument(
        "--dosya",
        default="CHANGELOG.md",
        help="CHANGELOG yolu (varsayilan: CHANGELOG.md)",
    )
    ayristirici.add_argument(
        "--kontrol",
        action="store_true",
        help="Ciktiyi yazma, yalnizca bolumun varligini dogrula",
    )
    args = ayristirici.parse_args()

    yol = Path(args.dosya)
    if not yol.is_file():
        raise SystemExit(f"Dosya bulunamadi: {yol}")

    govde = bolum_cikar(yol.read_text(encoding="utf-8"), args.surum)

    if args.kontrol:
        print(f"CHANGELOG [{args.surum}] bolumu bulundu ({len(govde)} karakter).")
        return 0

    sys.stdout.write(govde + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
