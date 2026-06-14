# Changelog

Bu projedeki dikkate değer değişiklikler bu dosyada belgelenir.

Biçim [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) standardını,
sürümleme [Semantic Versioning](https://semver.org/lang/tr/) kurallarını takip eder.

## [1.9.0] - 2026-06-15

### Eklendi
- **Olay tabanlı yeniden bağlanma:** Windows IP arayüz değişikliği bildirimine (`NotifyIpInterfaceChange`) abone olunarak, Wi-Fi bağlandığı an oturum kontrolü yapılır; düşen GSB oturumu 12 saatlik aralığı beklemeden saniyeler içinde geri açılır. 12 saatlik anket güvenlik ağı olarak kalır. Kayıt başarısız olursa sessizce ankete düşülür. Olay tetiklemeli kontrollerde "oturum aktif" log satırı bastırılır (log kirliliği olmaz).
- **DPAPI ile kimlik şifreleme:** Kayıtlı kimlik bilgileri artık Windows DPAPI (`CryptProtectData`) ile şifrelenir; anahtar işletim sistemi tarafından oturum açan kullanıcıya bağlanır (`v3:` öneki). Eski AES-GCM (v2/v1) kayıtları okunmaya devam eder ve ilk girişte DPAPI'ye taşınır. DPAPI kullanılamazsa AES-GCM v2'ye düşülür.
- **Kota geçmişi ve tükenme tahmini:** Her başarılı girişte günlük kota anlık görüntüsü `kota_gecmisi.json` dosyasına kaydedilir (gün başına bir kayıt, 90 gün sınırı). Hoş geldin ekranında kota kartının altında inline SVG sparkline ve günlük tüketime göre "bu hızla ~N gün sonra biter" tahmini gösterilir (en az 2 günlük veri olunca; yenilenme atlamaları tahmine dahil edilmez). Harici grafik kütüphanesi yoktur.
- **Bağlantı tanılama paneli:** Tek tıkla aşamalı self-test — GSB ağı, DNS çözümleme, TCP :443, internet/captive oturum durumu, portal HTTP erişimi ve kayıtlı oturum — her biri başarılı/uyarı/hata olarak. Sessiz otomatik-giriş hatalarını somut teşhise çevirir; mevcut ağ fonksiyonlarının üstüne kurulur.
- **Bilgileri yenile:** Bağlı ekrandaki yenile butonu, yeniden giriş yapmadan aktif oturumla portaldan güncel kullanıcı/kota bilgisini çeker; kart ve kota grafiği tazelenir, kota geçmişi ve bildirimler de güncellenir. Oturum düşmüşse anlaşılır bir uyarı gösterilir.

### Değiştirildi
- Pencere artık tam ekrana geçemez: sabit boyut (420×680) yanında maximize/fullscreen kapatıldı ve F11 ile HTML Fullscreen API bloke edildi.

## [1.8.0] - 2026-06-12

### Eklendi
- **Sistem tepsisi:** Tepsi ikonu ve menüsü (Bağlan, Çıkış Yap, Pencereyi Göster, Uygulamadan Çık); pencere kapatılınca tepsiye küçülme (ayarlardan kapatılabilir); tepsi ipucunda bağlantı durumu.
- **Ayarlar paneli:** Otomatik giriş, tepsiye küçülme, Windows başlangıcında çalıştırma ve periyodik oturum kontrolü için kalıcı ayarlar (`settings.json`).
- **Otomatik giriş:** Açılışta aktif profille kendiliğinden bağlanma; hata durumunda modal yerine sessiz log akışı. Maksimum cihaz durumu her zaman kullanıcıya sorulur.
- **Windows başlangıcında çalıştırma:** `tauri-plugin-autostart` ile; uygulama `--sessiz` bayrağıyla pencere açmadan tepsiden başlar.
- **Otomatik yeniden bağlanma:** 12 saatte bir oturum canlılık kontrolü (Windows NCSI ucu); oturum düştüyse son kimlik bilgileriyle otomatik yeniden giriş. Çıkış yapılınca devre dışı kalır.
- **GSB ağı algılama:** Açılışta portal erişilebilirlik kontrolü; GSB ağında değilken log uyarısı, otomatik girişin ve yeniden bağlanmanın gereksiz denemeleri atlaması.
- **Kota bildirimleri:** Kalan kota %20'nin altına düşünce veya kota dolunca Windows bildirimi; aynı eşik için tekrar bildirim gönderilmez, kota yenilenince sıfırlanır. Başarısız otomatik yeniden bağlanmada da bildirim gönderilir. Ayarlardan kapatılabilir.
- **Profil isimlendirme:** Kayıtlı profillere takma ad verme (token üzerindeki kalem butonu); görünen ad takma ad, tooltip'te maskeli TC korunur. TC kimlik numarası takma ad olarak kabul edilmez; takma ad girişlerde korunur.
- **Uygulama içi otomatik güncelleme:** Açılışta arka planda güncelleme kontrolü (`tauri-plugin-updater`, imzalı `latest.json`); güncelleme varsa pencerenin üstünde yeşil bir bar belirir, tıklanınca indirme ilerlemesi gösterilir ve kurulum otomatik tamamlanıp uygulama yeniden başlar. Portable kullanımda bar GitHub release sayfasını açar.
- **Kalıcı sistem günlüğü:** Log satırları `logs/uygulama.log` dosyasına da yazılır (1 MB üstünde tek yedekli rotasyon); log panelinden "Klasörü Aç" ile erişilir. Backend olayları tepsideyken de dosyaya işlenir.

### Güvenlik
- Profil seçimi log satırındaki bir fallback'in ham TC kimlik numarasını gösterebilmesi düzeltildi (maskeli ada düşürüldü).

## [1.7.2] - 2026-06-10

### Güvenlik
- Giriş isteklerinin yalnızca `wifi.gsb.gov.tr` adresine gönderilmesini garanti eden backend tarafı URL doğrulaması eklendi.
- Şifreleme anahtarı artık MAC adresine bağlı değil; makine adı + işletim sistemi kullanıcı adından türetiliyor (v2). MAC rastgeleleştirme veya ağ adaptörü değişikliği kayıtlı profilleri bozmuyor. Eski (v1) anahtarla şifrelenmiş kayıtlar okunmaya devam eder ve ilk kayıtta yeni anahtarla yeniden şifrelenir.
- Çözülemeyen profil kayıtları artık şifreli metni kullanıcı adı gibi göstermiyor; listeden gizleniyor ve yüklenmeye çalışıldığında anlaşılır bir hata veriliyor.
- Kısa kullanıcı adlarında maskeleme daha az karakter açığa çıkarıyor.

### Düzeltmeler
- Profil dosyası (`user_config.json`) atomik yazılıyor; yazma sırasındaki kesinti dosyayı bozmuyor.
- Portal 5xx hatası döndürdüğünde yanıltıcı "Giriş doğrulanamadı" yerine yeniden deneme (retry/backoff) akışı çalışıyor.
- "Önceki oturumu düşür" akışı da eşzamanlı giriş kilidini kullanıyor; çifte istek engellendi.
- Çıkış isteği başarısız olduğunda kullanıcıya görünür bir uyarı gösteriliyor.
- Yeni konuma taşınan ayar dosyasının exe yanındaki eski kopyası temizleniyor.

### Bakım
- Sürüm numarası tek kaynaktan (Cargo.toml) yönetiliyor: `config.rs` `env!("CARGO_PKG_VERSION")` kullanıyor, `tauri.conf.json`'dan mükerrer `version` alanı kaldırıldı.
- Kullanılmayan `pbkdf2/simple` feature'ı kaldırıldı; bağımlılık ağacı küçüldü.
- CI'a Rust derleme cache'i ve Tauri CLI cache'i eklendi; derleme ve release süreleri kısaldı.
- `.gitignore` eklendi.

## [1.7.0] - 2026-05-31

- Kota dolduğunda kota kartının görünmemesi sorunu düzeltildi; kota dolu durumu artık %0 kalan olarak gösteriliyor.
- Arayüz tasarım optimizasyonları.

## [1.6.1] - 2026-04-26

- Çıkış (logout) akışı düzeltildi.

## [1.6.0] - 2026-04-25

- Çoklu profil desteği: birden fazla hesabı kaydetme, seçme ve silme.
- GitHub Releases üzerinden yeni sürüm kontrolü.
- CI/CD iyileştirmeleri.

## [1.5.0] - 2026-04-07

- Proje Rust + Tauri v2 ile yeniden yazıldı.
- AES-GCM ile yerel kimlik bilgisi şifreleme.
- Maksimum cihaz durumunda önceki oturumu düşürme.

## [1.0.0] - 2026-03-31

- İlk kararlı sürüm.

[1.8.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.7.2...v1.8.0
[1.7.2]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.7.0...v1.7.2
[1.7.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.6.1...v1.7.0
[1.6.1]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.6.0...v1.6.1
[1.6.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.0.0...v1.5.0
[1.0.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/releases/tag/v1.0.0
