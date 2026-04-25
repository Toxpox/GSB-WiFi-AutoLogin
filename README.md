<div align="center">

# 🛜 GSB WiFi AutoLogin

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin.svg?type=shield&issueType=security)](https://app.fossa.com/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin?ref=badge_shield&issueType=security)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin.svg?type=shield&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin?ref=badge_shield&issueType=license)

<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-1.6.1-blue.svg?cacheSeconds=2592000&style=for-the-badge" />
  <a href="https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE" target="_blank">
    <img alt="License: GPLv3" src="https://img.shields.io/badge/License-GPLv3-blue.svg?style=for-the-badge" />
  </a>
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Windows-blue.svg?style=for-the-badge" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-Tauri%20v2-orange.svg?style=for-the-badge&logo=rust" />
</p>

**KYK yurtlarında kullanılan GSB WiFi ağına otomatik giriş yapan modern masaüstü uygulaması.**  
*Rust + Tauri v2 ile geliştirilmiştir.*

</div>

---

## ✨ Özellikler

- 🎨 **Modern Arayüz:** Koyu tema, kompakt giriş ekranı ve akıcı ekran geçişleri.
- ⚡ **Otomatik Giriş:** Kullanıcı adı ve şifre ile tek tıkla GSB WiFi captive portalına bağlanma.
- 👥 **Çoklu Profil:** Birden fazla kullanıcı hesabını yerelde kaydetme, seçme ve silme.
- 👁️ **Şifre Kontrolü:** Giriş ekranında şifreyi göster/gizle desteği.
- 👋 **Hoş Geldin Ekranı:** Başarılı girişten sonra kullanıcı, konum ve son giriş bilgilerini gösterme.
- 📊 **Kota Takibi:** Kalan kota, yüzde göstergesi, kullanılan kota ve yenilenme tarihi.
- 📜 **Sistem Günlüğü:** Giriş, çıkış, hata ve güncelleme kontrolü adımlarını log panelinden takip etme.
- 🔒 **Şifreleme:** Kullanıcı bilgilerini `user_config.json` içinde AES-GCM ile şifreli saklama.
- 🚪 **Oturum Yönetimi:** Aktif oturumu sonlandırma ve maksimum cihaz durumunda önceki oturumu düşürme.
- 🔄 **Yeniden Deneme:** Ağ hatalarında exponential backoff ile kontrollü tekrar deneme.
- 🧭 **GitHub Kısayolu:** Sağ üstteki GitHub butonu ile proje deposunu varsayılan tarayıcıda açma.
- 🆕 **Sürüm Kontrolü:** Başarılı bağlantıdan sonra GitHub Releases üzerinden yeni sürüm denetimi.

---

## 🆕 1.6.0 ile Gelenler

1.6.0 sürümü, `.old` klasöründeki 1.5.0 projesine göre özellikle hesap yönetimi ve güncelleme farkındalığına odaklanır.

| Alan | 1.5.0 | 1.6.0 |
|:---|:---|:---|
| Hesap kaydı | Tek kullanıcı kaydı | Çoklu profil listesi, seçim ve silme |
| Config formatı | Tek `username/password` kaydı | Geriye uyumlu çoklu profil formatı |
| Config konumu | Eski sürümde exe yanı dosya yaklaşımı | Yerel app data/config dizini, eski dosya okuma fallback'i |
| Kayıt hatası | Büyük ölçüde sessiz geçilebilir | Backend sonucu frontend'e taşınır ve log'a yazılır |
| Güncelleme | Manuel Releases kontrolü | Bağlantı sonrası GitHub Releases API kontrolü |
| GitHub erişimi | README linkleri | Uygulama içinde küçük GitHub butonu |
| UI | Sadece kullanıcı/şifre formu | Profil paneli, şifre göster/gizle, kaydırılabilir giriş ekranı |
| Test kapsamı | Daha dar birim testler | Profil migration, sürüm karşılaştırma ve güvenli GitHub URL testleri |

### Değerlendirme

1.6.0, 1.5.0'ın temel login akışını korurken kullanıcı deneyimini belirgin şekilde iyileştirir. En büyük kazanım, aynı bilgisayarı kullanan veya birden fazla hesabı yöneten kullanıcılar için profil sistemidir. GitHub sürüm kontrolü de kullanıcının eski installer ile uzun süre kalmasını azaltır.

Bakım tarafında kod yüzeyi büyüdü; özellikle frontend global script düzeni hâlâ dosya yükleme sırasına bağlı. Buna rağmen yeni backend komutları küçük ve sınırları belli tutuldu. Dış bağlantı açma komutu yalnızca proje GitHub URL'leriyle sınırlandığı için güvenlik açısından kontrollü bir genişleme yapıldı.

---

## 📥 İndirme

En güncel sürümü **[Releases](https://github.com/Toxpox/GSB-WiFi-AutoLogin/releases)** sayfasından indirebilirsiniz.

Uygulama başarılı bağlantıdan sonra GitHub Releases üzerinden yeni sürümü kontrol eder. Yeni sürüm bulunursa kullanıcıdan onay alarak release sayfasını açar.

### 📦 Installer vs 🚀 Portable

| Özellik | 📦 Installer (`.exe` setup) | 🚀 Portable (`.exe`) |
|:---|:---|:---|
| **Kurulum** | Klasik kurulum sihirbazı ile sisteminize kurulur. | Kurulum gerektirmez, indirip doğrudan çalıştırın. |
| **Konum** | `AppData\Local` dizini altına yerleşir. | Herhangi bir klasörden veya USB bellekten çalışır. |
| **Başlat Menüsü** | Kısayol oluşturur, kolay erişim sağlar. | Kısayol oluşturmaz, bağımsızdır. |
| **Kaldırma** | Windows "Program Ekle/Kaldır" menüsünden kaldırılır. | Sadece dosyayı silmek yeterlidir. |
| **Güncelleme** | Yeni installer çalıştırılarak üzerine yazılır. | Eski dosya silinip yenisi ile değiştirilir. |

> 💡 **Not:** Her iki sürüm de aynı uygulama mantığını kullanır. Kullanım alışkanlığınıza göre tercih yapabilirsiniz.

---

## ⚠️ Güvenlik Uyarısı

Bu uygulama **yalnızca GSB/KYK captive portali** için tasarlanmıştır.

Kimlik bilgileriniz sadece kendi bilgisayarınızda saklanır. Kayıtlı profiller `user_config.json` içinde AES-GCM ile şifrelenir ve hiçbir dış sunucuya gönderilmez. GitHub sürüm kontrolü yalnızca release bilgisi almak için GitHub API'ye istek atar; kullanıcı adı, şifre veya profil bilgisi bu isteğe eklenmez.

SSL doğrulaması, GSB captive portal akışının yönlendirme gereksinimleri nedeniyle portal istemcisinde devre dışıdır. GitHub sürüm kontrolü ise ayrı ve normal TLS doğrulamalı HTTP istemcisiyle yapılır.

---

## 🛠️ Geliştirme

Projeyi kendi bilgisayarınızda derlemek veya geliştirmek için:

### Gereksinimler

- [Rust](https://rustup.rs/) stable sürüm
- [Node.js](https://nodejs.org/) opsiyonel, frontend sözdizimi kontrolleri için
- Windows 10 veya Windows 11
- Tauri CLI (`cargo install tauri-cli --version "^2"` veya mevcut eşdeğer kurulum)

### Derleme Adımları

```powershell
# Depoyu klonlayın
git clone https://github.com/Toxpox/GSB-WiFi-AutoLogin.git

# Proje dizinine geçin
cd GSB-WiFi-AutoLogin/src-tauri

# Uygulamayı derleyin
cargo tauri build
```

📂 **Çıktı Yolları:**

- **Installer:** `src-tauri/target/release/bundle/nsis/`
- **Portable exe:** `src-tauri/target/release/`

### Kontrol Komutları

```powershell
cd src-tauri
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Frontend sözdizimi için:

```powershell
node --check frontend/js/app.js
node --check frontend/js/giris.js
```

---

## 💻 Teknolojiler

| Teknoloji | Kullanım Alanı |
|:---:|:---|
| ![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white) | Backend mantığı, ağ akışı, profil yönetimi ve şifreleme |
| ![Tauri v2](https://img.shields.io/badge/Tauri-FFC131?style=flat&logo=tauri&logoColor=white) | Windows masaüstü uygulama çerçevesi |
| ![HTML/CSS/JS](https://img.shields.io/badge/HTML5-E34F26?style=flat&logo=html5&logoColor=white) | Etkileşimli frontend arayüzü |
| **AES-GCM** | Yerel kullanıcı bilgisi şifreleme |
| **reqwest** | Captive portal, çıkış işlemi ve GitHub Releases API istekleri |
| **GitHub Releases API** | Yeni sürüm kontrolü |

---

## 📄 Lisans

Copyright © 2025 **[Toxpox](https://github.com/Toxpox)**.  
Bu proje **[GNU General Public License v3.0 (GPLv3)](https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE)** ile lisanslanmıştır. Detaylar için `LICENSE` dosyasına göz atabilirsiniz.
