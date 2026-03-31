"""HTTP katmani - giris, cikis, maksimum cihaz akisi."""

from __future__ import annotations

import contextlib
import random
import socket
import time
from collections.abc import Callable
from urllib.parse import urlparse

import requests
import urllib3

from config import (
    BACKOFF_CARPAN,
    BACKOFF_TABANI,
    CIKIS_URL,
    MAX_DENEME,
    TIMEOUT,
    USER_AGENT,
)
from errors import AgHatasi, DNSHatasi, GirisBasarisiz, MaksimumCihaz, ZamanAsimi

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

_HEADERS = {"User-Agent": USER_AGENT}


def ip_bul(url: str) -> str:
    """URL'den IP bul"""
    parsed = urlparse(url)
    host = parsed.hostname
    if not host:
        raise ValueError("URL hatali")
    try:
        return socket.gethostbyname(host)
    except socket.gaierror as e:
        raise DNSHatasi(host) from e


def giris_yap(
    url: str,
    kullanici: str,
    sifre: str,
    deneme_callback: Callable[[int, int], None] | None = None,
) -> tuple[requests.Session, str]:
    """
    Giris yap ve session dondur.

    Ag hatalarinda exponential backoff ile yeniden dener.
    Kimlik hatalarinda yeniden deneme yapmaz.
    """
    veri = {
        "j_username": kullanici,
        "j_password": sifre,
        "submit": "Giriş",
    }

    son_hata: Exception | None = None

    for deneme in range(1, MAX_DENEME + 1):
        try:
            if deneme_callback and deneme > 1:
                deneme_callback(deneme, MAX_DENEME)

            s = requests.Session()
            s.headers.update(_HEADERS)
            r = s.post(
                url,
                data=veri,
                allow_redirects=True,
                verify=False,
                timeout=TIMEOUT,
            )
            r.raise_for_status()

            # Portal 200 donduruyor ama URL bu string'i iceriyorsa cihaz limiti dolmus
            if "maksimumCihazHakkiDolu" in r.url:
                from parser import maksimum_bilgi_cek

                cihaz_bilgisi = maksimum_bilgi_cek(r.text)
                raise MaksimumCihaz(cihaz_bilgisi=cihaz_bilgisi, session=s, html=r.text)

            # Yanlis sifrede portal HTTP 200 donduruyor, redirect URL'den anliyoruz
            if "j_spring_security_check" in r.url:
                s.close()
                raise GirisBasarisiz("Kullanici adi veya sifre hatali")

            # Giris sayfasina geri yonlendirildiyse basarisiz
            if "login.html" in r.url:
                s.close()
                raise GirisBasarisiz("Kullanici adi veya sifre hatali")

            # URL temiz gorunse de sayfa bos gelebiliyor; ek kontrol
            if "content-div" not in r.text:
                s.close()
                raise GirisBasarisiz("Giris dogrulanamadi")

            return s, r.text

        except GirisBasarisiz:
            raise  # Kimlik hatasi icin yeniden deneme yapma

        except requests.HTTPError:
            raise  # HTTP hatasi icin yeniden deneme yapma

        except requests.Timeout:
            son_hata = ZamanAsimi()
            if deneme < MAX_DENEME:
                bekleme = BACKOFF_TABANI * (BACKOFF_CARPAN ** (deneme - 1)) + random.uniform(0, 1)
                time.sleep(bekleme)

        except requests.ConnectionError as e:
            son_hata = AgHatasi(str(e))
            if deneme < MAX_DENEME:
                bekleme = BACKOFF_TABANI * (BACKOFF_CARPAN ** (deneme - 1)) + random.uniform(0, 1)
                time.sleep(bekleme)

    if son_hata:
        raise son_hata
    raise AgHatasi("Baglanti kurulamadi")


def onceki_oturumu_kapat(session: requests.Session, html: str, login_url: str) -> bool:
    """Maksimum cihaz sayfasindaki onceki oturumu PrimeFaces AJAX ile sonlandir."""
    from parser import maksimum_form_bilgi_cek

    form_bilgi = maksimum_form_bilgi_cek(html)
    form_id = form_bilgi.get("form_id", "")
    buton_id = form_bilgi.get("buton_id", "")
    viewstate = form_bilgi.get("viewstate", "")

    if not form_id or not buton_id:
        return False

    parsed = urlparse(login_url)
    maksimum_url = f"{parsed.scheme}://{parsed.netloc}/maksimumCihazHakkiDolu.html"

    veri = {
        "javax.faces.partial.ajax": "true",
        "javax.faces.source": buton_id,
        "javax.faces.partial.execute": buton_id,
        "javax.faces.partial.render": "@all",
        buton_id: buton_id,
        form_id: form_id,
        "javax.faces.ViewState": viewstate,
    }
    hdrs = {**_HEADERS, "Faces-Request": "partial/ajax", "X-Requested-With": "XMLHttpRequest"}

    try:
        r = session.post(
            maksimum_url,
            data=veri,
            headers=hdrs,
            verify=False,
            timeout=TIMEOUT,
            allow_redirects=True,
        )
        return bool(r.status_code < 400)
    except (requests.RequestException, OSError):
        return False
    finally:
        with contextlib.suppress(Exception):
            session.close()


def cikis_yap(oturum: requests.Session | None) -> bool:
    """Oturumu kapat. Basarili ise True dondurur."""
    if not oturum:
        return False
    try:
        oturum.get(CIKIS_URL, verify=False, timeout=TIMEOUT, headers=_HEADERS)
        return True
    except (requests.RequestException, OSError):
        return False
    finally:
        with contextlib.suppress(Exception):
            oturum.close()
