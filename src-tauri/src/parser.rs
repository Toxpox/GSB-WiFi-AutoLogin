use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone, Default)]
pub struct KullaniciBilgi {
    pub isim: String,
    pub son_giris: String,
    pub konum: String,
    pub kota: HashMap<String, String>,
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
    bilgi
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

    let panel_sel =
        match Selector::parse("#mainPanel\\:kotaDisplay label.ui-outputlabel") {
            Ok(s) => s,
            Err(_) => return kota,
        };

    let td_sel = match Selector::parse("td") {
        Ok(s) => s,
        Err(_) => return kota,
    };

    let label_sel = match Selector::parse("label") {
        Ok(s) => s,
        Err(_) => return kota,
    };

    // Kota panelindeki label'lari topla
    let labels: Vec<_> = document.select(&panel_sel).collect();

    for label_el in &labels {
        let text = label_el
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        if text.is_empty() || text == "------" {
            continue;
        }

        // Parent td'yi bul, sibling td icindeki label'i al
        // scraper ile parent traversal sinirli oldugu icin
        // tum tr icindeki td ciftlerini isle
        let key = text.trim_end_matches(':');
        let norm = kota_normalize(key);

        // Bir sonraki label'i deger olarak kullanmak icin
        // labels dizisinde siradaki elemana bak
        // Bu basit yaklasim panel yapisina gore calisir
        if norm != key {
            // Bilinen bir anahtar bulduk, su anki index'in bir sonraki label'i deger
            // Bu mantik asagida pair olarak islenir
        }
        // Pair isleme: labels dizisinde index bazli degil, HTML yapisina gore
        // Daha saglam bir yaklasim: tr > td ciftleri
    }

    // Alternatif yaklasim: tr icindeki td ciftlerini isle
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
        bilgi.baslangic = hucreler[0]
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        bilgi.mac = hucreler[1]
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        bilgi.konum = hucreler[2]
            .text()
            .collect::<String>()
            .trim()
            .to_string();
    }
    bilgi
}

pub fn maksimum_form_bilgi_cek(html: &str) -> FormBilgi {
    let document = Html::parse_document(html);
    let mut bilgi = FormBilgi::default();

    let form_sel = Selector::parse("#j_idt20_data tr form").unwrap();
    let Some(form) = document.select(&form_sel).next() else {
        return bilgi;
    };

    bilgi.form_id = form.value().attr("id").unwrap_or("").to_string();

    let btn_sel = Selector::parse("button[type=submit]").unwrap();
    if let Some(btn) = form.select(&btn_sel).next() {
        bilgi.buton_id = btn.value().attr("id").unwrap_or("").to_string();
    }

    let vs_sel = Selector::parse("input[name='javax.faces.ViewState']").unwrap();
    if let Some(vs) = form.select(&vs_sel).next() {
        bilgi.viewstate = vs.value().attr("value").unwrap_or("").to_string();
    }
    bilgi
}
