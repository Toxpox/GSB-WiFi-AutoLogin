# Changelog

Bu projedeki dikkate değer değişiklikler bu dosyada belgelenir.

Biçim [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) standardını,
sürümleme [Semantic Versioning](https://semver.org/lang/tr/) kurallarını takip eder.

## [1.10.0] - 2026-08-23

Canlı GSB ağı üzerinde yapılan ölçümlere dayanan kapsamlı ağ, parser ve
güvenlik optimizasyonu turu. Kullanıcıya bakan davranış aynı kaldı; giriş akışı
daha hızlı, daha öngörülebilir ve daha güvenli hâle geldi. Test sayısı 27 → 63.

### Güvenlik
- **TLS doğrulaması açıldı:** Portal istemcisindeki `danger_accept_invalid_certs(true)` kaldırıldı. `wifi.gsb.gov.tr` artık normal sertifika doğrulamasından geçiyor; sertifika zinciri canlı ağda doğrulandı. Kimlik bilgileri artık doğrulanmamış bir TLS oturumu üzerinden gönderilemiyor.
- **Gövde boyutu sınırlandı:** Tüm yanıt gövdeleri akış sırasında ve decode sonrasında sınırlanıyor (portal 256 KiB, bağlantı kontrolü 1 KiB). Kötü niyetli veya bozuk bir yanıtın belleği doldurması engellendi.
- **Bağlantı kontrolü sahteciliğe kapatıldı:** Windows NCSI yanıtı artık `contains` yerine tam eşleşme ile doğrulanıyor; captive portalın beklenen metni kendi sayfasına gömerek "internet var" sonucu ürettirmesi mümkün değil.

### Eklendi
- **Aşama süresi ölçümü:** DNS, TCP bağlantısı, giriş denemesi, toplam giriş, gövde okuma, parse ve yeniden bağlanma süreleri yerel log dosyasına `anahtar=deger` biçiminde yazılıyor. Veri yalnızca cihazda kalır, hiçbir telemetri gönderilmez.
- **`GSB_VERI_DIZINI` ortam değişkeni:** Ayar, profil ve log dizini elle belirlenebiliyor (taşınabilir kurulum ve tanı için).
- **Mock HTTP sunucusu üzerinde entegrasyon testleri:** başarılı giriş, giriş formuna geri dönüş, gzip decode, decode sonrası limit aşımı, 503 sonrası yeniden deneme ve gövde sınırı senaryoları.
- **Linux CI işi:** `ubuntu-latest` üzerinde test + clippy; Windows dışı kolların derlenebilirliği artık CI'da denetleniyor. Masaüstü paketleme için kare PNG/icns/ico ikon seti eklendi.

### Değişti
- **Ağ istemcisi GSB portalına göre optimize edildi:** Merkezi portal (F5 BigIP + PrimeFaces/JSF) yalnızca HTTP/1.1 konuştuğu için istemci `http1_only` ile sabitlendi (gereksiz HTTP/2 ALPN denemesi kaldırıldı). Ardışık istekler (giriş → oturum doğrulama → kota) aynı TCP bağlantısını yeniden kullanacak şekilde bağlantı havuzu (`pool_idle_timeout`, `tcp_keepalive`) ve `tcp_nodelay` ayarlandı. Bu ayarlar merkezi portala dayalı olduğu için tüm GSB lokasyonlarında geçerlidir; MTU/gecikme gibi konuma özgü değerlere dokunulmadı.
- **Giriş akışına 25 saniyelik global bütçe:** Denemeler, gövde okuma ve oturum doğrulama isteği kalan bütçeye sarıldı. Kötü durumda ölçülen 33 saniyelik zincir artık bütçeyi aşamıyor. Zamanlama sabitleri arasındaki tutarlılık (`LOGIN_BUTCE_SECS >= TIMEOUT_SECS` vb.) derleme zamanında zorlanıyor.
- **Zaman aşımları ayrıştırıldı:** Toplam sürenin yanında ayrı `connect_timeout` (4 sn) ve `read_timeout` (8 sn); DNS ve TCP testlerinin kendi bütçeleri var.
- **gzip sıkıştırma açıldı:** Portal yanıtları sıkıştırılmış aktarılıyor.
- **Tanılama paralelleştirildi:** DNS bir kez çözülüyor ve sonuç GSB ağı kararı, IP gösterimi ve TCP testi arasında paylaşılıyor; DNS, internet ve portal kontrolleri eşzamanlı çalışıyor. Tanı başına DNS çözümlemesi 3 → 1.
- **GSB ağı algılamada adres yarışı:** IPv4/IPv6 aileleri Happy Eyeballs benzeri şekilde yarıştırılıyor; adres başına değil tek global TCP bütçesi uygulanıyor.
- **Açılış hızlandırıldı:** Uygulama bilgisi, ayarlar ve profil yüklemesi paralel çekiliyor (3 seri IPC turu → 1). Aktif profil değişmediyse profil dosyası yeniden yazılmıyor.
- **Yeniden bağlanmada trailing debounce:** 45 saniyelik leading cooldown yerine 4 saniyelik sessizlik penceresi; ağ olayı fırtınasının son olayı artık bastırılmıyor.
- **Sayfa sınıflandırma semantikleşti:** Ham `body.contains("content-div")` kontrolleri kaldırıldı; giriş formu, kimlik/kota/çıkış sinyalleri ve gerçek DOM düğümü birlikte değerlendiriliyor. Giriş formuna geri düşme ayrı ve anlaşılır bir hata mesajı üretiyor.
- **Kırılgan CSS seçicilerine yedek zinciri:** JSF ID'leri değiştiğinde de kota, cihaz listesi ve buton alanları ayıklanabiliyor. Tüm seçiciler bir kez derlenip yeniden kullanılıyor.
- **Maksimum cihaz akışı sadeleşti:** Cihaz ve form bilgisi tek DOM parse'ından geliyor (2 → 1 parse). Sabit 2 saniyelik bekleme yerine oturum düşer düşmez çıkan yoklama (8 sn bütçe).
- **Durum yönetimi sadeleşti:** ~20 KB'lik ham HTML state'i kaldırıldı; eşzamanlı giriş kilidi `Mutex<bool>` yerine RAII semaphore permit ile yönetiliyor, bayrağın takılı kalma riski ortadan kalktı.

### Düzeltmeler
- **Yeniden deneme güvenliği:** Deneme sonuçları "güvenli tekrar" (bağlantı kurulamadı, POST gitmedi), "belirsiz" (timeout/gövde) ve "kesin" (semantik) olarak sınıflandırılıyor. Belirsiz sonuçta kör POST tekrarı yerine oturum doğrulama isteği yapılıyor; kesin hatada döngü kırılıyor. Aynı girişin iki kez işlenmesi riski giderildi.
- **Kota ve alan ayıklama kayıpları:** Türkçe büyük/küçük harf dönüşümü (`SON GİRİŞ` gibi alanların kaybı), blok düzenine bağımlı erken dönüş ve kota anahtarı eşleştirmesi düzeltildi. Kota dolu tespiti artık ham HTML yerine görünür DOM metnini tarıyor (`script`, `style`, gizli düğümler kapsam dışı).
- **Çıkış/oturum doğrulaması:** Yönlendirme hedefi son yol bileşeninden (`;jsessionid` destekli) ve JSF partial-response gövdesindeki gerçek `<redirect url="...">` değerinden okunuyor.
- **Önceki oturumu düşürme sonucu kontrol ediliyor:** Sessizce başarısız olabilen istek artık doğrulanıyor.
- **Giriş öncesi gereksiz DNS çağrısı kritik yoldan çıkarıldı:** IP bilgisi kurulan bağlantının kendisinden alınıyor.
- **Pencereye sığmayan arayüz:** Sabit 420×680 pencere, Windows'tan büyük font metriklerine sahip ortamlarda (ör. Linux) içeriği taşırıyor ve alt buton çubuğunu (`Çıkış Yap` / `Bağlan`) kesiyordu. Pencere 440×760'a büyütüldü ve sınırlı biçimde yeniden boyutlandırılabilir yapıldı (min 420×620, max 640×1100; maximize/fullscreen hâlâ kapalı). Ayrıca ekran içeriği kaydırılabilir bir gövdeye alındı: marka satırı ve alt çubuk her zaman görünür kalıyor, arada kalan içerik sığmazsa kaydırılıyor. Böylece taşma font, tema veya ölçek farkından bağımsız olarak yapısal olarak engellendi.
- **Giriş ekranındaki yanlış güvenlik etiketi:** Altbilgideki "SSL kapalı · captive portal" yazısı, TLS doğrulaması bu sürümde açıldığı için artık doğru değildi; "TLS doğrulamalı · captive portal" olarak güncellendi ve uyarı noktası sarıdan yeşile çevrildi.

### Bakım
- `cargo test` artık gerçek kullanıcı veri dizinine yazmıyor; test derlemesinde veri dizini geçici dizine yönleniyor (derleme zamanı garantisi) ve iki koruma testiyle korunuyor.
- Yeni testlerin gerçekten koruduğu, kasıtlı hata enjeksiyonuyla (mutasyon testi) doğrulandı.
- Kaynak dosyalardaki yorum satırları temizlendi; her commit ayrı worktree'de derlenip test edilerek geçmişin bisect edilebilirliği doğrulandı.
- Release binary artışı +12.680 bayt (%0,154) ile sınırlı kaldı.

**Kapsam notu:** Bu turdaki doğrulamalar Linux x86_64 üzerinde yapıldı; Windows'a
özgü kollar (DPAPI, `NotifyIpInterfaceChange`, NSIS paketleme) CI'ın
`windows-latest` işinde derlenir. Zaman bütçesi sabitleri saha ölçümüyle
ayarlanacak başlangıç değerleridir.

## [1.9.1] - 2026-06-15

### Düzeltmeler
- **Güncelleme butonu tıklanamıyordu:** "Yeni sürüm" bildirimi tam genişlikte bir üst bardı ve ekran katmanının (`.ekran`) altında kaldığı için hem yazılarla çakışıyor hem de tıklanamıyordu. Artık bağlı ekranda, GitHub butonunun solunda küçük yeşil bir buton; tıklanabilir, indirme yüzdesi/durumu tooltip'te gösterilir.
- **Kullanım grafiği gözükmüyordu:** İki yönlü düzeltildi — (1) yeterli veri yokken kart gizlenmek yerine "Grafik birkaç günlük kullanım verisiyle oluşur." bilgisini gösteriyor; (2) oturum açıkken günde en fazla bir kez kota anlık görüntüsü kaydedilerek, sürekli bağlı kalan kullanıcılarda da grafik verisi birikiyor (grafik en az 2 farklı günlük veriyle çizilir).

### Bakım
- Bağımlılıklar güncellendi (Tauri 2.10 → 2.11, `windows` 0.58 → 0.61 ve ~80 paket semver-uyumlu); `cargo audit` ile güvenlik denetimi yapıldı (0 güvenlik açığı).

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

[1.10.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.9.1...v1.10.0
[1.9.1]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.9.0...v1.9.1
[1.9.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.8.0...v1.9.0
[1.8.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.7.2...v1.8.0
[1.7.2]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.7.0...v1.7.2
[1.7.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.6.1...v1.7.0
[1.6.1]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.6.0...v1.6.1
[1.6.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/compare/v1.0.0...v1.5.0
[1.0.0]: https://github.com/Toxpox/GSB-WiFi-AutoLogin/releases/tag/v1.0.0
