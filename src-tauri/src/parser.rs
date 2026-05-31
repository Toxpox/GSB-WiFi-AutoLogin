use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::collections::HashMap;

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

fn kota_normalize(key: &str) -> &str {
    match key {
        "Toplam Kota (MB)" | "Total Quota (MB)" => "toplam_mb",
        "Toplam Kalan Kota (MB)" | "Total Remaining Quota (MB)" => "kalan_mb",
        "Yenilenme Tarihi" | "Next Refresh Date" => "yenilenme",
        "Oturum Süresi" | "Session Time" => "oturum_suresi",
        "Login Zamanı" | "Login Time" => "login_zamani",
        "Başlangıç Tarihi" | "Start Date" => "baslangic",
        "Sona Erme Tarihi" | "Expiration Date" => "bitis",
        "Kalan Kota Zamanı" | "Remaining Quota Time" => "kalan_zaman",
        other => other,
    }
}

pub fn bilgi_cek(html: &str) -> KullaniciBilgi {
    let document = Html::parse_document(html);
    let mut bilgi = KullaniciBilgi {
        isim: "Kullanıcı".into(),
        ..Default::default()
    };

    let sel = Selector::parse("#content-div > center").unwrap();
    let Some(blok) = document.select(&sel).next() else {
        return bilgi;
    };

    // span.myinfo -> isim
    let span_sel = Selector::parse("span.myinfo").unwrap();
    if let Some(span) = blok.select(&span_sel).next() {
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

    // label.myinfo -> son_giris, konum
    let label_sel = Selector::parse("label.myinfo").unwrap();
    for lbl in blok.select(&label_sel) {
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
        alan_ayikla(&mut bilgi, &txt);
    }

    // Fallback: tum label'lara bak
    if bilgi.konum.is_empty() || bilgi.son_giris.is_empty() {
        let all_label_sel = Selector::parse("label").unwrap();
        for lbl in blok.select(&all_label_sel) {
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
            alan_ayikla(&mut bilgi, &txt);
            if (bilgi.konum != onceki_konum || bilgi.son_giris != onceki_giris)
                && !bilgi.detaylar.contains(&txt)
            {
                bilgi.detaylar.push(txt);
            }
        }
    }

    bilgi.kota = kota_cek(&document);
    bilgi.kota_doldu = kota_doldu_mu(html);
    bilgi
}

/// Türkçe karakterleri ASCII'ye indirger ve küçültür; portal etiketlerini
/// locale (TR/EN) ve büyük/küçük harf farkından bağımsız eşleştirmek için.
fn turkce_kucult(metin: &str) -> String {
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

/// Kota bittiğinde portal kota satırlarını (Toplam/Kalan MB) göstermez; bunun
/// yerine "Your quota is expired." / "Kotanız doldu." gibi bir uyarı basar.
/// Bu fonksiyon o uyarıyı yakalayarak kotanın dolduğunu bildirir.
fn kota_doldu_mu(html: &str) -> bool {
    let metin = turkce_kucult(html);
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

fn alan_ayikla(bilgi: &mut KullaniciBilgi, txt: &str) {
    let lower = txt.to_lowercase();
    if bilgi.son_giris.is_empty()
        && (lower.contains("son giriş")
            || lower.contains("son giris")
            || lower.contains("last login"))
    {
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

    let td_sel = match Selector::parse("td") {
        Ok(s) => s,
        Err(_) => return kota,
    };

    let label_sel = match Selector::parse("label") {
        Ok(s) => s,
        Err(_) => return kota,
    };

    let tr_sel = match Selector::parse("#mainPanel\\:kotaDisplay tr") {
        Ok(s) => s,
        Err(_) => return kota,
    };

    for tr in document.select(&tr_sel) {
        let tds: Vec<_> = tr.select(&td_sel).collect();
        if tds.len() >= 2 {
            let key_label = tds[0].select(&label_sel).next();
            let val_label = tds[1].select(&label_sel).next();

            if let (Some(kl), Some(vl)) = (key_label, val_label) {
                let key = kl.text().collect::<String>().trim().to_string();
                let val = vl.text().collect::<String>().trim().to_string();

                if key.is_empty() || key == "------" || val.is_empty() || val == "------" {
                    continue;
                }

                let clean_key = key.trim_end_matches(':');
                let norm = kota_normalize(clean_key);
                kota.insert(norm.to_string(), val);
            }
        }
    }

    kota
}

pub fn maksimum_bilgi_cek(html: &str) -> crate::errors::CihazBilgisi {
    let document = Html::parse_document(html);
    let mut bilgi = crate::errors::CihazBilgisi::default();

    let sel = Selector::parse("#j_idt20_data tr td[role=gridcell]").unwrap();
    let hucreler: Vec<_> = document.select(&sel).collect();

    if hucreler.len() >= 3 {
        bilgi.baslangic = hucreler[0].text().collect::<String>().trim().to_string();
        bilgi.mac = hucreler[1].text().collect::<String>().trim().to_string();
        bilgi.konum = hucreler[2].text().collect::<String>().trim().to_string();
    }
    bilgi
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Boş/normal içerikte tetiklenmemeli.
        assert!(!kota_doldu_mu(
            r#"<label>Quota information is updated every 5 m.</label>"#
        ));
        // İngilizce portal uyarısı.
        assert!(kota_doldu_mu(
            r#"<label style="color:red;">Your quota is expired.</label>"#
        ));
        // Türkçe portal uyarısı (Türkçe karakterler dâhil).
        assert!(kota_doldu_mu(r#"<label>Kotanız doldu.</label>"#));
    }

    #[test]
    fn kota_doldugunda_isaretlenir() {
        // Kota bittiğinde MB satırları yoktur, yerine uyarı etiketi gelir.
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
}

pub fn maksimum_form_bilgi_cek(html: &str) -> FormBilgi {
    let document = Html::parse_document(html);

    let form_sel = Selector::parse("#j_idt20_data tr form").unwrap();
    let Some(form) = document.select(&form_sel).next() else {
        return FormBilgi::default();
    };

    let mut bilgi = form_temel_bilgi_cek(&form);

    let btn_sel = Selector::parse("button[type=submit]").unwrap();
    if let Some(btn) = form.select(&btn_sel).next() {
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

    let vs_sel = Selector::parse("input[name='javax.faces.ViewState']").unwrap();
    if let Some(vs) = form.select(&vs_sel).next() {
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
