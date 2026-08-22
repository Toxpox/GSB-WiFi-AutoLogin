use crate::config::*;
use crate::errors::*;
use crate::olcum::{Asama, AsamaOlcer};
use crate::parser;
use reqwest::header::LOCATION;
use reqwest::{cookie::Jar, redirect::Policy, Client, ClientBuilder};
use scraper::{Html, Selector};
use std::sync::Arc;
use std::sync::OnceLock;
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
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .read_timeout(Duration::from_secs(READ_TIMEOUT_SECS))
        .user_agent(PORTAL_USER_AGENT)
        .cookie_provider(jar)
}

fn client_build_hatasi(e: reqwest::Error) -> GSBError {
    GSBError::AgHatasi {
        mesaj: e.to_string(),
        kullanici_mesaji: "HTTP istemcisi olusturulamadi".into(),
    }
}

async fn sinirli_govde(yanit: reqwest::Response, limit: usize) -> Result<String, GSBError> {
    let mut yanit = yanit;
    let mut govde: Vec<u8> = Vec::new();

    loop {
        let parca = yanit.chunk().await.map_err(|e| GSBError::AgHatasi {
            mesaj: e.to_string(),
            kullanici_mesaji: "Sunucu yaniti okunamadi. Lutfen tekrar deneyin.".into(),
        })?;
        let Some(parca) = parca else { break };
        if govde.len() + parca.len() > limit {
            return Err(GSBError::AgHatasi {
                mesaj: format!("Yanit govdesi {} bayt sinirini asti", limit),
                kullanici_mesaji: "Sunucu beklenmeyen buyuklukte bir yanit dondurdu.".into(),
            });
        }
        govde.extend_from_slice(&parca);
    }

    Ok(String::from_utf8_lossy(&govde).into_owned())
}

#[derive(Debug, Clone, Default)]
pub struct GirisYaniti {
    pub html: String,
    pub ip: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DenemeKarari {
    GuvenliTekrar,

    Belirsiz,

    Kesin,
}

fn deneme_karari(hata: &reqwest::Error) -> DenemeKarari {
    if hata.is_connect() {
        DenemeKarari::GuvenliTekrar
    } else if hata.is_timeout() || hata.is_body() || hata.is_decode() {
        DenemeKarari::Belirsiz
    } else if hata.is_builder() || hata.is_redirect() {
        DenemeKarari::Kesin
    } else {
        DenemeKarari::Belirsiz
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
    let dns_hatasi = || GSBError::DNSHatasi {
        host: host.to_string(),
        kullanici_mesaji: "Sunucuya ulasilamiyor. VPN aktifse devre disi birakin.".into(),
    };
    let olcer = AsamaOlcer::basla(Asama::Dns);
    let addr = tokio::time::timeout(
        Duration::from_secs(DNS_TIMEOUT_SECS),
        tokio::net::lookup_host(format!("{}:{}", host, port)),
    )
    .await
    .map_err(|_| dns_hatasi())
    .and_then(|sonuc| sonuc.map_err(|_| dns_hatasi()))
    .and_then(|mut adresler| {
        adresler.next().ok_or_else(|| GSBError::DNSHatasi {
            host: host.to_string(),
            kullanici_mesaji: "DNS cozumlenemedi".into(),
        })
    });
    olcer.bitir(&addr);
    Ok(addr?.ip().to_string())
}

pub async fn giris_yap(
    client: &Client,
    url: &str,
    kullanici: &str,
    sifre: &str,
) -> Result<GirisYaniti, GSBError> {
    let veri = [
        ("j_username", kullanici),
        ("j_password", sifre),
        ("submit", "Giriş"),
    ];

    let mut son_hata: Option<GSBError> = None;
    let biten_sure = tokio::time::Instant::now() + Duration::from_secs(LOGIN_BUTCE_SECS);
    let toplam_olcer = AsamaOlcer::basla(Asama::LoginToplam);

    for deneme in 1..=MAX_DENEME {
        let deneme_olcer = AsamaOlcer::basla(Asama::LoginDenemesi);
        match client.post(url).form(&veri).send().await {
            Ok(r) => {
                let status = r.status();
                let final_url = r.url().to_string();
                let ip = r.remote_addr().map(|adres| adres.ip().to_string());
                let govde_olcer = AsamaOlcer::basla(Asama::Body);
                let body = match sinirli_govde(r, PORTAL_BODY_LIMIT).await {
                    Ok(body) => {
                        govde_olcer.bitir::<_, GSBError>(&Ok(()));
                        body
                    }
                    Err(e) => {
                        govde_olcer.bitir::<(), _>(&Err(&e));
                        son_hata = Some(e);
                        String::new()
                    }
                };

                if status.is_server_error() {
                    son_hata = Some(GSBError::AgHatasi {
                        mesaj: format!("HTTP {}", status),
                        kullanici_mesaji: "Sunucu gecici bir hata dondurdu. Lutfen tekrar deneyin."
                            .into(),
                    });
                } else if body.is_empty() {
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
                    match parser::sayfa_sinifla(&body) {
                        parser::PortalSayfa::Authenticated => {}
                        parser::PortalSayfa::LoginForm => {
                            return Err(GSBError::GirisBasarisiz {
                                mesaj: "Login formuna geri donuldu".into(),
                                kullanici_mesaji: "Kullanici adi veya sifrenizi kontrol edin."
                                    .into(),
                            });
                        }
                        parser::PortalSayfa::Bilinmiyor => {
                            return Err(GSBError::GirisBasarisiz {
                                mesaj: "Dogrulanamadi".into(),
                                kullanici_mesaji: "Giris dogrulanamadi".into(),
                            });
                        }
                    }
                    let basarili: Result<(), GSBError> = Ok(());
                    deneme_olcer.bitir(&basarili);
                    toplam_olcer.bitir_ek(&basarili, &format!("retry_count={}", deneme - 1));
                    return Ok(GirisYaniti { html: body, ip });
                }
            }
            Err(e) => {
                let karar = deneme_karari(&e);
                son_hata = Some(if e.is_timeout() {
                    GSBError::ZamanAsimi
                } else {
                    GSBError::AgHatasi {
                        mesaj: e.to_string(),
                        kullanici_mesaji: "GSB WiFi agina bagli oldugunuzdan emin olun.".into(),
                    }
                });

                match karar {
                    DenemeKarari::Belirsiz => {
                        if let Ok(html) = oturum_bilgisi_getir(client).await {
                            let basarili: Result<(), GSBError> = Ok(());
                            deneme_olcer.bitir(&basarili);
                            toplam_olcer.bitir_ek(
                                &basarili,
                                &format!("retry_count={} dogrulama=1", deneme - 1),
                            );
                            return Ok(GirisYaniti { html, ip: None });
                        }
                    }
                    DenemeKarari::Kesin => {
                        deneme_olcer.bitir::<(), _>(&Err(&()));
                        break;
                    }
                    DenemeKarari::GuvenliTekrar => {}
                }
                deneme_olcer.bitir::<(), _>(&Err(&()));
            }
        }

        if deneme < MAX_DENEME {
            let bekleme =
                BACKOFF_TABANI * BACKOFF_CARPAN.powi((deneme - 1) as i32) + rand::random::<f64>();
            let uyanma = tokio::time::Instant::now() + Duration::from_secs_f64(bekleme);
            if uyanma >= biten_sure {
                break;
            }
            tokio::time::sleep_until(uyanma).await;
        }
    }

    toplam_olcer.bitir_ek::<(), _>(&Err(&()), &format!("retry_count={}", MAX_DENEME - 1));
    Err(son_hata.unwrap_or(GSBError::AgHatasi {
        mesaj: "Baglanti kurulamadi".into(),
        kullanici_mesaji: "GSB WiFi agina bagli oldugunuzdan emin olun.".into(),
    }))
}

pub async fn internet_var_mi() -> bool {
    static NCSI_CLIENT: OnceLock<Option<Client>> = OnceLock::new();

    let Some(client) = NCSI_CLIENT.get_or_init(|| {
        ClientBuilder::new()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .build()
            .ok()
    }) else {
        return false;
    };

    match client.get(BAGLANTI_TEST_URL).send().await {
        Ok(yanit) => {
            let status = yanit.status().as_u16();
            let body = sinirli_govde(yanit, NCSI_BODY_LIMIT)
                .await
                .unwrap_or_default();
            ncsi_yaniti_saglam_mi(status, &body)
        }
        Err(_) => false,
    }
}

fn ncsi_yaniti_saglam_mi(status: u16, body: &str) -> bool {
    status == 200 && body.trim() == BAGLANTI_TEST_BEKLENEN
}

pub async fn gsb_aginda_mi() -> bool {
    let cozumleme = tokio::time::timeout(
        Duration::from_secs(DNS_TIMEOUT_SECS),
        tokio::net::lookup_host((PORTAL_HOST, 443)),
    )
    .await;
    let Ok(Ok(adresler)) = cozumleme else {
        return false;
    };
    let adresler: Vec<_> = adresler.collect();
    if adresler.iter().any(|addr| ozel_ip_mi(&addr.ip())) {
        return true;
    }

    let (v6, v4): (Vec<_>, Vec<_>) = adresler.into_iter().partition(|adr| adr.is_ipv6());
    let yaris = async {
        tokio::select! {
            basarili = sirayla_baglan(v4) => basarili,
            basarili = sirayla_baglan(v6) => basarili,
        }
    };

    matches!(
        tokio::time::timeout(Duration::from_secs(TCP_TIMEOUT_SECS), yaris).await,
        Ok(true)
    )
}

async fn sirayla_baglan(adresler: Vec<std::net::SocketAddr>) -> bool {
    for adr in adresler {
        if tokio::net::TcpStream::connect(adr).await.is_ok() {
            return true;
        }
    }
    std::future::pending().await
}

pub fn ozel_ip_mi(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_private(),
        std::net::IpAddr::V6(_) => false,
    }
}

pub async fn portal_erisim_testi(clients: &PortalClients) -> Result<u16, GSBError> {
    let yanit = clients
        .no_redirect
        .get(INDEX_URL)
        .send()
        .await
        .map_err(|e| GSBError::AgHatasi {
            mesaj: e.to_string(),
            kullanici_mesaji: "Portala erisilemedi.".into(),
        })?;
    Ok(yanit.status().as_u16())
}

pub async fn oturum_bilgisi_getir(client: &Client) -> Result<String, GSBError> {
    let yanit = client
        .get(INDEX_URL)
        .send()
        .await
        .map_err(|e| GSBError::AgHatasi {
            mesaj: e.to_string(),
            kullanici_mesaji: "Bilgiler alınamadı. GSB WiFi ağına bağlı olduğunuzdan emin olun."
                .into(),
        })?;
    let final_url = yanit.url().to_string();
    let body = sinirli_govde(yanit, PORTAL_BODY_LIMIT).await?;

    let oturum_dustu = final_url.contains("login.html")
        || final_url.contains("j_spring_security_check")
        || parser::sayfa_sinifla(&body) != parser::PortalSayfa::Authenticated;
    if oturum_dustu {
        return Err(GSBError::GirisBasarisiz {
            mesaj: "Oturum dusmus".into(),
            kullanici_mesaji: "Oturum düşmüş görünüyor. Lütfen yeniden bağlanın.".into(),
        });
    }
    Ok(body)
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
        let body = sinirli_govde(ilk_yanit, PORTAL_BODY_LIMIT).await?;
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
    let body = sinirli_govde(son_yanit, PORTAL_BODY_LIMIT).await?;

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
    let body_lower = crate::parser::turkce_kucult(body);

    body_lower.contains("j_spring_security_check")
        || body_lower.contains("basari ile")
        || partial_response_cikis_hedefi_mi(body)
        || cikis_sonuc_yolu_mu(final_url)
}

fn partial_response_cikis_hedefi_mi(body: &str) -> bool {
    static REDIRECT_SEL: OnceLock<Option<Selector>> = OnceLock::new();

    let document = Html::parse_fragment(body);
    let Some(selector) = REDIRECT_SEL
        .get_or_init(|| Selector::parse("partial-response redirect[url]").ok())
        .as_ref()
    else {
        return false;
    };

    document.select(selector).any(|redirect| {
        redirect
            .value()
            .attr("url")
            .is_some_and(cikis_sonuc_hedefi_mi)
    })
}

fn cikis_sonuc_yolu_mu(final_url: &str) -> bool {
    let Ok(url) = url::Url::parse(final_url) else {
        return false;
    };
    url_yolu_cikis_sonucu_mu(&url)
}

fn cikis_sonuc_hedefi_mi(hedef: &str) -> bool {
    let url = match url::Url::parse(hedef) {
        Ok(url) => url,
        Err(_) => {
            let Ok(taban) = url::Url::parse("https://portal.invalid/") else {
                return false;
            };
            let Ok(url) = taban.join(hedef) else {
                return false;
            };
            url
        }
    };
    url_yolu_cikis_sonucu_mu(&url)
}

fn url_yolu_cikis_sonucu_mu(url: &url::Url) -> bool {
    let Some(son) = url
        .path_segments()
        .and_then(|mut parcalar| parcalar.rfind(|parca| !parca.is_empty()))
    else {
        return false;
    };

    let endpoint = son.split(';').next().unwrap_or(son).to_ascii_lowercase();
    matches!(
        endpoint.as_str(),
        "login"
            | "login.html"
            | "cikisson"
            | "cikisson.html"
            | "cikissonrasi"
            | "cikissonrasi.html"
    )
}

pub async fn onceki_oturumu_kapat(
    client: &Client,
    form: &parser::FormBilgi,
    login_url: &str,
) -> bool {
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

pub async fn oturum_dusmesini_bekle(client: &Client) {
    let biten_sure = tokio::time::Instant::now() + Duration::from_secs(OTURUM_DUSME_BUTCE_SECS);

    while tokio::time::Instant::now() < biten_sure {
        if oturum_bilgisi_getir(client).await.is_err() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(OTURUM_YOKLAMA_ARALIK_MS)).await;
    }
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
        assert!(ncsi_yaniti_saglam_mi(200, "Microsoft Connect Test\r\n"));

        assert!(!ncsi_yaniti_saglam_mi(200, "<html>login</html>"));
        assert!(!ncsi_yaniti_saglam_mi(302, ""));

        assert!(!ncsi_yaniti_saglam_mi(
            200,
            "<html><body>Microsoft Connect Test</body></html>"
        ));
    }

    #[test]
    fn cikis_basarisi_dogru_tespit_edilir() {
        assert!(cikis_yaniti_basarili_mi(
            "<form action=\"/j_spring_security_check\">",
            "https://portal.example/"
        ));
        assert!(cikis_yaniti_basarili_mi(
            "",
            "https://portal.example/cikisSon.html?logout=1"
        ));
        assert!(cikis_yaniti_basarili_mi(
            "",
            "https://portal.example/login/"
        ));
        assert!(cikis_yaniti_basarili_mi(
            "",
            "https://portal.example/login.html;jsessionid=ABC123"
        ));
        assert!(cikis_yaniti_basarili_mi(
            "<partial-response><redirect url=\"/login.html\"/></partial-response>",
            "https://portal.example/maksimum.html"
        ));
        assert!(
            !cikis_yaniti_basarili_mi(
                "<partial-response><redirect url=\"/hata.html\"/><changes><update>login hatasi</update></changes></partial-response>",
                "https://portal.example/maksimum.html"
            ),
            "redirect hedefi hata sayfasiyken govdenin baska yerindeki login kelimesi basari sayilmamali"
        );
        assert!(
            !cikis_yaniti_basarili_mi(
                "<partial-response><redirect url=\"/hata.html?next=login\"/></partial-response>",
                "https://portal.example/maksimum.html"
            ),
            "redirect query'sindeki login kelimesi endpoint eslesmesi degildir"
        );

        assert!(cikis_yaniti_basarili_mi(
            "<p>Oturumunuz Başarı ile sonlandırıldı.</p>",
            "https://portal.example/"
        ));

        assert!(
            !cikis_yaniti_basarili_mi("", "https://logineksatolyesi.example/anasayfa"),
            "host adinin icindeki 'login' basari sayilmamali"
        );
        assert!(
            !cikis_yaniti_basarili_mi("", "https://portal.example/blogindex.html"),
            "yol adinin icindeki 'login' basari sayilmamali"
        );
        assert!(
            !cikis_yaniti_basarili_mi("", "https://cikissonuc.example/anasayfa"),
            "host adinin icindeki 'cikisson' basari sayilmamali"
        );
        assert!(
            !cikis_yaniti_basarili_mi("", "https://portal.example/oncikissonuc.html"),
            "benzer bir yol adi cikis endpoint'i sayilmamali"
        );
        assert!(
            !cikis_yaniti_basarili_mi(
                "",
                "https://portal.example/hata.html?next=cikisSonrasi.html"
            ),
            "query icindeki cikis endpoint'i son yol bileseni degildir"
        );
        assert!(
            !cikis_yaniti_basarili_mi("", "https://portal.example/login.foo.html"),
            "ilk nokta oncesi 'login' olan farkli bir dosya eslesmemeli"
        );
        assert!(
            !cikis_yaniti_basarili_mi("", "gecersiz-cikisson-url"),
            "gecersiz bir URL yalnizca alt dizge nedeniyle basari sayilmamali"
        );

        assert!(!cikis_yaniti_basarili_mi(
            "<p>Hata olustu.</p>",
            "https://portal.example/hata.html"
        ));
    }

    async fn sahte_sunucu(yanit: Vec<u8>) -> String {
        let dinleyici = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adres = dinleyici.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((mut soket, _)) = dinleyici.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut tampon = [0u8; 1024];
                let _ = soket.read(&mut tampon).await;
                let _ = soket.write_all(&yanit).await;
                let _ = soket.shutdown().await;
            }
        });

        format!("http://{}/", adres)
    }

    fn duz_yanit(govde: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            govde.len(),
            govde
        )
        .into_bytes()
    }

    #[tokio::test]
    async fn govde_siniri_asilinca_hata_doner() {
        let buyuk = "x".repeat(4096);
        let url = sahte_sunucu(duz_yanit(&buyuk)).await;
        let client = Client::builder().build().unwrap();
        let yanit = client.get(&url).send().await.unwrap();

        let sonuc = sinirli_govde(yanit, 1024).await;

        match sonuc {
            Err(GSBError::AgHatasi { mesaj, .. }) => assert!(mesaj.contains("1024")),
            other => panic!("sinir asimi hatasi bekleniyordu, gelen: {:?}", other),
        }
    }

    #[tokio::test]
    async fn sinir_altindaki_govde_okunur() {
        let url = sahte_sunucu(duz_yanit("<html>merhaba</html>")).await;
        let client = Client::builder().build().unwrap();
        let yanit = client.get(&url).send().await.unwrap();

        let govde = sinirli_govde(yanit, 1024).await.unwrap();

        assert_eq!(govde, "<html>merhaba</html>");
    }

    #[tokio::test]
    async fn baglanti_hatasi_guvenli_tekrar_sayilir() {
        let client = Client::builder().build().unwrap();

        let hata = client
            .get("http://127.0.0.1:1/")
            .send()
            .await
            .expect_err("kapali porta baglanti basarili olmamali");

        assert!(hata.is_connect());
        assert_eq!(deneme_karari(&hata), DenemeKarari::GuvenliTekrar);
    }

    #[tokio::test]
    async fn timeout_belirsiz_sayilir() {
        let dinleyici = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adres = dinleyici.local_addr().unwrap();
        tokio::spawn(async move {
            let kabul = dinleyici.accept().await;
            tokio::time::sleep(Duration::from_secs(30)).await;
            drop(kabul);
        });

        let client = Client::builder()
            .timeout(Duration::from_millis(150))
            .build()
            .unwrap();
        let hata = client
            .get(format!("http://{}/", adres))
            .send()
            .await
            .expect_err("yanit yazmayan sunucuda timeout bekleniyordu");

        assert!(hata.is_timeout());
        assert_eq!(deneme_karari(&hata), DenemeKarari::Belirsiz);
    }

    const OTURUM_HTML: &str =
        "<div id=\"content-div\"><center><span class=\"myinfo\">TEST</span></center></div>";
    const LOGIN_HTML: &str =
        "<form action=\"/j_spring_security_check\"><input name=\"j_username\"><input name=\"j_password\"></form>";

    async fn sirali_sunucu(yanitlar: Vec<Vec<u8>>) -> String {
        let dinleyici = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adres = dinleyici.local_addr().unwrap();

        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            for yanit in yanitlar {
                let Ok((mut soket, _)) = dinleyici.accept().await else {
                    return;
                };
                let mut tampon = [0u8; 4096];
                let _ = soket.read(&mut tampon).await;
                let _ = soket.write_all(&yanit).await;
                let _ = soket.shutdown().await;
            }
        });

        format!("http://{}/", adres)
    }

    fn html_yanit(govde: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            govde.len(),
            govde
        )
        .into_bytes()
    }

    fn gzip_yanit(govde: &str) -> Vec<u8> {
        use std::io::Write;

        let mut deflate = Vec::new();
        for (i, parca) in govde.as_bytes().chunks(65535).enumerate() {
            let son = (i + 1) * 65535 >= govde.len();
            deflate.push(if son { 1 } else { 0 });
            deflate
                .write_all(&(parca.len() as u16).to_le_bytes())
                .unwrap();
            deflate
                .write_all(&(!(parca.len() as u16)).to_le_bytes())
                .unwrap();
            deflate.write_all(parca).unwrap();
        }

        let mut govde_gz = vec![0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 0xff];
        govde_gz.extend_from_slice(&deflate);
        govde_gz.extend_from_slice(&crc32(govde.as_bytes()).to_le_bytes());
        govde_gz.extend_from_slice(&(govde.len() as u32).to_le_bytes());

        let mut yanit = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            govde_gz.len()
        )
        .into_bytes();
        yanit.extend_from_slice(&govde_gz);
        yanit
    }

    fn crc32(veri: &[u8]) -> u32 {
        let mut crc = 0xffff_ffffu32;
        for &bayt in veri {
            crc ^= bayt as u32;
            for _ in 0..8 {
                let maske = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (0xedb8_8320 & maske);
            }
        }
        !crc
    }

    #[tokio::test]
    async fn basarili_login_authenticated_html_dondurur() {
        let url = sirali_sunucu(vec![html_yanit(OTURUM_HTML)]).await;
        let client = Client::builder().build().unwrap();

        let yanit = giris_yap(&client, &url, "kullanici", "sifre")
            .await
            .expect("oturum acik sayfa basarili sayilmaliydi");

        assert!(yanit.html.contains("content-div"));
        assert!(yanit.ip.is_some(), "IP remote_addr'dan gelmeliydi");
    }

    #[tokio::test]
    async fn login_formuna_donus_kimlik_hatasi_uretir() {
        let url = sirali_sunucu(vec![html_yanit(LOGIN_HTML)]).await;
        let client = Client::builder().build().unwrap();

        let hata = giris_yap(&client, &url, "kullanici", "yanlis")
            .await
            .expect_err("login formu basari sayilmamali");

        assert!(matches!(hata, GSBError::GirisBasarisiz { .. }));
    }

    #[tokio::test]
    async fn gzip_yanit_decode_edilir() {
        let url = sirali_sunucu(vec![gzip_yanit(OTURUM_HTML)]).await;
        let client = Client::builder().gzip(true).build().unwrap();

        let yanit = giris_yap(&client, &url, "kullanici", "sifre")
            .await
            .expect("gzip yanit decode edilip basarili sayilmaliydi");

        assert!(yanit.html.contains("content-div"));
    }

    #[tokio::test]
    async fn gzip_decode_sonrasi_limit_uygulanir() {
        let buyuk = "y".repeat(200 * 1024);
        let url = sirali_sunucu(vec![gzip_yanit(&buyuk)]).await;
        let client = Client::builder().gzip(true).build().unwrap();
        let yanit = client.get(&url).send().await.unwrap();

        let sonuc = sinirli_govde(yanit, 64 * 1024).await;

        assert!(
            matches!(sonuc, Err(GSBError::AgHatasi { .. })),
            "decode sonrasi buyuyen govde limiti asmali"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn sunucu_hatasindan_sonra_tekrar_denenir() {
        let hata_yaniti =
            b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_vec();
        let url = sirali_sunucu(vec![hata_yaniti, html_yanit(OTURUM_HTML)]).await;
        let client = Client::builder().build().unwrap();

        let yanit = giris_yap(&client, &url, "kullanici", "sifre")
            .await
            .expect("503 sonrasi ikinci deneme basarili olmaliydi");

        assert!(yanit.html.contains("content-div"));
    }

    #[tokio::test(start_paused = true)]
    async fn asiri_buyuk_login_yaniti_reddedilir() {
        let dev_govde = "z".repeat(PORTAL_BODY_LIMIT + 1024);
        let url = sirali_sunucu(vec![
            html_yanit(&dev_govde),
            html_yanit(&dev_govde),
            html_yanit(&dev_govde),
        ])
        .await;
        let client = Client::builder().build().unwrap();

        let hata = giris_yap(&client, &url, "kullanici", "sifre")
            .await
            .expect_err("limit asan govde kabul edilmemeli");

        assert!(matches!(hata, GSBError::AgHatasi { .. }));
    }
}
