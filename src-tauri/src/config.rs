use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::GSBError;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PORTAL_HOST: &str = "wifi.gsb.gov.tr";
pub const GIRIS_URL: &str = "https://wifi.gsb.gov.tr/j_spring_security_check";
pub const GITHUB_URL: &str = "https://github.com/Toxpox/GSB-WiFi-AutoLogin";
pub const GITHUB_RELEASE_LATEST_URL: &str =
    "https://api.github.com/repos/Toxpox/GSB-WiFi-AutoLogin/releases/latest";
pub const INDEX_URL: &str = "https://wifi.gsb.gov.tr/index.html";
pub const LOGOUT_URL: &str = "https://wifi.gsb.gov.tr/logout";
pub const CIKIS_SON_URL: &str = "https://wifi.gsb.gov.tr/cikisSon.html?logout=1";
pub const TIMEOUT_SECS: u64 = 15;
pub const MAX_DENEME: u32 = 3;
pub const BACKOFF_TABANI: f64 = 2.0;
pub const BACKOFF_CARPAN: f64 = 3.0;
pub const USER_AGENT: &str = concat!("GSB-WiFi-AutoLogin/", env!("CARGO_PKG_VERSION"));
pub const PORTAL_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";

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
    profil_coz(&profil).unwrap_or_default()
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
        .find(|p| p.id == id || profil_kullanici_adi(p).as_deref() == Some(k))
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
    let Some(kullanici) = profil_coz(&profil) else {
        return Err(ayar_hatasi(
            "Profil cozulemedi",
            "Profil bilgileri bu cihazda cozulemedi. Profili silip yeniden kaydedin.",
        ));
    };
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
                    let username = profil_kullanici_adi(profil).unwrap_or_default();
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
    let yol = ayar_yolu()?;

    // Yazma sirasinda kesinti dosyayi bozmasin diye once gecici dosyaya
    // yazip uzerine tasiyoruz.
    let gecici = yol.with_extension("json.tmp");
    fs::write(&gecici, json).map_err(|e| ayar_hatasi(e, "Kullanici bilgileri kaydedilemedi."))?;
    fs::rename(&gecici, &yol).map_err(|e| ayar_hatasi(e, "Kullanici bilgileri kaydedilemedi."))?;

    // Yeni konuma basariyla yazildiysa exe yanindaki eski dosya artik gereksiz.
    let eski = eski_ayar_yolu();
    if eski != yol && eski.is_file() {
        let _ = fs::remove_file(eski);
    }
    Ok(())
}

fn tekil_kayittan_depo(veri: KayitliKullanici) -> ProfilAyarlari {
    let username = if veri.sifreli {
        match crate::crypto::coz(&veri.username) {
            Ok(u) => u,
            // Cozulemeyen eski kayit tasinmasin; sifreli metin kullanici adi
            // gibi gorunmesin.
            Err(_) => return ProfilAyarlari::default(),
        }
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

/// Profil bilgilerini cozer. Cozme basarisizsa (ornegin kayit baska bir
/// makinede sifrelendiyse) sifreli metni geri sizdirmamak icin None doner.
fn profil_coz(profil: &KayitliProfil) -> Option<(String, String)> {
    if profil.sifreli {
        let k = crate::crypto::coz(&profil.username).ok()?;
        let s = crate::crypto::coz(&profil.password).ok()?;
        Some((k, s))
    } else {
        Some((profil.username.clone(), profil.password.clone()))
    }
}

fn profil_kullanici_adi(profil: &KayitliProfil) -> Option<String> {
    profil_coz(profil).map(|(k, _)| k)
}

fn profil_ozetleri(depo: &ProfilAyarlari) -> Vec<KullaniciProfiliOzet> {
    let aktif_id = depo
        .aktif_id
        .as_deref()
        .or_else(|| depo.profiles.first().map(|p| p.id.as_str()));

    depo.profiles
        .iter()
        .filter_map(|profil| {
            let username = profil_kullanici_adi(profil)?;
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
    if chars.len() >= 10 {
        let bas: String = chars.iter().take(3).collect();
        let son: String = chars[chars.len() - 3..].iter().collect();
        format!("{}****{}", bas, son)
    } else if chars.len() >= 7 {
        // Kisa girdilerde daha az karakter aciga cikar.
        let bas: String = chars.iter().take(2).collect();
        let son: String = chars[chars.len() - 2..].iter().collect();
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
            Some(("12345678901".into(), "secret".into()))
        );

        let ozetler = profil_ozetleri(&depo);
        assert_eq!(ozetler.len(), 1);
        assert_eq!(ozetler[0].masked_username, "123****901");
        assert!(ozetler[0].aktif);
    }

    #[test]
    fn cozulemeyen_profil_listelenmez() {
        let depo = ProfilAyarlari {
            version: 2,
            aktif_id: None,
            profiles: vec![KayitliProfil {
                id: "abc".into(),
                username: "Z2VjZXJzaXotc2lmcmVsaS12ZXJp".into(),
                password: "Z2VjZXJzaXotc2lmcmVsaS12ZXJp".into(),
                sifreli: true,
                son_kullanim: 0,
            }],
        };

        assert!(profil_ozetleri(&depo).is_empty());
        assert_eq!(profil_coz(&depo.profiles[0]), None);
    }

    #[test]
    fn tc_maskeleme_uzunluga_gore_daralir() {
        assert_eq!(tc_maskele("12345678901"), "123****901");
        assert_eq!(tc_maskele("1234567"), "12****67");
        assert_eq!(tc_maskele("123456"), "***");
    }

    #[test]
    fn profil_id_kullanici_adi_icin_kararlidir() {
        assert_eq!(
            profil_id_uret("12345678901"),
            profil_id_uret(" 12345678901 ")
        );
    }
}
