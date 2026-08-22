use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! secici {
    ($ad:ident, $desen:literal) => {
        static $ad: LazyLock<Selector> = LazyLock::new(|| Selector::parse($desen).unwrap());
    };
}

secici!(BLOK_SEL, "#content-div > center");
secici!(SPAN_MYINFO_SEL, "span.myinfo");
secici!(LABEL_MYINFO_SEL, "label.myinfo");
secici!(LABEL_SEL, "label");
secici!(TD_SEL, "td");
secici!(KOTA_TR_SEL, "#mainPanel\\:kotaDisplay tr");
secici!(CIHAZ_HUCRE_SEL, "#j_idt20_data tr td[role=gridcell]");
secici!(CIHAZ_FORM_SEL, "#j_idt20_data tr form");
secici!(SUBMIT_SEL, "button[type=submit]");
secici!(VIEWSTATE_SEL, "input[name='javax.faces.ViewState']");

#[derive(Debug, Serialize, Clone, Default)]
pub struct KullaniciBilgi {
    pub isim: String,
    pub son_giris: String,
    pub konum: String,
    pub kota: HashMap<String, String>,
    pub kota_doldu: bool,
    pub detaylar: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct FormBilgi {
    pub form_id: String,
    pub buton_id: String,
    pub viewstate: String,
}

fn kota_normalize(key: &str) -> String {
    let sade: String = turkce_kucult(key)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let eslesme = match sade.as_str() {
        "toplam kota (mb)" | "total quota (mb)" => Some("toplam_mb"),
        "toplam kalan kota (mb)" | "total remaining quota (mb)" => Some("kalan_mb"),
        "yenilenme tarihi" | "next refresh date" => Some("yenilenme"),
        "oturum suresi" | "session time" => Some("oturum_suresi"),
        "login zamani" | "login time" => Some("login_zamani"),
        "baslangic tarihi" | "start date" => Some("baslangic"),
        "sona erme tarihi" | "expiration date" => Some("bitis"),
        "kalan kota zamani" | "remaining quota time" => Some("kalan_zaman"),
        _ => None,
    };

    match eslesme {
        Some(anahtar) => anahtar.to_string(),

        None => key.trim().to_string(),
    }
}

pub fn bilgi_cek(html: &str) -> KullaniciBilgi {
    let document = Html::parse_document(html);
    let mut bilgi = KullaniciBilgi {
        isim: "Kullanıcı".into(),
        ..Default::default()
    };

    if let Some(blok) = document.select(&BLOK_SEL).next() {
        kimlik_alanlarini_doldur(&mut bilgi, &blok);
    }

    bilgi.kota = kota_cek(&document);
    bilgi.kota_doldu = kota_doldu_mu(&document);
    bilgi
}

fn kimlik_alanlarini_doldur(bilgi: &mut KullaniciBilgi, blok: &ElementRef<'_>) {
    if let Some(span) = blok.select(&SPAN_MYINFO_SEL).next() {
        let txt: String = span
            .text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !txt.is_empty() {
            bilgi.isim = txt.clone();
            bilgi.detaylar.push(txt);
        }
    }

    for lbl in blok.select(&LABEL_MYINFO_SEL) {
        let txt: String = lbl
            .text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if txt.is_empty() {
            continue;
        }
        bilgi.detaylar.push(txt.clone());
        alan_ayikla(bilgi, &txt);
    }

    if bilgi.konum.is_empty() || bilgi.son_giris.is_empty() {
        for lbl in blok.select(&LABEL_SEL) {
            let classes = lbl.value().attr("class").unwrap_or("");
            if classes.contains("myinfo") {
                continue;
            }
            let txt: String = lbl
                .text()
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if txt.is_empty() {
                continue;
            }
            let onceki_konum = bilgi.konum.clone();
            let onceki_giris = bilgi.son_giris.clone();
            alan_ayikla(bilgi, &txt);
            if (bilgi.konum != onceki_konum || bilgi.son_giris != onceki_giris)
                && !bilgi.detaylar.contains(&txt)
            {
                bilgi.detaylar.push(txt);
            }
        }
    }
}

pub(crate) fn turkce_kucult(metin: &str) -> String {
    let mut sonuc = String::with_capacity(metin.len());
    for c in metin.chars() {
        match c {
            'ı' | 'İ' | 'I' | 'i' => sonuc.push('i'),
            'ş' | 'Ş' => sonuc.push('s'),
            'ğ' | 'Ğ' => sonuc.push('g'),
            'ü' | 'Ü' => sonuc.push('u'),
            'ö' | 'Ö' => sonuc.push('o'),
            'ç' | 'Ç' => sonuc.push('c'),
            other => sonuc.extend(other.to_lowercase()),
        }
    }
    sonuc
}

fn kota_doldu_mu(document: &Html) -> bool {
    let gorunur = gorunur_metin(document);
    let metin = turkce_kucult(&gorunur);
    const ISARETLER: [&str; 14] = [
        "quota is expired",
        "quota has expired",
        "quota expired",
        "kotaniz doldu",
        "kota doldu",
        "kotaniz bitti",
        "kota bitti",
        "kotaniz tukendi",
        "kota tukendi",
        "kotaniz sona erdi",
        "kota sona erdi",
        "kotaniz dolmustur",
        "kota dolmustur",
        "kota sureniz doldu",
    ];
    ISARETLER.iter().any(|isaret| metin.contains(isaret))
}

fn gorunur_metin(document: &Html) -> String {
    document
        .root_element()
        .descendants()
        .filter_map(|dugum| {
            let metin = dugum.value().as_text()?;
            let gizli_altinda = dugum
                .ancestors()
                .filter_map(ElementRef::wrap)
                .any(|element| element_gizli_mi(&element));
            if gizli_altinda {
                None
            } else {
                Some(&**metin)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn element_gizli_mi(element: &ElementRef<'_>) -> bool {
    let deger = element.value();
    if matches!(deger.name(), "script" | "style" | "template" | "noscript")
        || deger.attr("hidden").is_some()
        || deger.attr("inert").is_some()
        || deger
            .attr("aria-hidden")
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("true"))
    {
        return true;
    }

    deger.attr("style").is_some_and(|style| {
        let sade: String = style
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .flat_map(char::to_lowercase)
            .collect();
        sade.contains("display:none")
            || sade.contains("visibility:hidden")
            || sade.contains("visibility:collapse")
    })
}

fn alan_ayikla(bilgi: &mut KullaniciBilgi, txt: &str) {
    let lower = turkce_kucult(txt);
    if bilgi.son_giris.is_empty() && (lower.contains("son giris") || lower.contains("last login")) {
        if let Some((_, val)) = txt.split_once(':') {
            bilgi.son_giris = val.trim().to_string();
        }
    }
    if bilgi.konum.is_empty() && (lower.contains("konum") || lower.contains("location")) {
        if let Some((_, val)) = txt.split_once(':') {
            bilgi.konum = val.trim().to_string();
        }
    }
}

fn kota_cek(document: &Html) -> HashMap<String, String> {
    let mut kota = HashMap::new();

    for tr in document.select(&KOTA_TR_SEL) {
        let tds: Vec<_> = tr.select(&TD_SEL).collect();
        if tds.len() >= 2 {
            let key_label = tds[0].select(&LABEL_SEL).next();
            let val_label = tds[1].select(&LABEL_SEL).next();

            if let (Some(kl), Some(vl)) = (key_label, val_label) {
                let key = kl.text().collect::<String>().trim().to_string();
                let val = vl.text().collect::<String>().trim().to_string();

                if key.is_empty() || key == "------" || val.is_empty() || val == "------" {
                    continue;
                }

                let clean_key = key.trim_end_matches(':');
                kota.insert(kota_normalize(clean_key), val);
            }
        }
    }

    kota
}

pub fn maksimum_bilgi_cek(html: &str) -> crate::errors::CihazBilgisi {
    let document = Html::parse_document(html);
    let mut bilgi = crate::errors::CihazBilgisi::default();

    let hucreler: Vec<_> = document.select(&CIHAZ_HUCRE_SEL).collect();

    if hucreler.len() >= 3 {
        bilgi.baslangic = hucreler[0].text().collect::<String>().trim().to_string();
        bilgi.mac = hucreler[1].text().collect::<String>().trim().to_string();
        bilgi.konum = hucreler[2].text().collect::<String>().trim().to_string();
    }
    bilgi
}

pub fn maksimum_form_bilgi_cek(html: &str) -> FormBilgi {
    let document = Html::parse_document(html);

    let Some(form) = document.select(&CIHAZ_FORM_SEL).next() else {
        return FormBilgi::default();
    };

    let mut bilgi = form_temel_bilgi_cek(&form);

    if let Some(btn) = form.select(&SUBMIT_SEL).next() {
        bilgi.buton_id = buton_kimligi(&btn);
    }
    bilgi
}

fn form_temel_bilgi_cek(form: &ElementRef<'_>) -> FormBilgi {
    let form_id = form
        .value()
        .attr("id")
        .or_else(|| form.value().attr("name"))
        .unwrap_or("")
        .to_string();
    let mut bilgi = FormBilgi {
        form_id,
        ..Default::default()
    };

    if let Some(vs) = form.select(&VIEWSTATE_SEL).next() {
        bilgi.viewstate = vs.value().attr("value").unwrap_or("").to_string();
    }

    bilgi
}

fn buton_kimligi(btn: &ElementRef<'_>) -> String {
    btn.value()
        .attr("name")
        .or_else(|| btn.value().attr("id"))
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html_kota_doldu_mu(html: &str) -> bool {
        let document = Html::parse_document(html);
        kota_doldu_mu(&document)
    }

    #[test]
    fn basarili_giris_bilgilerini_ayiklar() {
        let html = r#"
            <div id="content-div">
                <center>
                    <span class="myinfo">TEST KULLANICI </span>
                    <label class="myinfo">Last Login: 25.04.2026 05:07</label>
                    <label class="myinfo">Location : ABC ÖĞRENCİ YURDU</label>
                </center>
            </div>
            <table id="mainPanel:kotaDisplay">
                <tr>
                    <td><label>Total Quota (MB):</label></td>
                    <td><label>10240</label></td>
                </tr>
                <tr>
                    <td><label>Total Remaining Quota (MB):</label></td>
                    <td><label>5120</label></td>
                </tr>
                <tr>
                    <td><label>Next Refresh Date:</label></td>
                    <td><label>2026-05-01</label></td>
                </tr>
            </table>
        "#;

        let bilgi = bilgi_cek(html);

        assert_eq!(bilgi.isim, "TEST KULLANICI");
        assert!(!bilgi.kota_doldu);
        assert_eq!(bilgi.son_giris, "25.04.2026 05:07");
        assert_eq!(bilgi.konum, "ABC ÖĞRENCİ YURDU");
        assert_eq!(
            bilgi.kota.get("toplam_mb").map(String::as_str),
            Some("10240")
        );
        assert_eq!(bilgi.kota.get("kalan_mb").map(String::as_str), Some("5120"));
        assert_eq!(
            bilgi.kota.get("yenilenme").map(String::as_str),
            Some("2026-05-01")
        );
    }

    #[test]
    fn maksimum_cihaz_bilgilerini_ayiklar() {
        let html = r#"
            <table id="j_idt20_data">
                <tr>
                    <td role="gridcell">2026-04-24 04:59</td>
                    <td role="gridcell">AA:BB:CC:DD:EE:FF</td>
                    <td role="gridcell">Yurt WiFi</td>
                    <td>
                        <form id="mainForm">
                            <button type="submit" id="disconnectButton"></button>
                            <input name="javax.faces.ViewState" value="view-state-1">
                        </form>
                    </td>
                </tr>
            </table>
        "#;

        let cihaz = maksimum_bilgi_cek(html);
        let form = maksimum_form_bilgi_cek(html);

        assert_eq!(cihaz.baslangic, "2026-04-24 04:59");
        assert_eq!(cihaz.mac, "AA:BB:CC:DD:EE:FF");
        assert_eq!(cihaz.konum, "Yurt WiFi");
        assert_eq!(form.form_id, "mainForm");
        assert_eq!(form.buton_id, "disconnectButton");
        assert_eq!(form.viewstate, "view-state-1");
    }

    #[test]
    fn kota_doldu_uyarisi_tespit_edilir() {
        assert!(!html_kota_doldu_mu(
            r#"<label>Quota information is updated every 5 m.</label>"#
        ));

        assert!(html_kota_doldu_mu(
            r#"<label style="color:red;">Your quota is expired.</label>"#
        ));

        assert!(html_kota_doldu_mu(r#"<label>Kotanız doldu.</label>"#));
    }

    #[test]
    fn gizli_dom_ve_yorum_icindeki_metin_kotayi_doldurmaz() {
        assert!(!html_kota_doldu_mu(
            r#"<script>var msg = "Your quota is expired.";</script>"#
        ));

        assert!(!html_kota_doldu_mu(
            r#"<!-- Kotanız doldu. --><p>Hoş geldiniz</p>"#
        ));

        assert!(!html_kota_doldu_mu(
            r#"<div hidden>Your quota is expired.</div>"#
        ));
        assert!(!html_kota_doldu_mu(
            r#"<div aria-hidden=" TRUE ">Your quota is expired.</div>"#
        ));
        assert!(!html_kota_doldu_mu(
            r#"<div style="display: none">Your quota is expired.</div>"#
        ));
        assert!(!html_kota_doldu_mu(
            r#"<div style="visibility: hidden">Your quota is expired.</div>"#
        ));
        assert!(!html_kota_doldu_mu(
            r#"<div inert>Your quota is expired.</div>"#
        ));
        assert!(!html_kota_doldu_mu(
            r#"<template>Your quota is expired.</template>"#
        ));
        assert!(!html_kota_doldu_mu(
            r#"<noscript>Your quota is expired.</noscript>"#
        ));

        assert!(html_kota_doldu_mu(
            r#"<div><label style="color:red;">Your quota is expired.</label></div>"#
        ));
    }

    #[test]
    fn scriptteki_uyari_kotayi_dolu_saymaz() {
        let html = r#"
            <div id="content-div">
                <center>
                    <span class="myinfo">TEST KULLANICI</span>
                    <div id="mainPanel:kotaDisplay">
                        <table>
                            <tr>
                                <td><label>Total Quota (MB):</label></td>
                                <td><label>5000</label></td>
                            </tr>
                            <tr>
                                <td><label>Total Remaining Quota (MB):</label></td>
                                <td><label>2500</label></td>
                            </tr>
                        </table>
                    </div>
                    <script>var uyari = "Your quota is expired.";</script>
                </center>
            </div>
        "#;

        let bilgi = bilgi_cek(html);

        assert_eq!(bilgi.kota.get("kalan_mb").map(String::as_str), Some("2500"));
        assert!(
            !bilgi.kota_doldu,
            "kalan kota 2500 MB iken kota dolu sayılmamalı"
        );
    }

    #[test]
    fn gorunur_kota_uyarisi_sayisal_degere_onceliklidir() {
        let html = r#"
            <div id="content-div">
                <center>
                    <div id="mainPanel:kotaDisplay">
                        <table>
                            <tr>
                                <td><label>Total Remaining Quota (MB):</label></td>
                                <td><label>1500</label></td>
                            </tr>
                        </table>
                    </div>
                    <label style="color:red;">Your quota is expired.</label>
                </center>
            </div>
        "#;

        let bilgi = bilgi_cek(html);

        assert!(
            bilgi.kota_doldu,
            "portalın açık kota uyarısı güncelliğini yitirmiş olabilecek sayısal satırla bastırılmamalı"
        );
    }

    #[test]
    fn kota_doldugunda_isaretlenir() {
        let html = r#"
            <div id="content-div">
                <center>
                    <span class="myinfo">TEST KULLANICI</span>
                    <label class="myinfo">Last Login: 31.05.2026 16:24</label>
                    <label class="myinfo">Location : ABC ÖĞRENCİ YURDU</label>
                    <div id="mainPanel:kotaDisplay">
                        <table>
                            <tr>
                                <td><label>Session Time:</label></td>
                                <td><label>0 Day 0 h 0 m 0 s</label></td>
                            </tr>
                            <tr>
                                <td><label>Login Time:</label></td>
                                <td><label>31/05/2026 16:24:29</label></td>
                            </tr>
                        </table>
                        <label style="color:red;">Your quota is expired.</label>
                    </div>
                </center>
            </div>
        "#;

        let bilgi = bilgi_cek(html);

        assert!(bilgi.kota_doldu);
        assert!(!bilgi.kota.contains_key("toplam_mb"));
        assert!(!bilgi.kota.contains_key("kalan_mb"));
        assert_eq!(bilgi.isim, "TEST KULLANICI");
        assert_eq!(bilgi.konum, "ABC ÖĞRENCİ YURDU");
    }

    #[test]
    fn blok_bulunamazsa_bile_kota_ayiklanir() {
        let html = r#"
            <div id="content-div">
                <div class="wrapper">
                    <center>
                        <span class="myinfo">TEST KULLANICI</span>
                        <div id="mainPanel:kotaDisplay">
                            <table>
                                <tr>
                                    <td><label>Total Quota (MB):</label></td>
                                    <td><label>5000</label></td>
                                </tr>
                                <tr>
                                    <td><label>Total Remaining Quota (MB):</label></td>
                                    <td><label>0</label></td>
                                </tr>
                            </table>
                            <label style="color:red;">Your quota is expired.</label>
                        </div>
                    </center>
                </div>
            </div>
        "#;

        let bilgi = bilgi_cek(html);

        assert_eq!(
            bilgi.kota.get("toplam_mb").map(String::as_str),
            Some("5000")
        );
        assert_eq!(bilgi.kota.get("kalan_mb").map(String::as_str), Some("0"));
        assert!(
            bilgi.kota_doldu,
            "blok düzeni değişse de kota-doldu uyarısı görülmeli"
        );
    }

    #[test]
    fn buyuk_harfli_turkce_etiketler_ayiklanir() {
        let html = r#"
            <div id="content-div">
                <center>
                    <span class="myinfo">TEST KULLANICI</span>
                    <label class="myinfo">SON GİRİŞ: 25.04.2026 05:07</label>
                    <label class="myinfo">KONUM: ABC ÖĞRENCİ YURDU</label>
                </center>
            </div>
        "#;

        let bilgi = bilgi_cek(html);

        assert_eq!(bilgi.son_giris, "25.04.2026 05:07");
        assert_eq!(bilgi.konum, "ABC ÖĞRENCİ YURDU");
    }

    #[test]
    fn kota_anahtarlari_bosluk_farkina_dayanikli() {
        let html = r#"
            <div id="mainPanel:kotaDisplay">
                <table>
                    <tr>
                        <td><label>Total  Quota (MB):</label></td>
                        <td><label>5000</label></td>
                    </tr>
                    <tr>
                        <td><label>TOPLAM KALAN KOTA (MB):</label></td>
                        <td><label>1234</label></td>
                    </tr>
                </table>
            </div>
        "#;

        let bilgi = bilgi_cek(html);

        assert_eq!(
            bilgi.kota.get("toplam_mb").map(String::as_str),
            Some("5000")
        );
        assert_eq!(bilgi.kota.get("kalan_mb").map(String::as_str), Some("1234"));
    }
}
