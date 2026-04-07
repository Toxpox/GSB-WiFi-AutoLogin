<div align="center">

# 🛜 GSB WiFi AutoLogin
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin.svg?type=shield&issueType=security)](https://app.fossa.com/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin?ref=badge_shield&issueType=security)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin.svg?type=shield&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin?ref=badge_shield&issueType=license)
<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-1.5.0-blue.svg?cacheSeconds=2592000&style=for-the-badge" />
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

- 🎨 **Modern Arayüz:** Dark tema tasarım ve akıcı animasyonlu UI.
- ⚡ **Otomatik Giriş:** Kullanıcı adınızı ve şifrenizi kaydederek tek tıkla ağa bağlanın.
- 👋 **Hoşgeldin Ekranı:** Başarılı girişten sonra kullanıcı bilgilerini ve anlık kota durumunu görün.
- 📊 **Kota Takibi:** Kalan kota, yüzde göstergesi ve kotanın yenilenme tarihi elinizin altında.
- 📜 **Sistem Günlüğü:** Tüm arka plan işlemlerini renkli log penceresiyle adım adım takip edin.
- 🔒 **Şifreleme:** AES-GCM teknolojisi ile kullanıcı bilgileriniz yüksek güvenlikle şifrelenir.
- 🚪 **Oturum Yönetimi:** İstediğiniz an güvenli çıkış yapma desteği.
- 🔄 **Yeniden Deneme:** Ağ hatalarında *exponential backoff* algoritması ile akıllı ve otomatik tekrar bağlantı.

---

## 📥 İndirme

En güncel sürümü **[Releases](https://github.com/Toxpox/GSB-WiFi-AutoLogin/releases)** sayfasından hemen indirebilirsiniz.

### 📦 Installer vs 🚀 Portable

| Özellik | 📦 Installer (`.exe` setup) | 🚀 Portable (`.exe`) |
|:---|:---|:---|
| **Kurulum** | Klasik kurulum sihirbazı ile sisteminize kurulur. | Kurulum gerektirmez, indirip doğrudan çalıştırın. |
| **Konum** | `AppData\Local` dizini altına yerleşir. | Herhangi bir klasörden veya USB bellekten çalışır. |
| **Başlat Menüsü** | Kısayol oluşturur, kolay erişim sağlar. | Kısayol oluşturmaz, bağımsızdır. |
| **Kaldırma** | Windows "Program Ekle/Kaldır" menüsünden kaldırılır. | Sadece dosyayı silmek yeterlidir. |
| **Güncelleme** | Yeni installer çalıştırılarak üzerine yazılır. | Eski dosya silinip yenisi ile değiştirilir. |

> 💡 **Not:** Her iki sürüm de tamamen aynı işlevi görür. Kullanım alışkanlığınıza göre tercih yapabilirsiniz.

---

## ⚠️ Güvenlik Uyarısı

Bu uygulama, **yalnızca GSB/KYK captive portali** için tasarlanmıştır. 

Kimlik bilgileriniz **sadece kendi bilgisayarınızda** `user_config.json` dosyasında **AES-GCM ile şifrelenerek** saklanır ve hiçbir şekilde dış sunuculara aktarılmaz. SSL doğrulaması, captive portal ağının doğası (yönlendirme gereksinimleri) nedeniyle devre dışı bırakılmıştır.

---

## 🛠️ Geliştirme

Projeyi kendi bilgisayarınızda derlemek veya geliştirmek isterseniz aşağıdaki adımları izleyebilirsiniz.

### Gereksinimler

- [Rust](https://rustup.rs/) *(stable sürüm)*
- [Node.js](https://nodejs.org/) *(opsiyonel, frontend bağımlılıkları için)*
- Windows 10 veya Windows 11

### Derleme Adımları

```powershell
# Depoyu klonlayın
git clone [https://github.com/Toxpox/GSB-WiFi-AutoLogin.git](https://github.com/Toxpox/GSB-WiFi-AutoLogin.git)

# Proje dizinine geçin
cd GSB-WiFi-AutoLogin/src-tauri

# Uygulamayı derleyin
cargo tauri build
```

📂 **Çıktı Yolları:**

- **Installer (Kurulum dosyası):** `src-tauri/target/release/bundle/nsis/`
- **Portable (.exe dosyası):** `src-tauri/target/release/`

---

## 💻 Teknolojiler

Bu projede kullanılan temel teknolojiler ve kullanım amaçları:

| Teknoloji | Kullanım Alanı |
|:---:|:---|
| ![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white) | Backend mantığı, ağ işlemleri ve şifreleme |
| ![Tauri v2](https://img.shields.io/badge/Tauri-FFC131?style=flat&logo=tauri&logoColor=white) | Modern masaüstü uygulama çerçevesi |
| ![HTML/CSS/JS](https://img.shields.io/badge/HTML5-E34F26?style=flat&logo=html5&logoColor=white) | Etkileşimli ve dinamik Frontend arayüzü |
| **AES-GCM** | Endüstri standardı kullanıcı bilgisi şifreleme |
| **reqwest** | Hızlı ve güvenilir HTTP istek/ağ işlemleri |

---

## 📄 Lisans

Copyright © 2025 **[Toxpox](https://github.com/Toxpox)**.  
Bu proje **[GNU General Public License v3.0 (GPLv3)](https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE)** ile lisanslanmıştır. Detaylar için `LICENSE` dosyasına göz atabilirsiniz.
