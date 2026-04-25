use crate::parser::KullaniciBilgi;
use crate::{config, errors::GSBError, network, parser};
use reqwest::Client;
use std::cmp::Ordering;
use std::process::Command;
use std::time::Duration;
use tauri::State;
use tokio::sync::Mutex;

pub struct AppState {
    pub client: Mutex<Client>,
    pub giris_aktif: Mutex<bool>,
    pub son_html: Mutex<String>,
}

impl AppState {
    pub fn new() -> Result<Self, GSBError> {
        Ok(Self {
            client: Mutex::new(network::client_olustur()?),
            giris_aktif: Mutex::new(false),
            son_html: Mutex::new(String::new()),
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

#[tauri::command]
pub async fn giris(
    url: String,
    kullanici: String,
    sifre: String,
    state: State<'_, AppState>,
) -> Result<GirisSonuc, String> {
    {
        let mut aktif = state.giris_aktif.lock().await;
        if *aktif {
            return Err("Giris zaten devam ediyor".into());
        }
        *aktif = true;
    }

    let sonuc = async {
        // Her giris denemesinde temiz cookie jar ile basla.
        {
            let mut client = state.client.lock().await;
            *client = network::client_olustur()?;
        }
        let client = state.client.lock().await.clone();

        let ip = network::ip_bul(&url).await.ok();
        match network::giris_yap(&client, &url, &kullanici, &sifre).await {
            Ok(html) => {
                *state.son_html.lock().await = html.clone();
                let bilgi = parser::bilgi_cek(&html);
                let kayit = config::kullanici_kaydet(&kullanici, &sifre);
                Ok(giris_sonuc_olustur(bilgi, ip, kayit))
            }
            Err(GSBError::MaksimumCihaz {
                cihaz_bilgisi,
                html,
            }) => {
                // HTML'i sakla - onceki oturumu kapatmak icin gerekli
                *state.son_html.lock().await = html;
                Err(GSBError::MaksimumCihaz {
                    cihaz_bilgisi,
                    html: String::new(), // frontend'e gondermeye gerek yok
                })
            }
            Err(e) => Err(e),
        }
    }
    .await;

    *state.giris_aktif.lock().await = false;
    sonuc.map_err(|e: GSBError| e.into())
}

#[tauri::command]
pub async fn cikis(state: State<'_, AppState>) -> Result<bool, String> {
    let client = state.client.lock().await.clone();
    Ok(network::cikis_yap(&client).await)
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
    url: String,
    kullanici: String,
    sifre: String,
    state: State<'_, AppState>,
) -> Result<GirisSonuc, String> {
    let html = state.son_html.lock().await.clone();
    let mevcut_client = state.client.lock().await.clone();

    network::onceki_oturumu_kapat(&mevcut_client, &html, &url).await;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Eski oturum cerezlerini tasimamak icin ikinci denemeden once istemciyi yenile.
    {
        let mut client = state.client.lock().await;
        *client = network::client_olustur().map_err(|e: GSBError| -> String { e.into() })?;
    }
    let yeni_client = state.client.lock().await.clone();

    let yeni_html = match network::giris_yap(&yeni_client, &url, &kullanici, &sifre).await {
        Ok(html) => html,
        Err(GSBError::MaksimumCihaz { .. }) => {
            return Err(GSBError::GirisBasarisiz {
                mesaj: "Maksimum cihaz limiti devam ediyor".into(),
                kullanici_mesaji:
                    "Onceki cihazin baglantisi henuz dusmedi. Lutfen manuel tekrar deneyin.".into(),
            }
            .into());
        }
        Err(e) => return Err(e.into()),
    };

    *state.son_html.lock().await = yeni_html.clone();
    let bilgi = parser::bilgi_cek(&yeni_html);
    let ip = network::ip_bul(&url).await.ok();
    let kayit = config::kullanici_kaydet(&kullanici, &sifre);
    Ok(giris_sonuc_olustur(bilgi, ip, kayit))
}

#[tauri::command]
pub fn tc_maskele(tc: String) -> String {
    config::tc_maskele(&tc)
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
}
