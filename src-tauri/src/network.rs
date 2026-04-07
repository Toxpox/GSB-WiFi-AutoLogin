use crate::config::*;
use crate::errors::*;
use crate::parser;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub fn client_olustur() -> Result<Client, GSBError> {
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .user_agent(USER_AGENT)
        .cookie_store(true)
        .build()
        .map_err(|e| GSBError::AgHatasi {
            mesaj: e.to_string(),
            kullanici_mesaji: "HTTP istemcisi olusturulamadi".into(),
        })
}

pub async fn ip_bul(url: &str) -> Result<String, GSBError> {
    let parsed: url::Url = url.parse().map_err(|_| GSBError::DNSHatasi {
        host: url.to_string(),
        kullanici_mesaji: "URL hatali".into(),
    })?;
    let host = parsed.host_str().unwrap_or("");
    let addr = tokio::net::lookup_host(format!("{}:443", host))
        .await
        .map_err(|_| GSBError::DNSHatasi {
            host: host.to_string(),
            kullanici_mesaji: "Sunucuya ulasilamiyor. VPN aktifse devre disi birakin.".into(),
        })?
        .next()
        .ok_or_else(|| GSBError::DNSHatasi {
            host: host.to_string(),
            kullanici_mesaji: "DNS cozumlenemedi".into(),
        })?;
    Ok(addr.ip().to_string())
}

pub async fn giris_yap(
    client: &Client,
    url: &str,
    kullanici: &str,
    sifre: &str,
) -> Result<String, GSBError> {
    let veri = [
        ("j_username", kullanici),
        ("j_password", sifre),
        ("submit", "Giriş"),
    ];

    let mut son_hata: Option<GSBError> = None;

    for deneme in 1..=MAX_DENEME {
        match client.post(url).form(&veri).send().await {
            Ok(r) => {
                let final_url = r.url().to_string();
                let body = r.text().await.unwrap_or_default();

                if final_url.contains("maksimumCihazHakkiDolu") {
                    let cihaz = parser::maksimum_bilgi_cek(&body);
                    return Err(GSBError::MaksimumCihaz {
                        cihaz_bilgisi: cihaz,
                        html: body,
                    });
                }
                if final_url.contains("j_spring_security_check")
                    || final_url.contains("login.html")
                {
                    return Err(GSBError::GirisBasarisiz {
                        mesaj: "Yanlis kimlik".into(),
                        kullanici_mesaji: "Kullanici adi veya sifrenizi kontrol edin.".into(),
                    });
                }
                if !body.contains("content-div") {
                    return Err(GSBError::GirisBasarisiz {
                        mesaj: "Dogrulanamadi".into(),
                        kullanici_mesaji: "Giris dogrulanamadi".into(),
                    });
                }
                return Ok(body);
            }
            Err(e) if e.is_timeout() => {
                son_hata = Some(GSBError::ZamanAsimi);
            }
            Err(e) => {
                son_hata = Some(GSBError::AgHatasi {
                    mesaj: e.to_string(),
                    kullanici_mesaji: "GSB WiFi agina bagli oldugunuzdan emin olun.".into(),
                });
            }
        }

        if deneme < MAX_DENEME {
            let bekleme =
                BACKOFF_TABANI * BACKOFF_CARPAN.powi((deneme - 1) as i32) + rand::random::<f64>();
            tokio::time::sleep(Duration::from_secs_f64(bekleme)).await;
        }
    }

    Err(son_hata.unwrap_or(GSBError::AgHatasi {
        mesaj: "Baglanti kurulamadi".into(),
        kullanici_mesaji: "GSB WiFi agina bagli oldugunuzdan emin olun.".into(),
    }))
}

pub async fn cikis_yap(client: &Client) -> bool {
    client.get(CIKIS_URL).send().await.is_ok()
}

pub async fn onceki_oturumu_kapat(client: &Client, html: &str, login_url: &str) -> bool {
    let form = parser::maksimum_form_bilgi_cek(html);
    if form.form_id.is_empty() || form.buton_id.is_empty() {
        return false;
    }

    let parsed: url::Url = match login_url.parse() {
        Ok(u) => u,
        Err(_) => return false,
    };
    let maks_url = format!(
        "{}://{}/maksimumCihazHakkiDolu.html",
        parsed.scheme(),
        parsed.host_str().unwrap_or("")
    );

    let veri = [
        ("javax.faces.partial.ajax", "true"),
        ("javax.faces.source", &form.buton_id),
        ("javax.faces.partial.execute", &form.buton_id),
        ("javax.faces.partial.render", "@all"),
        (&form.buton_id, &form.buton_id),
        (&form.form_id, &form.form_id),
        ("javax.faces.ViewState", &form.viewstate),
    ];

    client
        .post(&maks_url)
        .header("Faces-Request", "partial/ajax")
        .header("X-Requested-With", "XMLHttpRequest")
        .form(&veri)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
