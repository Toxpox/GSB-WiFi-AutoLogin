use crate::parser::KullaniciBilgi;
use crate::{config, errors::GSBError, network, parser};
use reqwest::Client;
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
                config::kullanici_kaydet(&kullanici, &sifre);
                let bilgi = parser::bilgi_cek(&html);
                Ok(GirisSonuc { bilgi, ip })
            }
            Err(GSBError::MaksimumCihaz { cihaz_bilgisi, html }) => {
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
    config::kullanici_kaydet(&kullanici, &sifre);
    let bilgi = parser::bilgi_cek(&yeni_html);
    let ip = network::ip_bul(&url).await.ok();
    Ok(GirisSonuc { bilgi, ip })
}

#[tauri::command]
pub fn tc_maskele(tc: String) -> String {
    config::tc_maskele(&tc)
}
