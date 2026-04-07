use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
pub const VERSION: &str = "1.5.0";
#[allow(dead_code)]
pub const GIRIS_URL: &str = "https://wifi.gsb.gov.tr/j_spring_security_check";
pub const CIKIS_URL: &str = "https://wifi.gsb.gov.tr/cikisSon.html?logout=1";
pub const TIMEOUT_SECS: u64 = 15;
pub const MAX_DENEME: u32 = 3;
pub const BACKOFF_TABANI: f64 = 2.0;
pub const BACKOFF_CARPAN: f64 = 3.0;
pub const USER_AGENT: &str = concat!("GSB-WiFi-AutoLogin/", "1.5.0");

#[derive(Serialize, Deserialize, Default)]
pub struct KayitliKullanici {
    pub username: String,
    pub password: String,
    pub sifreli: bool,
}

fn ayar_yolu() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent()
        .unwrap_or(&PathBuf::from("."))
        .join("user_config.json")
}

pub fn kayitli_kullanici_al() -> (String, String) {
    let yol = ayar_yolu();
    let icerik = match fs::read_to_string(&yol) {
        Ok(s) => s,
        Err(_) => return (String::new(), String::new()),
    };
    let veri: KayitliKullanici = match serde_json::from_str(&icerik) {
        Ok(v) => v,
        Err(_) => return (String::new(), String::new()),
    };
    if veri.sifreli {
        let k = crate::crypto::coz(&veri.username).unwrap_or(veri.username);
        let s = crate::crypto::coz(&veri.password).unwrap_or(veri.password);
        (k, s)
    } else {
        (veri.username, veri.password)
    }
}

pub fn kullanici_kaydet(k: &str, sifre: &str) {
    if k.is_empty() {
        return;
    }
    let veri = KayitliKullanici {
        username: crate::crypto::sifrele(k).unwrap_or_else(|_| k.to_string()),
        password: crate::crypto::sifrele(sifre).unwrap_or_else(|_| sifre.to_string()),
        sifreli: true,
    };
    let json = serde_json::to_string(&veri).unwrap_or_default();
    let _ = fs::write(ayar_yolu(), json);
}

pub fn tc_maskele(tc: &str) -> String {
    if tc.len() >= 7 {
        format!("{}****{}", &tc[..3], &tc[tc.len() - 3..])
    } else {
        "***".to_string()
    }
}
