# GSB WiFi Auto Login
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin?ref=badge_shield)

<p>
  <img alt="Version" src="https://img.shields.io/badge/version-0.9.9-blue.svg?cacheSeconds=2592000" />
  <a href="https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE" target="_blank">
  <img alt="License: MIT License" src="https://img.shields.io/badge/License-MIT License-purple.svg" />
  </a>
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Windows-blue.svg" />
</p>

Bu uygulama, KYK yurtlarında kullanılan GSB WiFi ağına otomatik giriş yapılmasını sağlayan modern bir masaüstü uygulamasıdır. Premium dark tema arayüzü ile kullanıcı bilgilerinizi kolayca takip edebilirsiniz.

### 🏠 [Anasayfa](https://github.com/Toxpox/GSB-WiFi-AutoLogin)

## ⚠️ Güvenlik Uyarısı

Bu uygulama, yalnızca GSB/KYK captive portalı için tasarlanmıştır. Kimlik bilgileriniz **sadece kendi bilgisayarınızda** `user_config.json` dosyasında saklanır ve dışarıya aktarılmaz. Uygulamayı güvenilir olmayan kaynaklardan indirmeyiniz.

## 📸 Ekran Görüntüsü

<div align="center">
<img src="src/LoginPage.png" alt="LoginPage" width="400" />
</div>

## ✨ Özellikler

- **Modern Arayüz:** CustomTkinter ile geliştirilmiş  dark tema tasarım
- **Otomatik Giriş:** Kullanıcı adınızı kaydederek tek tıkla giriş yapın
- **Hoşgeldin Ekranı:** Başarılı girişten sonra kullanıcı bilgilerini görüntüleme
- **Sistem Günlüğü:** Tüm işlemleri renkli log penceresiyle takip edin
- **Animasyonlu UI:** Durum göstergeleri ve focus efektleri
- **Oturum Yönetimi:** Güvenli çıkış yapma desteği

## 🚀 Kurulum ve Kullanım

### Hazır Çalıştırılabilir
1. [Releases](https://github.com/Toxpox/GSB-WiFi-AutoLogin/releases) sayfasından `GSB_AutoLogin.exe` dosyasını indirin
2. Bilgisayarınızın GSB WiFi ağına bağlı olduğundan emin olun
3. `GSB_AutoLogin.exe` dosyasını çalıştırın
4. Kullanıcı adı ve şifrenizi girerek **Bağlan** butonuna tıklayın

### Python ile Çalıştırma
```powershell
pip install -r requirements.txt
python gsb-autologin.py
```

## � Gereksinimler

```
beautifulsoup4
requests
urllib3
customtkinter>=5.2.0
Pillow>=10.0.0
```

## 🛠️ Derleme

Projeyi kendiniz derlemek isterseniz `PyInstaller` kullanabilirsiniz:

```powershell
pyinstaller --noconsole --onefile --name="GSB_AutoLogin" gsb-autologin.py
```

## 📂 Proje Yapısı

```
GSB-WiFi-AutoLogin/
├── src/
│   ├── gsb-autologin.py    # Ana uygulama (CustomTkinter UI)
│   ├── requirements.txt    # Python bağımlılıkları
│   ├── user_config.json    # Kullanıcı ayarları (otomatik oluşur)
│   └── LoginPage.png       # Ekran görüntüsü
├── LICENSE
└── README.md
```

## 🎨 Teknolojiler

- **Python**
- **CustomTkinter** - Modern Tkinter UI framework
- **Requests** - HTTP istekleri
- **BeautifulSoup4** - HTML parsing

## 📝 Lisans

Copyright © 2025 [Toxpox](https://github.com/Toxpox). 
Bu proje [MIT License](https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE) ile lisanslanmıştır.

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FToxpox%2FGSB-WiFi-AutoLogin?ref=badge_large)
