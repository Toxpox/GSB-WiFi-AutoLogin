"""Portal HTML'inden kullanici ve kota bilgisi ceker."""

from __future__ import annotations

from bs4 import BeautifulSoup

# Kota anahtar normalizasyonu (TR ve EN)
_KOTA_ANAHTAR: dict[str, str] = {
    # TR
    "Toplam Kota (MB)": "toplam_mb",
    "Toplam Kalan Kota (MB)": "kalan_mb",
    "Yenilenme Tarihi": "yenilenme",
    "Oturum Süresi": "oturum_suresi",
    "Login Zamanı": "login_zamani",
    "Başlangıç Tarihi": "baslangic",
    "Sona Erme Tarihi": "bitis",
    "Kalan Kota Zamanı": "kalan_zaman",
    # EN
    "Total Quota (MB)": "toplam_mb",
    "Total Remaining Quota (MB)": "kalan_mb",
    "Next Refresh Date": "yenilenme",
    "Session Time": "oturum_suresi",
    "Login Time": "login_zamani",
    "Start Date": "baslangic",
    "Expiration Date": "bitis",
    "Remaining Quota Time": "kalan_zaman",
}


def bilgi_cek(html: str) -> dict:
    """Portal'dan kullanici bilgilerini cek"""
    soup = BeautifulSoup(html, "html.parser")
    blok = soup.select_one("#content-div > center")

    bilgi: dict = {
        "isim": "Kullanıcı",
        "son_giris": "",
        "konum": "",
        "kota": {},
        "detaylar": [],
        "ham": "",
    }

    if not blok:
        bilgi["ham"] = "Bilgi yok"
        return bilgi

    # Isim: span.myinfo
    span = blok.select_one("span.myinfo")
    if span:
        txt = " ".join(span.get_text(separator=" ", strip=True).split())
        if txt:
            bilgi["isim"] = txt
            bilgi["detaylar"].append(txt)

    # Son Giris ve Konum: label.myinfo elementleri
    for lbl in blok.select("label.myinfo"):
        txt = " ".join(lbl.get_text(separator=" ", strip=True).split())
        if not txt:
            continue
        bilgi["detaylar"].append(txt)
        _alan_ayikla(bilgi, txt)

    # Portal bazen label'lardan myinfo class'ini kaldirabiliyor, o zaman tum label'lara bakiyoruz
    if not bilgi["konum"] or not bilgi["son_giris"]:
        for lbl in blok.find_all("label"):
            if "myinfo" in (lbl.get("class") or []):
                continue
            txt = " ".join(lbl.get_text(separator=" ", strip=True).split())
            if not txt:
                continue
            onceki_konum = bilgi["konum"]
            onceki_giris = bilgi["son_giris"]
            _alan_ayikla(bilgi, txt)
            if (bilgi["konum"] != onceki_konum or bilgi["son_giris"] != onceki_giris) and txt not in bilgi["detaylar"]:
                bilgi["detaylar"].append(txt)

    # Kota bilgileri
    bilgi["kota"] = _kota_cek(soup)

    bilgi["ham"] = "\n".join(bilgi["detaylar"]) if bilgi["detaylar"] else "Bilgi yok"
    return bilgi


def _alan_ayikla(bilgi: dict, txt: str) -> None:
    """Metin iceriginden son_giris ve konum alanlarini ayikla (TR ve EN)"""
    lower = txt.lower()

    # Son Giris / Last Login
    if not bilgi["son_giris"] and ("son giriş" in lower or "son giris" in lower or "last login" in lower):
        parts = txt.split(":", 1)
        if len(parts) > 1:
            bilgi["son_giris"] = parts[1].strip()

    # Konum / Location
    if not bilgi["konum"] and ("konum" in lower or "location" in lower):
        parts = txt.split(":", 1)
        if len(parts) > 1:
            bilgi["konum"] = parts[1].strip()


def maksimum_bilgi_cek(html: str) -> dict:
    """Maksimum cihaz sayfasindan bagli cihaz bilgilerini cek"""
    soup = BeautifulSoup(html, "html.parser")
    bilgi: dict = {"baslangic": "", "mac": "", "konum": ""}

    tablo_satir = soup.select_one("#j_idt20_data tr")
    if not tablo_satir:
        return bilgi

    hucreler = tablo_satir.find_all("td", role="gridcell")
    if len(hucreler) >= 3:
        bilgi["baslangic"] = hucreler[0].get_text(strip=True)
        bilgi["mac"] = hucreler[1].get_text(strip=True)
        bilgi["konum"] = hucreler[2].get_text(strip=True)

    return bilgi


def maksimum_form_bilgi_cek(html: str) -> dict:
    """Maksimum cihaz sayfasindan End butonu form bilgilerini cek"""
    soup = BeautifulSoup(html, "html.parser")
    bilgi: dict = {"form_id": "", "buton_id": "", "viewstate": ""}

    form = soup.select_one("#j_idt20_data tr form")
    if not form:
        return bilgi

    bilgi["form_id"] = form.get("id", "")

    buton = form.find("button", type="submit")
    if buton:
        bilgi["buton_id"] = buton.get("id", "")

    vs = form.find("input", attrs={"name": "javax.faces.ViewState"})
    if vs:
        bilgi["viewstate"] = vs.get("value", "")

    return bilgi


def _kota_cek(soup: BeautifulSoup) -> dict:
    """Kota panelinden bilgileri cek, normalize edilmis anahtarlarla dondur"""
    panel = soup.find(id="mainPanel:kotaDisplay")
    if not panel:
        return {}

    kota: dict = {}
    labels = panel.find_all("label", class_="ui-outputlabel")

    for label in labels:
        text = label.get_text(strip=True)
        if not text or text == "------":
            continue

        td = label.find_parent("td")
        if not td:
            continue

        next_td = td.find_next_sibling("td")
        if not next_td:
            continue

        val_label = next_td.find("label")
        if not val_label:
            continue

        val = val_label.get_text(strip=True)
        if not val or val == "------":
            continue

        key = text.rstrip(":")
        norm = _KOTA_ANAHTAR.get(key, key)
        kota[norm] = val

    return kota
