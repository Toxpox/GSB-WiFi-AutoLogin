use crate::config::*;
use crate::errors::*;
use crate::parser;
use reqwest::header::LOCATION;
use reqwest::{cookie::Jar, redirect::Policy, Client, ClientBuilder};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct PortalClients {
    pub normal: Client,
    pub no_redirect: Client,
}

pub fn client_olustur() -> Result<PortalClients, GSBError> {
    let jar = Arc::new(Jar::default());
    let normal = portal_client_builder(jar.clone())
        .build()
        .map_err(client_build_hatasi)?;
    let no_redirect = portal_client_builder(jar)
        .redirect(Policy::none())
        .build()
        .map_err(client_build_hatasi)?;

    Ok(PortalClients {
        normal,
        no_redirect,
    })
}

fn portal_client_builder(jar: Arc<Jar>) -> ClientBuilder {
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .user_agent(PORTAL_USER_AGENT)
        .cookie_provider(jar)
}

fn client_build_hatasi(e: reqwest::Error) -> GSBError {
    GSBError::AgHatasi {
        mesaj: e.to_string(),
        kullanici_mesaji: "HTTP istemcisi olusturulamadi".into(),
    }
}

pub async fn ip_bul(url: &str) -> Result<String, GSBError> {
    let parsed: url::Url = url.parse().map_err(|_| GSBError::DNSHatasi {
        host: url.to_string(),
        kullanici_mesaji: "URL hatali".into(),
    })?;
    let host = parsed.host_str().ok_or_else(|| GSBError::DNSHatasi {
        host: url.to_string(),
        kullanici_mesaji: "URL hatali".into(),
    })?;
    let port = parsed.port_or_known_default().unwrap_or(443);
    let addr = tokio::net::lookup_host(format!("{}:{}", host, port))
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

    // Not: Login POST'u idempotent degildir; timeout sonrasi tekrar deneme,
    // ilk istek sunucuya ulastiysa maksimum cihaz akisini tetikleyebilir.
    // Portal bu durumu maksimumCihazHakkiDolu yonlendirmesiyle bildirdigi
    // icin tekrar deneme bilincli olarak korunuyor.
    for deneme in 1..=MAX_DENEME {
        match client.post(url).form(&veri).send().await {
            Ok(r) => {
                let status = r.status();
                let final_url = r.url().to_string();
                let body = match r.text().await {
                    Ok(body) => body,
                    Err(e) => {
                        son_hata = Some(GSBError::AgHatasi {
                            mesaj: e.to_string(),
                            kullanici_mesaji: "Sunucu yaniti okunamadi. Lutfen tekrar deneyin."
                                .into(),
                        });
                        String::new()
                    }
                };

                if status.is_server_error() {
                    // 5xx gecici sunucu hatasidir; "Giris dogrulanamadi"
                    // yerine retry/backoff akisina sokulur.
                    son_hata = Some(GSBError::AgHatasi {
                        mesaj: format!("HTTP {}", status),
                        kullanici_mesaji: "Sunucu gecici bir hata dondurdu. Lutfen tekrar deneyin."
                            .into(),
                    });
                } else if body.is_empty() {
                    // Body okunamadiginda retry/backoff akisi devam etsin.
                } else {
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

/// Internet erisimi var mi? Captive portal oturumu dusmusse NCSI istegi
/// portala yonlenir ve beklenen govde donmez.
pub async fn internet_var_mi() -> bool {
    let Ok(client) = ClientBuilder::new()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
    else {
        return false;
    };

    match client.get(BAGLANTI_TEST_URL).send().await {
        Ok(yanit) => {
            let status = yanit.status().as_u16();
            let body = yanit.text().await.unwrap_or_default();
            ncsi_yaniti_saglam_mi(status, &body)
        }
        Err(_) => false,
    }
}

fn ncsi_yaniti_saglam_mi(status: u16, body: &str) -> bool {
    status == 200 && body.contains(BAGLANTI_TEST_BEKLENEN)
}

/// GSB aginda miyiz? Once DNS: cozulemiyorsa portal erisilemez demektir.
/// Ozel (RFC 1918) IP'ye cozuluyorsa kesin yurt agindayiz. Genel IP'ye
/// cozulduyse TCP baglanti denemesiyle dogrulanir; yanlis negatif kullaniciyi
/// engellememeli.
pub async fn gsb_aginda_mi() -> bool {
    let Ok(adresler) = tokio::net::lookup_host((PORTAL_HOST, 443)).await else {
        return false;
    };
    let adresler: Vec<_> = adresler.collect();
    if adresler.iter().any(|addr| ozel_ip_mi(&addr.ip())) {
        return true;
    }
    for addr in adresler {
        let deneme =
            tokio::time::timeout(Duration::from_secs(3), tokio::net::TcpStream::connect(addr))
                .await;
        if matches!(deneme, Ok(Ok(_))) {
            return true;
        }
    }
    false
}

fn ozel_ip_mi(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_private(),
        std::net::IpAddr::V6(_) => false,
    }
}

pub async fn cikis_yap(clients: &PortalClients) -> Result<bool, GSBError> {
    let ilk_yanit = clients
        .no_redirect
        .get(LOGOUT_URL)
        .headers(tarayici_navigasyon_basliklari(INDEX_URL))
        .send()
        .await
        .map_err(|e| GSBError::AgHatasi {
            mesaj: e.to_string(),
            kullanici_mesaji:
                "Cikis istegi gonderilemedi. GSB WiFi agina bagli oldugunuzdan emin olun.".into(),
        })?;

    if ilk_yanit.status().is_success() {
        let status = ilk_yanit.status();
        let final_url = ilk_yanit.url().to_string();
        let body = ilk_yanit.text().await.map_err(|e| GSBError::AgHatasi {
            mesaj: e.to_string(),
            kullanici_mesaji: "Cikis yaniti okunamadi. Lutfen tekrar deneyin.".into(),
        })?;
        return Ok(status.is_success() && cikis_yaniti_basarili_mi(&body, &final_url));
    }

    if !ilk_yanit.status().is_redirection() {
        return Ok(false);
    }

    let yonlendirme_url =
        cikis_yonlendirme_url(&ilk_yanit).unwrap_or_else(|| CIKIS_SON_URL.to_string());

    let son_yanit = clients
        .no_redirect
        .get(&yonlendirme_url)
        .headers(tarayici_navigasyon_basliklari(INDEX_URL))
        .send()
        .await
        .map_err(|e| GSBError::AgHatasi {
            mesaj: e.to_string(),
            kullanici_mesaji:
                "Cikis sonucu alinamadi. GSB WiFi agina bagli oldugunuzdan emin olun.".into(),
        })?;

    let status = son_yanit.status();
    let final_url = son_yanit.url().to_string();
    let body = son_yanit.text().await.map_err(|e| GSBError::AgHatasi {
        mesaj: e.to_string(),
        kullanici_mesaji: "Cikis yaniti okunamadi. Lutfen tekrar deneyin.".into(),
    })?;

    Ok(status.is_success() && cikis_yaniti_basarili_mi(&body, &final_url))
}

fn tarayici_navigasyon_basliklari(referer: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert(
        "Accept-Language",
        "tr-TR,tr;q=0.9,en-GB;q=0.8,en;q=0.7,en-US;q=0.6"
            .parse()
            .unwrap(),
    );
    headers.insert("Referer", referer.parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());
    headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
    headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
    headers
}

fn cikis_yonlendirme_url(yanit: &reqwest::Response) -> Option<String> {
    let location = yanit.headers().get(LOCATION)?.to_str().ok()?;
    yanit.url().join(location).ok().map(|url| url.to_string())
}

fn cikis_yaniti_basarili_mi(body: &str, final_url: &str) -> bool {
    let body_lower = body.to_lowercase();
    let final_url_lower = final_url.to_lowercase();

    body_lower.contains("j_spring_security_check")
        || final_url_lower.contains("cikisson")
        || body_lower.contains("basari ile")
        || body_lower.contains("partial-response")
            && body_lower.contains("redirect")
            && (body_lower.contains("login") || body_lower.contains("cikisson"))
        || final_url_lower.contains("login")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ozel_ip_dogru_tespit_edilir() {
        let ozel: std::net::IpAddr = "10.20.30.40".parse().unwrap();
        let ozel2: std::net::IpAddr = "192.168.1.1".parse().unwrap();
        let genel: std::net::IpAddr = "93.184.216.34".parse().unwrap();
        assert!(ozel_ip_mi(&ozel));
        assert!(ozel_ip_mi(&ozel2));
        assert!(!ozel_ip_mi(&genel));
    }

    #[test]
    fn ncsi_yaniti_dogru_degerlendirilir() {
        assert!(ncsi_yaniti_saglam_mi(200, "Microsoft Connect Test"));
        // Portal araya girdiginde farkli govde veya yonlendirme doner.
        assert!(!ncsi_yaniti_saglam_mi(200, "<html>login</html>"));
        assert!(!ncsi_yaniti_saglam_mi(302, ""));
    }
}
