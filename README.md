# GSB WiFi Auto Login
<p>
  <img alt="Version" src="https://img.shields.io/badge/version-0.9.0-blue.svg?cacheSeconds=2592000" />
  <a href="https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE" target="_blank">
    <img alt="License: MIT License" src="https://img.shields.io/badge/License-MIT License-purple.svg" />
  </a>
</p>

Bu uygulama, KYK yurtlarında kullanılan internete otomatik giriş yapılmasını sağlar. Login ekranının yüklenmesini beklemenize gerek kalmaz. Kod yerel olarak çalışmakta olup herhangi bir şekilde verilerinizi dışarıya aktarmaz.

### 🏠 [Anasayfa](https://github.com/Toxpox/GSB-WiFi-AutoLogin)

## ⚠️ Güvenlik Uyarısı

Bu uygulama, yalnızca GSB/KYK captive portalı için tasarlanmıştır. Kimlik bilgilerinizin güvenliğini sağlamak adına, uygulamayı kişisel cihazınızda çalıştırın ve dosyaları üçüncü kişilerle paylaşmayın.

Eğitim amacıyla yapılmıştır.

## Özellikler

- Kullanıcı adı ve şifre girişi için sade grafik arayüzü
- IP bilgisi log ekranında görüntülenir
- Başarılı giriş sonrası kullanıcı bilgilerini görüntüler
- Tüm ağ işlemleri arka planda yürütülür

## 🔧 Gereksinimler

- Python 3.10 veya üzeri (Windows üzerinde test edildi)
- Aşağıdaki Python paketleri:
  - `requests`
  - `beautifulsoup4`
  - `urllib3`

Kurulum için:

```powershell
pip install -r requirements.txt
```

## Kullanım

```powershell
python .\gsb_autologin.py
```

1. Programı başlatmadan önce GSB WiFi ağında olduğunuzdan emin olun.
2. Açılan pencerede kullanıcı adınızı ve şifrenizi girin.
3. **Giriş Yap** tuşuna basın.
4. Log ekranında DNS çözümlemesi, kullanıcı bilgileri ve olası hataları takip edebilirsiniz.

> ⚠️ Yazılım eğitim ve otomasyon denemeleri amacıyla sağlanmıştır.


## Proje yapısı

```
v0.9.0/
├── gsb_auto.py
├── README.md
└── requirements.txt
```

## Sorun Giderme

- **DNS çözümlemesi başarısız**: Ağ bağlantınızı kontrol edin.
- **HTTP hatası 302/401**: Kullanıcı adı veya şifrenizi doğrulayın.

## 📝 Lisans

Copyright © 2025 [Toxpox](https://github.com/Toxpox).<br/>
This project is [MIT License](https://github.com/Toxpox/GSB-WiFi-AutoLogin/blob/main/LICENSE) licensed.

***
