use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::GSBError;

#[allow(dead_code)]
pub const VERSION: &str = "1.6.0";
#[allow(dead_code)]
pub const GIRIS_URL: &str = "https://wifi.gsb.gov.tr/j_spring_security_check";
pub const GITHUB_URL: &str = "https://github.com/Toxpox/GSB-WiFi-AutoLogin";
pub const GITHUB_RELEASE_LATEST_URL: &str =
    "https://api.github.com/repos/Toxpox/GSB-WiFi-AutoLogin/releases/latest";
pub const CIKIS_URL: &str = "https://wifi.gsb.gov.tr/cikisSon.html?logout=1";
pub const TIMEOUT_SECS: u64 = 15;
pub const MAX_DENEME: u32 = 3;
pub const BACKOFF_TABANI: f64 = 2.0;
pub const BACKOFF_CARPAN: f64 = 3.0;
pub const USER_AGENT: &str = concat!("GSB-WiFi-AutoLogin/", "1.6.0");

#[derive(Serialize, Deserialize, Default)]
pub struct KayitliKullanici {
    pub username: String,
    pub password: String,
    pub sifreli: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct KayitliProfil {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub sifreli: bool,
    #[serde(default)]
    pub son_kullanim: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ProfilAyarlari {
    #[serde(default = "profil_format_versiyonu")]
    pub version: u8,
    #[serde(default)]
    pub aktif_id: Option<String>,
    #[serde(default)]
    pub profiles: Vec<KayitliProfil>,
}

#[derive(Serialize)]
pub struct KullaniciProfiliOzet {
    pub id: String,
    pub masked_username: String,
    pub aktif: bool,
    pub son_kullanim: u64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AyarDosyasi {
    Tekil(KayitliKullanici),
    Profiller(ProfilAyarlari),
}

fn profil_format_versiyonu() -> u8 {
    2
}

fn ayar_hatasi(mesaj: impl ToString, kullanici_mesaji: &str) -> GSBError {
    GSBError::AyarHatasi {
        mesaj: mesaj.to_string(),
        kullanici_mesaji: kullanici_mesaji.into(),
    }
}

fn ayar_dizini() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("GSB WiFi AutoLogin")
}

fn ayar_yolu() -> Result<PathBuf, GSBError> {
    let dizin = ayar_dizini();
    fs::create_dir_all(&dizin).map_err(|e| ayar_hatasi(e, "Ayar klasoru olusturulamadi."))?;
    Ok(dizin.join("user_config.json"))
}

fn eski_ayar_yolu() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent()
        .unwrap_or(&PathBuf::from("."))
        .join("user_config.json")
}

pub fn kayitli_kullanici_al() -> (String, String) {
    let depo = profil_deposu_oku();
    let Some(profil) = aktif_profil(&depo) else {
        return (String::new(), String::new());
    };
    profil_coz(&profil)
}

pub fn kullanici_kaydet(k: &str, sifre: &str) -> Result<(), GSBError> {
    if k.is_empty() {
        return Ok(());
    }
    let mut depo = profil_deposu_oku();
    let id = profil_id_uret(k);
    let son_kullanim = simdiki_zaman();
    let yeni_profil = KayitliProfil {
        id: id.clone(),
        username: crate::crypto::sifrele(k)
            .map_err(|e| ayar_hatasi(e, "Kullanici bilgileri sifrelenemedi."))?,
        password: crate::crypto::sifrele(sifre)
            .map_err(|e| ayar_hatasi(e, "Kullanici bilgileri sifrelenemedi."))?,
        sifreli: true,
        son_kullanim,
    };

    if let Some(mevcut) = depo
        .profiles
        .iter_mut()
        .find(|p| p.id == id || profil_kullanici_adi(p) == k)
    {
        *mevcut = yeni_profil;
    } else {
        depo.profiles.push(yeni_profil);
    }

    depo.aktif_id = Some(id);
    depo.profiles
        .sort_by_key(|profil| std::cmp::Reverse(profil.son_kullanim));
    profil_deposu_yaz(&depo)
}

pub fn profilleri_listele() -> Vec<KullaniciProfiliOzet> {
    let depo = profil_deposu_oku();
    profil_ozetleri(&depo)
}

pub fn profil_yukle(id: &str) -> Result<(String, String), GSBError> {
    let mut depo = profil_deposu_oku();
    let Some(profil) = depo.profiles.iter().find(|p| p.id == id).cloned() else {
        return Err(ayar_hatasi(
            "Profil bulunamadi",
            "Secilen profil bulunamadi.",
        ));
    };
    let kullanici = profil_coz(&profil);
    depo.aktif_id = Some(profil.id);
    profil_deposu_yaz(&depo)?;
    Ok(kullanici)
}

pub fn profil_sil(id: &str) -> Result<Vec<KullaniciProfiliOzet>, GSBError> {
    let mut depo = profil_deposu_oku();
    depo.profiles.retain(|p| p.id != id);
    if depo.aktif_id.as_deref() == Some(id) {
        depo.aktif_id = depo.profiles.first().map(|p| p.id.clone());
    }
    profil_deposu_yaz(&depo)?;
    Ok(profil_ozetleri(&depo))
}

fn ayar_icerigi_oku() -> Option<String> {
    ayar_yolu()
        .ok()
        .and_then(|yol| fs::read_to_string(yol).ok())
        .or_else(|| fs::read_to_string(eski_ayar_yolu()).ok())
}

fn profil_deposu_oku() -> ProfilAyarlari {
    let Some(icerik) = ayar_icerigi_oku() else {
        return ProfilAyarlari::default();
    };

    match serde_json::from_str::<AyarDosyasi>(&icerik) {
        Ok(AyarDosyasi::Profiller(mut depo)) => {
            depo.version = profil_format_versiyonu();
            depo.profiles.retain(|p| !p.username.is_empty());
            for profil in &mut depo.profiles {
                if profil.id.is_empty() {
                    let username = profil_kullanici_adi(profil);
                    profil.id = profil_id_uret(&username);
                }
            }
            depo
        }
        Ok(AyarDosyasi::Tekil(veri)) => tekil_kayittan_depo(veri),
        Err(_) => ProfilAyarlari::default(),
    }
}

fn profil_deposu_yaz(depo: &ProfilAyarlari) -> Result<(), GSBError> {
    let json = serde_json::to_string_pretty(depo)
        .map_err(|e| ayar_hatasi(e, "Ayar dosyasi hazirlanamadi."))?;
    fs::write(ayar_yolu()?, json)
        .map_err(|e| ayar_hatasi(e, "Kullanici bilgileri kaydedilemedi."))?;
    Ok(())
}

fn tekil_kayittan_depo(veri: KayitliKullanici) -> ProfilAyarlari {
    let username = if veri.sifreli {
        crate::crypto::coz(&veri.username).unwrap_or_else(|_| veri.username.clone())
    } else {
        veri.username.clone()
    };
    if username.is_empty() {
        return ProfilAyarlari::default();
    }

    let id = profil_id_uret(&username);
    ProfilAyarlari {
        version: profil_format_versiyonu(),
        aktif_id: Some(id.clone()),
        profiles: vec![KayitliProfil {
            id,
            username: veri.username,
            password: veri.password,
            sifreli: veri.sifreli,
            son_kullanim: 0,
        }],
    }
}

fn aktif_profil(depo: &ProfilAyarlari) -> Option<KayitliProfil> {
    depo.aktif_id
        .as_deref()
        .and_then(|id| depo.profiles.iter().find(|p| p.id == id))
        .or_else(|| depo.profiles.first())
        .cloned()
}

fn profil_coz(profil: &KayitliProfil) -> (String, String) {
    if profil.sifreli {
        let k = crate::crypto::coz(&profil.username).unwrap_or_else(|_| profil.username.clone());
        let s = crate::crypto::coz(&profil.password).unwrap_or_else(|_| profil.password.clone());
        (k, s)
    } else {
        (profil.username.clone(), profil.password.clone())
    }
}

fn profil_kullanici_adi(profil: &KayitliProfil) -> String {
    profil_coz(profil).0
}

fn profil_ozetleri(depo: &ProfilAyarlari) -> Vec<KullaniciProfiliOzet> {
    let aktif_id = depo
        .aktif_id
        .as_deref()
        .or_else(|| depo.profiles.first().map(|p| p.id.as_str()));

    depo.profiles
        .iter()
        .filter_map(|profil| {
            let username = profil_kullanici_adi(profil);
            if username.is_empty() {
                return None;
            }
            Some(KullaniciProfiliOzet {
                id: profil.id.clone(),
                masked_username: tc_maskele(&username),
                aktif: aktif_id == Some(profil.id.as_str()),
                son_kullanim: profil.son_kullanim,
            })
        })
        .collect()
}

fn profil_id_uret(kullanici: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(kullanici.trim().as_bytes());
    hasher.finalize()[..8]
        .iter()
        .map(|b| format!("{:02x}", *b))
        .collect()
}

fn simdiki_zaman() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

pub fn tc_maskele(tc: &str) -> String {
    let chars: Vec<char> = tc.chars().collect();
    if chars.len() >= 7 {
        let bas: String = chars.iter().take(3).collect();
        let son: String = chars[chars.len() - 3..].iter().collect();
        format!("{}****{}", bas, son)
    } else {
        "***".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tekil_kayit_profil_deposuna_tasinir() {
        let depo = tekil_kayittan_depo(KayitliKullanici {
            username: "12345678901".into(),
            password: "secret".into(),
            sifreli: false,
        });

        assert_eq!(depo.version, 2);
        assert_eq!(depo.profiles.len(), 1);
        assert_eq!(depo.aktif_id.as_deref(), Some(depo.profiles[0].id.as_str()));
        assert_eq!(
            profil_coz(&depo.profiles[0]),
            ("12345678901".into(), "secret".into())
        );

        let ozetler = profil_ozetleri(&depo);
        assert_eq!(ozetler.len(), 1);
        assert_eq!(ozetler[0].masked_username, "123****901");
        assert!(ozetler[0].aktif);
    }

    #[test]
    fn profil_id_kullanici_adi_icin_kararlidir() {
        assert_eq!(
            profil_id_uret("12345678901"),
            profil_id_uret(" 12345678901 ")
        );
    }
}
