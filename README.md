# GSB WiFi AutoLogin

<p>
  <img alt="Version" src="https://img.shields.io/badge/version-1.5.0-blue.svg?cacheSeconds=2592000" />
  <a href="https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE" target="_blank">
  <img alt="License: GPLv3" src="https://img.shields.io/badge/License-GPLv3-blue.svg" />
  </a>
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Windows-blue.svg" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-Tauri%20v2-orange.svg" />
</p>

KYK yurtlarinda kullanilan GSB WiFi agina otomatik giris yapan modern masaustu uygulamasi. Rust + Tauri v2 ile gelistirilmistir.

## Ozellikler

- **Modern Arayuz:** Dark tema tasarim, animasyonlu UI
- **Otomatik Giris:** Kullanici adinizi kaydederek tek tikla giris yapin
- **Hosgeldin Ekrani:** Basarili giristen sonra kullanici bilgileri ve kota durumu
- **Kota Takibi:** Kalan kota, yuzde gostergesi ve yenilenme tarihi
- **Sistem Gunlugu:** Tum islemleri renkli log penceresiyle takip edin
- **Sifreleme:** AES-GCM ile kullanici bilgileri sifrelenir
- **Oturum Yonetimi:** Guvenli cikis yapma destegi
- **Yeniden Deneme:** Ag hatalarinda exponential backoff ile otomatik tekrar

## Indirme

[Releases](https://github.com/Toxpox/GSB-WiFi-AutoLogin/releases) sayfasindan indirebilirsiniz.

### Installer vs Portable

| | Installer (.exe setup) | Portable (.exe) |
|---|---|---|
| **Kurulum** | Klasik kurulum sihirbazi ile kurulur | Kurulum gerektirmez, dogrudan calistirilir |
| **Konum** | `Program Files` altina kurulur | Herhangi bir klasorden calisir (USB dahil) |
| **Baslat Menusu** | Kisayol olusturur | Kisayol olusturmaz |
| **Kaldirma** | Program Ekle/Kaldir'dan kaldirilir | Dosyayi silmek yeterlidir |
| **Guncelleme** | Yeni installer ile guncellenir | Yeni dosya ile degistirilir |

> Her iki surum de ayni islevi gorur. Tercih size kalmis.

## Guvenlik Uyarisi

Bu uygulama, yalnizca GSB/KYK captive portali icin tasarlanmistir. Kimlik bilgileriniz **sadece kendi bilgisayarinizda** `user_config.json` dosyasinda AES-GCM ile sifrelenerek saklanir ve disariya aktarilmaz. SSL dogrulamasi captive portal gereksinimi nedeniyle devre disidir.

## Gelistirme

### Gereksinimler

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (opsiyonel, frontend icin)
- Windows 10/11

### Derleme

```powershell
git clone https://github.com/Toxpox/GSB-WiFi-AutoLogin.git
cd GSB-WiFi-AutoLogin/src-tauri
cargo tauri build
```

Cikti: `src-tauri/target/release/bundle/nsis/` (installer) ve `src-tauri/target/release/` (portable exe)

## Teknolojiler

| Teknoloji | Kullanim |
|-----------|----------|
| **Rust** | Backend, ag islemleri, sifreleme |
| **Tauri v2** | Masaustu uygulama cercevesi |
| **HTML/CSS/JS** | Frontend arayuz |
| **AES-GCM** | Kullanici bilgileri sifreleme |
| **reqwest** | HTTP islemleri |

## Lisans

Copyright 2025 [Toxpox](https://github.com/Toxpox).
Bu proje [GNU General Public License v3.0 (GPLv3)](https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE) ile lisanslanmistir.
