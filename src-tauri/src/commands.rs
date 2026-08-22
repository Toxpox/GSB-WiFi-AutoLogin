use crate::parser::KullaniciBilgi;
use crate::{config, errors::GSBError, network, parser};
use reqwest::Client;
use std::cmp::Ordering;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{Mutex, Notify};

pub struct AppState {
    pub client: Mutex<network::PortalClients>,
    pub giris_aktif: Mutex<bool>,
    pub son_html: Mutex<String>,

    pub son_kimlik: Mutex<Option<(String, String)>>,

    pub bekleyen_guncelleme: Mutex<Option<tauri_plugin_updater::Update>>,

    pub ag_olay: Arc<Notify>,
}

impl AppState {
    pub fn new() -> Result<Self, GSBError> {
        Ok(Self {
            client: Mutex::new(network::client_olustur()?),
            giris_aktif: Mutex::new(false),
            son_html: Mutex::new(String::new()),
            son_kimlik: Mutex::new(None),
            bekleyen_guncelleme: Mutex::new(None),
            ag_olay: Arc::new(Notify::new()),
        })
    }
}

#[derive(serde::Serialize)]
pub struct GirisSonuc {
    pub bilgi: KullaniciBilgi,
    pub ip: Option<String>,
    pub kayit_kaydedildi: bool,
    pub kayit_hatasi: Option<String>,
}

#[derive(serde::Serialize)]
pub struct AppBilgisi {
    pub version: &'static str,
    pub giris_url: &'static str,
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    name: Option<String>,
}

#[derive(serde::Serialize)]
pub struct VersiyonKontrolSonuc {
    pub guncel: bool,
    pub mevcut: &'static str,
    pub son: Option<String>,
    pub release_url: Option<String>,
    pub release_adi: Option<String>,
}

fn giris_sonuc_olustur(
    bilgi: KullaniciBilgi,
    ip: Option<String>,
    kayit: Result<(), GSBError>,
) -> GirisSonuc {
    match kayit {
        Ok(()) => GirisSonuc {
            bilgi,
            ip,
            kayit_kaydedildi: true,
            kayit_hatasi: None,
        },
        Err(e) => GirisSonuc {
            bilgi,
            ip,
            kayit_kaydedildi: false,
            kayit_hatasi: Some(e.to_string()),
        },
    }
}

fn portal_url_dogrula(url: &str) -> Result<(), GSBError> {
    let gecerli = url::Url::parse(url)
        .map(|u| u.scheme() == "https" && u.host_str() == Some(config::PORTAL_HOST))
        .unwrap_or(false);
    if gecerli {
        Ok(())
    } else {
        Err(GSBError::AgHatasi {
            mesaj: format!("Gecersiz portal adresi: {}", url),
            kullanici_mesaji: "Giris adresi GSB portali degil.".into(),
        })
    }
}

#[tauri::command]
pub async fn giris(
    app: AppHandle,
    url: String,
    kullanici: String,
    sifre: String,
    state: State<'_, AppState>,
) -> Result<GirisSonuc, String> {
    portal_url_dogrula(&url).map_err(|e: GSBError| -> String { e.into() })?;
    {
        let mut aktif = state.giris_aktif.lock().await;
        if *aktif {
            return Err("Giris zaten devam ediyor".into());
        }
        *aktif = true;
    }

    let sonuc = async {
        {
            let mut client = state.client.lock().await;
            *client = network::client_olustur()?;
        }
        let client = state.client.lock().await.normal.clone();

        match network::giris_yap(&client, &url, &kullanici, &sifre).await {
            Ok(yanit) => {
                let network::GirisYaniti { html, ip } = yanit;
                *state.son_html.lock().await = html.clone();
                *state.son_kimlik.lock().await = Some((kullanici.clone(), sifre.clone()));
                let bilgi = parser::bilgi_cek(&html);
                let kayit = config::kullanici_kaydet(&kullanici, &sifre);
                Ok(giris_sonuc_olustur(bilgi, ip, kayit))
            }
            Err(GSBError::MaksimumCihaz {
                cihaz_bilgisi,
                html,
            }) => {
                *state.son_html.lock().await = html;
                Err(GSBError::MaksimumCihaz {
                    cihaz_bilgisi,
                    html: String::new(),
                })
            }
            Err(e) => Err(e),
        }
    }
    .await;

    *state.giris_aktif.lock().await = false;
    if let Ok(s) = &sonuc {
        tepsi_ipucu_guncelle(&app, "Bağlı");
        kota_bildirimi_isle(&app, &s.bilgi);
        kota_gecmisi_isle(&s.bilgi);
    }
    sonuc.map_err(|e: GSBError| e.into())
}

#[tauri::command]
pub async fn cikis(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let clients = state.client.lock().await.clone();
    let basarili = network::cikis_yap(&clients)
        .await
        .map_err(|e: GSBError| -> String { e.into() })?;
    if basarili {
        *state.son_html.lock().await = String::new();
        *state.son_kimlik.lock().await = None;
        tepsi_ipucu_guncelle(&app, "Bağlı değil");
    }
    Ok(basarili)
}

#[tauri::command]
pub async fn bilgi_yenile(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<KullaniciBilgi, String> {
    let client = state.client.lock().await.normal.clone();
    let html = network::oturum_bilgisi_getir(&client)
        .await
        .map_err(|e: GSBError| e.to_string())?;
    *state.son_html.lock().await = html.clone();
    let bilgi = parser::bilgi_cek(&html);
    tepsi_ipucu_guncelle(&app, "Bağlı");
    kota_bildirimi_isle(&app, &bilgi);
    kota_gecmisi_isle(&bilgi);
    Ok(bilgi)
}

#[tauri::command]
pub fn kayitli_kullanici() -> (String, String) {
    config::kayitli_kullanici_al()
}

#[tauri::command]
pub fn profilleri_listele() -> Vec<config::KullaniciProfiliOzet> {
    config::profilleri_listele()
}

#[tauri::command]
pub fn profil_yukle(id: String) -> Result<(String, String), String> {
    config::profil_yukle(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn profil_sil(id: String) -> Result<Vec<config::KullaniciProfiliOzet>, String> {
    config::profil_sil(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn profil_takma_ad_ayarla(
    id: String,
    takma_ad: String,
) -> Result<Vec<config::KullaniciProfiliOzet>, String> {
    config::profil_takma_ad_ayarla(&id, &takma_ad).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn log_satiri_yaz(satir: String, tip: Option<String>) {
    crate::gunluk::yaz(tip.as_deref().unwrap_or("bilgi"), &satir);
}

#[tauri::command]
pub fn log_klasoru_ac() -> Result<(), String> {
    let dizin = config::log_dizini();

    std::fs::create_dir_all(&dizin).map_err(|e| format!("Log klasoru olusturulamadi: {}", e))?;
    klasor_ac(&dizin).map_err(|e| format!("Klasor acilamadi: {}", e))
}

fn klasor_ac(yol: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut komut = Command::new("explorer");

    #[cfg(target_os = "macos")]
    let mut komut = Command::new("open");

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let mut komut = Command::new("xdg-open");

    komut.arg(yol).spawn().map(|_| ())
}

#[tauri::command]
pub fn app_bilgisi() -> AppBilgisi {
    AppBilgisi {
        version: config::VERSION,
        giris_url: config::GIRIS_URL,
    }
}

#[tauri::command]
pub fn github_ac() -> Result<(), String> {
    harici_link_ac(config::GITHUB_URL).map_err(|e| format!("GitHub baglantisi acilamadi: {}", e))
}

#[tauri::command]
pub fn github_link_ac(url: String) -> Result<(), String> {
    if !guvenli_github_url(&url) {
        return Err("Sadece proje GitHub baglantilari acilabilir.".into());
    }
    harici_link_ac(&url).map_err(|e| format!("GitHub baglantisi acilamadi: {}", e))
}

#[tauri::command]
pub async fn yeni_versiyon_kontrol() -> Result<VersiyonKontrolSonuc, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent(config::USER_AGENT)
        .build()
        .map_err(|e| format!("Guncelleme istemcisi olusturulamadi: {}", e))?;

    let yanit = client
        .get(config::GITHUB_RELEASE_LATEST_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("GitHub surum bilgisi alinamadi: {}", e))?;

    if !yanit.status().is_success() {
        return Err(format!("GitHub yaniti basarisiz: {}", yanit.status()));
    }

    let release: GithubRelease = yanit
        .json()
        .await
        .map_err(|e| format!("GitHub surum bilgisi okunamadi: {}", e))?;

    if !guvenli_github_url(&release.html_url) {
        return Err("GitHub release baglantisi guvenli degil.".into());
    }

    let karsilastirma = surum_karsilastir(&release.tag_name, config::VERSION)
        .ok_or_else(|| "GitHub surum etiketi okunamadi.".to_string())?;

    Ok(VersiyonKontrolSonuc {
        guncel: karsilastirma != Ordering::Greater,
        mevcut: config::VERSION,
        son: Some(release.tag_name),
        release_url: Some(release.html_url),
        release_adi: release.name,
    })
}

#[derive(serde::Serialize)]
pub struct GuncellemeBilgisi {
    pub surum: String,
}

#[derive(serde::Serialize, Clone)]
struct GuncellemeIlerleme {
    yuzde: Option<u8>,
    indirilen_mb: f64,
}

fn kurulu_uygulama_mi() -> bool {
    let exe = match std::env::current_exe() {
        Ok(yol) => yol,
        Err(_) => return false,
    };
    dirs::data_local_dir()
        .map(|yerel| exe.starts_with(yerel))
        .unwrap_or(false)
}

#[tauri::command]
pub async fn guncelleme_kontrol(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<GuncellemeBilgisi>, String> {
    use tauri_plugin_updater::UpdaterExt;

    if !kurulu_uygulama_mi() {
        return Err("Portable kullanimda otomatik guncelleme desteklenmiyor.".into());
    }

    let updater = app
        .updater()
        .map_err(|e| format!("Guncelleyici baslatilamadi: {}", e))?;
    let guncelleme = updater
        .check()
        .await
        .map_err(|e| format!("Guncelleme kontrolu basarisiz: {}", e))?;

    match guncelleme {
        Some(g) => {
            let surum = g.version.clone();
            *state.bekleyen_guncelleme.lock().await = Some(g);
            Ok(Some(GuncellemeBilgisi { surum }))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn guncelleme_kur(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let guncelleme = state
        .bekleyen_guncelleme
        .lock()
        .await
        .take()
        .ok_or_else(|| "Once guncelleme kontrolu yapilmali.".to_string())?;

    let handle = app.clone();
    let mut indirilen: u64 = 0;
    guncelleme
        .download_and_install(
            move |parca, toplam| {
                indirilen += parca as u64;
                let yuzde = toplam
                    .filter(|t| *t > 0)
                    .map(|t| ((indirilen as f64 / t as f64) * 100.0).min(100.0) as u8);
                let _ = handle.emit(
                    "guncelleme-ilerleme",
                    GuncellemeIlerleme {
                        yuzde,
                        indirilen_mb: indirilen as f64 / (1024.0 * 1024.0),
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|e| format!("Guncelleme indirilemedi veya kurulamadi: {}", e))
}

fn harici_link_ac(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut komut = {
        let mut komut = Command::new("rundll32");
        komut.arg("url.dll,FileProtocolHandler");
        komut
    };

    #[cfg(target_os = "macos")]
    let mut komut = Command::new("open");

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let mut komut = Command::new("xdg-open");

    komut.arg(url).spawn().map(|_| ())
}

fn guvenli_github_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };

    parsed.scheme() == "https"
        && parsed.host_str() == Some("github.com")
        && parsed.path().starts_with("/Toxpox/GSB-WiFi-AutoLogin")
}

fn surum_karsilastir(son: &str, mevcut: &str) -> Option<Ordering> {
    Some(surum_parcala(son)?.cmp(&surum_parcala(mevcut)?))
}

fn surum_parcala(surum: &str) -> Option<[u64; 3]> {
    let temiz = surum.trim().trim_start_matches(['v', 'V']);
    let ana = temiz.split(['-', '+']).next()?;
    let mut parcalar = ana.split('.');

    Some([
        parcalar.next()?.parse().ok()?,
        parcalar.next()?.parse().ok()?,
        parcalar.next()?.parse().ok()?,
    ])
}

#[tauri::command]
pub async fn maksimum_cihaz_isle(
    app: AppHandle,
    url: String,
    kullanici: String,
    sifre: String,
    state: State<'_, AppState>,
) -> Result<GirisSonuc, String> {
    portal_url_dogrula(&url).map_err(|e: GSBError| -> String { e.into() })?;
    {
        let mut aktif = state.giris_aktif.lock().await;
        if *aktif {
            return Err("Giris zaten devam ediyor".into());
        }
        *aktif = true;
    }

    let sonuc = async {
        let html = state.son_html.lock().await.clone();
        let mevcut_client = state.client.lock().await.normal.clone();

        network::onceki_oturumu_kapat(&mevcut_client, &html, &url).await;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        {
            let mut client = state.client.lock().await;
            *client = network::client_olustur().map_err(|e: GSBError| -> String { e.into() })?;
        }
        let yeni_client = state.client.lock().await.normal.clone();

        let (yeni_html, ip) = match network::giris_yap(&yeni_client, &url, &kullanici, &sifre).await
        {
            Ok(yanit) => (yanit.html, yanit.ip),
            Err(GSBError::MaksimumCihaz { .. }) => {
                return Err(GSBError::GirisBasarisiz {
                    mesaj: "Maksimum cihaz limiti devam ediyor".into(),
                    kullanici_mesaji:
                        "Onceki cihazin baglantisi henuz dusmedi. Lutfen manuel tekrar deneyin."
                            .into(),
                }
                .into());
            }
            Err(e) => return Err(e.into()),
        };

        *state.son_html.lock().await = yeni_html.clone();
        *state.son_kimlik.lock().await = Some((kullanici.clone(), sifre.clone()));
        let bilgi = parser::bilgi_cek(&yeni_html);
        let kayit = config::kullanici_kaydet(&kullanici, &sifre);
        Ok(giris_sonuc_olustur(bilgi, ip, kayit))
    }
    .await;

    *state.giris_aktif.lock().await = false;
    if let Ok(s) = &sonuc {
        tepsi_ipucu_guncelle(&app, "Bağlı");
        kota_bildirimi_isle(&app, &s.bilgi);
        kota_gecmisi_isle(&s.bilgi);
    }
    sonuc
}

#[tauri::command]
pub fn tc_maskele(tc: String) -> String {
    config::tc_maskele(&tc)
}

#[tauri::command]
pub fn ayarlari_al() -> config::UygulamaAyarlari {
    config::ayarlari_oku()
}

#[tauri::command]
pub fn ayarlari_kaydet(app: AppHandle, ayarlar: config::UygulamaAyarlari) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    config::ayarlari_yaz(&ayarlar).map_err(|e: GSBError| -> String { e.into() })?;

    let otomatik = app.autolaunch();
    if ayarlar.baslangicta_calis {
        otomatik
            .enable()
            .map_err(|e| format!("Baslangicta calistirma ayarlanamadi: {}", e))?;
    } else {
        let _ = otomatik.disable();
    }
    Ok(())
}

#[tauri::command]
pub async fn gsb_aginda() -> bool {
    network::gsb_aginda_mi().await
}

#[derive(serde::Serialize)]
pub struct TaniSonuc {
    pub ad: String,

    pub durum: String,
    pub detay: String,
}

fn tani(ad: &str, durum: &str, detay: String) -> TaniSonuc {
    TaniSonuc {
        ad: ad.into(),
        durum: durum.into(),
        detay,
    }
}

async fn tcp_testi(ip: &str, port: u16) -> bool {
    let adr = if ip.contains(':') {
        format!("[{}]:{}", ip, port)
    } else {
        format!("{}:{}", ip, port)
    };
    matches!(
        tokio::time::timeout(Duration::from_secs(3), tokio::net::TcpStream::connect(&adr)).await,
        Ok(Ok(_))
    )
}

#[tauri::command]
pub async fn tani_calistir(state: State<'_, AppState>) -> Result<Vec<TaniSonuc>, String> {
    let mut sonuclar = Vec::new();

    let clients = state.client.lock().await.clone();
    let (ip, net, portal) = tokio::join!(
        network::ip_bul(config::GIRIS_URL),
        network::internet_var_mi(),
        network::portal_erisim_testi(&clients)
    );

    let gsb = match &ip {
        Ok(adr) => adr
            .parse::<std::net::IpAddr>()
            .map(|adres| network::ozel_ip_mi(&adres))
            .unwrap_or(false),
        Err(_) => false,
    };
    let tcp = match &ip {
        Ok(adr) => tcp_testi(adr, 443).await,
        Err(_) => false,
    };
    let gsb = gsb || tcp;

    sonuclar.push(tani(
        "GSB ağı",
        if gsb { "ok" } else { "hata" },
        if gsb {
            "GSB ağında görünüyorsunuz.".into()
        } else {
            "GSB ağı bulunamadı (DNS/TCP doğrulaması başarısız).".into()
        },
    ));

    match &ip {
        Ok(adr) => sonuclar.push(tani(
            "DNS çözümleme",
            "ok",
            format!("{} → {}", config::PORTAL_HOST, adr),
        )),
        Err(_) => sonuclar.push(tani(
            "DNS çözümleme",
            "hata",
            format!(
                "{} çözümlenemedi (VPN açıksa kapatın).",
                config::PORTAL_HOST
            ),
        )),
    }

    if let Ok(adr) = &ip {
        sonuclar.push(tani(
            "TCP bağlantısı (:443)",
            if tcp { "ok" } else { "hata" },
            if tcp {
                format!("{}:443 erişilebilir.", adr)
            } else {
                format!("{}:443 bağlantı kurulamadı.", adr)
            },
        ));
    }

    sonuclar.push(tani(
        "İnternet / oturum",
        if net { "ok" } else { "uyari" },
        if net {
            "İnternet erişimi var; portal oturumu açık görünüyor.".into()
        } else {
            "İnternet yok ya da captive portal arkasındasınız (giriş gerekebilir).".into()
        },
    ));

    match portal {
        Ok(kod) => sonuclar.push(tani(
            "Portal erişimi",
            if (200..400).contains(&kod) {
                "ok"
            } else {
                "uyari"
            },
            format!("{} → HTTP {}", config::INDEX_URL, kod),
        )),
        Err(e) => sonuclar.push(tani(
            "Portal erişimi",
            "hata",
            format!("Portala erişilemedi: {}", e),
        )),
    }

    let oturum = state.son_kimlik.lock().await.is_some();
    sonuclar.push(tani(
        "Kayıtlı oturum",
        if oturum { "ok" } else { "bilgi" },
        if oturum {
            "Bu oturumda başarılı giriş yapıldı; otomatik yeniden bağlanma etkin.".into()
        } else {
            "Henüz başarılı giriş yok; otomatik yeniden bağlanma bekliyor.".into()
        },
    ));

    Ok(sonuclar)
}

pub fn tepsi_ipucu_guncelle(app: &AppHandle, durum: &str) {
    if let Some(tepsi) = app.tray_by_id("ana-tepsi") {
        let _ = tepsi.set_tooltip(Some(format!("GSB WiFi AutoLogin — {}", durum)));
    }
}

const KOTA_DUSUK_ESIK: f64 = 0.20;

fn bildirim_gonder(app: &AppHandle, baslik: &str, govde: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title(baslik)
        .body(govde)
        .show();
}

fn kota_orani(bilgi: &KullaniciBilgi) -> Option<f64> {
    let kalan: f64 = bilgi.kota.get("kalan_mb")?.trim().parse().ok()?;
    let toplam: f64 = bilgi.kota.get("toplam_mb")?.trim().parse().ok()?;
    if toplam > 0.0 {
        Some(kalan / toplam)
    } else {
        None
    }
}

fn kota_bildirimi_sec(
    oran: Option<f64>,
    kota_doldu: bool,
    durum: config::BildirimDurumu,
) -> (Option<(String, String)>, config::BildirimDurumu) {
    let mut yeni = durum;

    if kota_doldu || matches!(oran, Some(o) if o <= 0.0) {
        if yeni.doldu_bildirildi {
            return (None, yeni);
        }
        yeni.doldu_bildirildi = true;
        yeni.dusuk_bildirildi = true;
        return (
            Some((
                "Kota Doldu".into(),
                "Aylık kotanız tükendi. Yenilenme tarihine kadar hız düşebilir.".into(),
            )),
            yeni,
        );
    }

    let Some(o) = oran else {
        return (None, yeni);
    };

    if o < KOTA_DUSUK_ESIK {
        if yeni.dusuk_bildirildi {
            return (None, yeni);
        }
        yeni.dusuk_bildirildi = true;
        (
            Some((
                "Kota Azalıyor".into(),
                format!("Kalan kotanız %{} seviyesine düştü.", (o * 100.0).round()),
            )),
            yeni,
        )
    } else {
        (None, config::BildirimDurumu::default())
    }
}

pub fn kota_bildirimi_isle(app: &AppHandle, bilgi: &KullaniciBilgi) {
    if !config::ayarlari_oku().kota_bildirim {
        return;
    }
    let durum = config::bildirim_durumu_oku();
    let (bildirim, yeni_durum) = kota_bildirimi_sec(kota_orani(bilgi), bilgi.kota_doldu, durum);
    if yeni_durum != durum {
        let _ = config::bildirim_durumu_yaz(&yeni_durum);
    }
    if let Some((baslik, govde)) = bildirim {
        bildirim_gonder(app, &baslik, &govde);
    }
}

pub fn kota_gecmisi_isle(bilgi: &KullaniciBilgi) {
    let kalan = bilgi
        .kota
        .get("kalan_mb")
        .and_then(|v| v.trim().parse::<f64>().ok());
    let toplam = bilgi
        .kota
        .get("toplam_mb")
        .and_then(|v| v.trim().parse::<f64>().ok());
    let (Some(kalan), Some(toplam)) = (kalan, toplam) else {
        return;
    };
    if toplam <= 0.0 {
        return;
    }
    let bugun = chrono::Local::now().format("%Y-%m-%d").to_string();
    let _ = config::kota_gecmisi_kaydet(kalan, toplam, &bugun);
}

#[tauri::command]
pub fn kota_gecmisi_al() -> Vec<config::KotaKaydi> {
    config::kota_gecmisi_oku()
}

#[derive(serde::Serialize, Clone)]
struct YenidenBaglanmaDurumu {
    tip: &'static str,
    mesaj: String,
}

fn yeniden_baglanma_bildir(app: &AppHandle, tip: &'static str, mesaj: String) {
    crate::gunluk::yaz(tip, &mesaj);
    let _ = app.emit("yeniden-baglanma", YenidenBaglanmaDurumu { tip, mesaj });
}

pub async fn yeniden_baglanma_dongusu(app: AppHandle) {
    let notify = app.state::<AppState>().ag_olay.clone();
    let periyot = Duration::from_secs(config::YENIDEN_BAGLAN_ARALIK_SAAT * 3600);

    loop {
        let olaydan = tokio::time::timeout(periyot, notify.notified())
            .await
            .is_ok();

        if olaydan {
            olay_firtinasini_yatistir(&notify).await;
        }

        yeniden_baglanmayi_dene(&app, olaydan).await;
    }
}

const AG_OLAY_SESSIZLIK_SN: u64 = 4;

async fn olay_firtinasini_yatistir(notify: &Notify) {
    let pencere = Duration::from_secs(AG_OLAY_SESSIZLIK_SN);
    while tokio::time::timeout(pencere, notify.notified())
        .await
        .is_ok()
    {}
}

async fn yeniden_baglanmayi_dene(app: &AppHandle, sessiz_aktif: bool) {
    if !config::ayarlari_oku().yeniden_baglan {
        return;
    }

    let state = app.state::<AppState>();

    let Some((kullanici, sifre)) = state.son_kimlik.lock().await.clone() else {
        return;
    };

    if !network::gsb_aginda_mi().await {
        return;
    }

    if network::internet_var_mi().await {
        let bugun = chrono::Local::now().format("%Y-%m-%d").to_string();
        if !config::kota_gecmisi_oku().iter().any(|k| k.tarih == bugun) {
            let client = state.client.lock().await.normal.clone();
            if let Ok(html) = network::oturum_bilgisi_getir(&client).await {
                kota_gecmisi_isle(&parser::bilgi_cek(&html));
            }
        }
        if !sessiz_aktif {
            yeniden_baglanma_bildir(app, "soluk", "Bağlantı kontrolü: oturum aktif.".to_string());
        }
        return;
    }

    {
        let mut aktif = state.giris_aktif.lock().await;
        if *aktif {
            return;
        }
        *aktif = true;
    }

    yeniden_baglanma_bildir(
        app,
        "uyari",
        "Bağlantı kontrolü: oturum düşmüş, yeniden bağlanılıyor…".to_string(),
    );

    let sonuc = async {
        {
            let mut client = state.client.lock().await;
            *client = network::client_olustur()?;
        }
        let client = state.client.lock().await.normal.clone();
        network::giris_yap(&client, config::GIRIS_URL, &kullanici, &sifre).await
    }
    .await;

    match sonuc {
        Ok(network::GirisYaniti { html, .. }) => {
            let bilgi = parser::bilgi_cek(&html);
            *state.son_html.lock().await = html;
            tepsi_ipucu_guncelle(app, "Bağlı");
            yeniden_baglanma_bildir(
                app,
                "basarili",
                "Oturum otomatik olarak yenilendi.".to_string(),
            );

            kota_bildirimi_isle(app, &bilgi);
            kota_gecmisi_isle(&bilgi);
        }
        Err(e) => {
            tepsi_ipucu_guncelle(app, "Bağlantı koptu");
            yeniden_baglanma_bildir(
                app,
                "hata",
                format!("Otomatik yeniden bağlanma başarısız: {}", e),
            );

            bildirim_gonder(
                app,
                "Bağlantı Koptu",
                "GSB WiFi oturumu yenilenemedi. Uygulamadan manuel bağlanmayı deneyin.",
            );
        }
    }

    *state.giris_aktif.lock().await = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surum_etiketi_parse_edilir() {
        assert_eq!(surum_parcala("v1.6.0"), Some([1, 6, 0]));
        assert_eq!(surum_parcala("1.6.0-beta.1"), Some([1, 6, 0]));
    }

    #[test]
    fn surumler_karsilastirilir() {
        assert_eq!(
            surum_karsilastir("v1.6.1", "1.6.0"),
            Some(Ordering::Greater)
        );
        assert_eq!(surum_karsilastir("v1.6.0", "1.6.0"), Some(Ordering::Equal));
        assert_eq!(surum_karsilastir("v1.5.9", "1.6.0"), Some(Ordering::Less));
    }

    #[test]
    fn kota_bildirimi_esik_altinda_bir_kez_gonderilir() {
        let temiz = config::BildirimDurumu::default();

        let (bildirim, durum) = kota_bildirimi_sec(Some(0.15), false, temiz);
        assert!(bildirim.is_some());
        assert!(durum.dusuk_bildirildi);

        let (bildirim, durum) = kota_bildirimi_sec(Some(0.10), false, durum);
        assert!(bildirim.is_none());

        let (bildirim, durum) = kota_bildirimi_sec(None, true, durum);
        assert_eq!(bildirim.unwrap().0, "Kota Doldu");
        let (bildirim, durum) = kota_bildirimi_sec(None, true, durum);
        assert!(bildirim.is_none());

        let (bildirim, durum) = kota_bildirimi_sec(Some(0.95), false, durum);
        assert!(bildirim.is_none());
        assert_eq!(durum, config::BildirimDurumu::default());
        let (bildirim, _) = kota_bildirimi_sec(Some(0.05), false, durum);
        assert!(bildirim.is_some());
    }

    #[test]
    fn kota_orani_hesaplanir() {
        let mut bilgi = KullaniciBilgi::default();
        bilgi.kota.insert("kalan_mb".into(), "5120".into());
        bilgi.kota.insert("toplam_mb".into(), "10240".into());
        assert_eq!(kota_orani(&bilgi), Some(0.5));

        bilgi.kota.insert("toplam_mb".into(), "0".into());
        assert_eq!(kota_orani(&bilgi), None);

        bilgi.kota.remove("kalan_mb");
        assert_eq!(kota_orani(&bilgi), None);
    }

    #[test]
    fn sadece_gsb_portal_urlsi_kabul_edilir() {
        assert!(portal_url_dogrula("https://wifi.gsb.gov.tr/j_spring_security_check").is_ok());
        assert!(portal_url_dogrula("http://wifi.gsb.gov.tr/j_spring_security_check").is_err());
        assert!(portal_url_dogrula("https://example.com/j_spring_security_check").is_err());
        assert!(portal_url_dogrula("https://wifi.gsb.gov.tr.evil.com/login").is_err());
        assert!(portal_url_dogrula("bozuk url").is_err());
    }

    #[test]
    fn sadece_proje_github_urlsi_acilir() {
        assert!(guvenli_github_url(
            "https://github.com/Toxpox/GSB-WiFi-AutoLogin/releases/tag/v1.6.0"
        ));
        assert!(!guvenli_github_url(
            "https://example.com/Toxpox/GSB-WiFi-AutoLogin"
        ));
        assert!(!guvenli_github_url(
            "http://github.com/Toxpox/GSB-WiFi-AutoLogin"
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn olay_firtinasi_son_olaydan_sonra_yatisir() {
        let notify = Arc::new(Notify::new());
        let pencere = Duration::from_secs(AG_OLAY_SESSIZLIK_SN);

        let uretici = notify.clone();
        tokio::spawn(async move {
            for _ in 0..5 {
                tokio::time::sleep(pencere / 2).await;
                uretici.notify_one();
            }
        });

        let baslangic = tokio::time::Instant::now();
        olay_firtinasini_yatistir(&notify).await;
        let gecen = baslangic.elapsed();

        assert!(
            gecen >= (pencere * 5) / 2 + pencere,
            "debounce son olaydan once dondu: {:?}",
            gecen
        );
    }

    #[tokio::test(start_paused = true)]
    async fn tek_olayda_yalnizca_bir_pencere_beklenir() {
        let notify = Arc::new(Notify::new());
        let baslangic = tokio::time::Instant::now();

        olay_firtinasini_yatistir(&notify).await;

        assert_eq!(
            baslangic.elapsed(),
            Duration::from_secs(AG_OLAY_SESSIZLIK_SN)
        );
    }
}
