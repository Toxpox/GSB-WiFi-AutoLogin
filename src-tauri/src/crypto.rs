use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::sync::OnceLock;

static ANAHTAR: OnceLock<[u8; 32]> = OnceLock::new();
static ESKI_ANAHTAR: OnceLock<[u8; 32]> = OnceLock::new();

fn anahtar_uret(girdi: &str) -> [u8; 32] {
    let mut anahtar = [0u8; 32];
    pbkdf2_hmac::<Sha256>(girdi.as_bytes(), b"gsb-salt-v1", 100_000, &mut anahtar);
    anahtar
}

fn makine_adi() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// v2 anahtari: makine adi + isletim sistemi kullanici adi. MAC adresine
/// bagli degildir; MAC rastgelelestirme veya adaptor degisikligi cozmeyi bozmaz.
fn anahtar() -> &'static [u8; 32] {
    ANAHTAR.get_or_init(|| {
        let kullanici = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "unknown".to_string());
        anahtar_uret(&format!(
            "gsb-wifi-autologin:v2:{}:{}",
            makine_adi(),
            kullanici
        ))
    })
}

/// v1 anahtari (MAC + makine adi). Yalnizca eski kayitlari cozebilmek icin
/// tutulur; yeni kayitlar her zaman v2 anahtariyla sifrelenir.
fn eski_anahtar() -> &'static [u8; 32] {
    ESKI_ANAHTAR.get_or_init(|| {
        let mac = mac_address::get_mac_address()
            .ok()
            .flatten()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        anahtar_uret(&format!("gsb-wifi-autologin:{}:{}", mac, makine_adi()))
    })
}

/// v3 (Windows DPAPI) kayitlari bu onekle isaretlenir; onek'siz kayitlar eski
/// AES-GCM (v2/v1) formatidir ve geriye donuk olarak hala cozulur.
const DPAPI_ONEK: &str = "v3:";

pub fn sifrele(metin: &str) -> Result<String, String> {
    if metin.is_empty() {
        return Ok(String::new());
    }
    // Windows'ta tercih: DPAPI (CryptProtectData) — anahtar isletim sistemi
    // tarafindan oturum/kullaniciya baglanir; makine adi+kullanici adindan
    // turetilen (yani yeniden uretilebilen) PBKDF2 anahtarindan daha guclu.
    // DPAPI kullanilamazsa AES-GCM v2'ye duser.
    #[cfg(windows)]
    {
        if let Ok(korunan) = dpapi::koru(metin.as_bytes()) {
            return Ok(format!("{}{}", DPAPI_ONEK, STANDARD.encode(&korunan)));
        }
    }
    aes_sifrele(metin)
}

fn aes_sifrele(metin: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(anahtar()).map_err(|e| e.to_string())?;
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let sifrelenmis = cipher
        .encrypt(nonce, metin.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut birlesik = nonce_bytes.to_vec();
    birlesik.extend_from_slice(&sifrelenmis);
    Ok(STANDARD.encode(&birlesik))
}

fn coz_ile(anahtar: &[u8; 32], sifreli: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(anahtar).map_err(|e| e.to_string())?;
    let ham = STANDARD.decode(sifreli).map_err(|e| e.to_string())?;
    if ham.len() < 12 {
        return Err("Veri cok kisa".into());
    }
    let (nonce_bytes, icerik) = ham.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let acik = cipher.decrypt(nonce, icerik).map_err(|e| e.to_string())?;
    String::from_utf8(acik).map_err(|e| e.to_string())
}

pub fn coz(sifreli: &str) -> Result<String, String> {
    if sifreli.is_empty() {
        return Ok(String::new());
    }
    // v3 (DPAPI) kaydi: onek'i ayikla, base64 coz, DPAPI ile ac.
    if let Some(b64) = sifreli.strip_prefix(DPAPI_ONEK) {
        #[cfg(windows)]
        {
            let ham = STANDARD.decode(b64).map_err(|e| e.to_string())?;
            let acik = dpapi::coz(&ham)?;
            return String::from_utf8(acik).map_err(|e| e.to_string());
        }
        #[cfg(not(windows))]
        {
            let _ = b64;
            return Err("DPAPI ile sifrelenmis kayit bu platformda cozulemez".into());
        }
    }
    // Eski (onek'siz) AES-GCM kayitlari: once v2 anahtari, sonra v1 fallback.
    coz_ile(anahtar(), sifreli).or_else(|_| coz_ile(eski_anahtar(), sifreli))
}

/// Windows Veri Koruma API'si (DPAPI) sarmalayicisi. Kimlik bilgisini oturum
/// acan Windows kullanicisina baglar; cozme yalnizca ayni kullanici hesabinda
/// mumkundur. Uygulamaya ozgu entropi ile baglanir.
#[cfg(windows)]
mod dpapi {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    const ENTROPI: &[u8] = b"gsb-wifi-autologin:v3:dpapi";

    fn blob(veri: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: veri.len() as u32,
            pbData: veri.as_ptr() as *mut u8,
        }
    }

    fn bos_blob() -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        }
    }

    /// Cikti blob'unu kopyalar ve API'nin ayirdigi bellegi LocalFree ile birakir.
    unsafe fn cikti_al(cikis: &CRYPT_INTEGER_BLOB) -> Vec<u8> {
        if cikis.pbData.is_null() || cikis.cbData == 0 {
            return Vec::new();
        }
        let dilim = std::slice::from_raw_parts(cikis.pbData, cikis.cbData as usize);
        let sonuc = dilim.to_vec();
        let _ = LocalFree(HLOCAL(cikis.pbData as *mut _));
        sonuc
    }

    pub fn koru(veri: &[u8]) -> Result<Vec<u8>, String> {
        let entropi = ENTROPI.to_vec();
        unsafe {
            let giris = blob(veri);
            let entropi_blob = blob(&entropi);
            let mut cikis = bos_blob();
            CryptProtectData(
                &giris,
                PCWSTR::null(),
                Some(&entropi_blob),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut cikis,
            )
            .map_err(|e| e.to_string())?;
            Ok(cikti_al(&cikis))
        }
    }

    pub fn coz(veri: &[u8]) -> Result<Vec<u8>, String> {
        let entropi = ENTROPI.to_vec();
        unsafe {
            let giris = blob(veri);
            let entropi_blob = blob(&entropi);
            let mut cikis = bos_blob();
            CryptUnprotectData(
                &giris,
                None,
                Some(&entropi_blob),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut cikis,
            )
            .map_err(|e| e.to_string())?;
            Ok(cikti_al(&cikis))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sifrele_coz_dongusu() {
        let orijinal = "12345678901";
        let sifrelenmis = sifrele(orijinal).unwrap();
        assert_ne!(sifrelenmis, orijinal);
        let acik = coz(&sifrelenmis).unwrap();
        assert_eq!(acik, orijinal);
    }

    #[test]
    fn bos_metin() {
        assert_eq!(sifrele("").unwrap(), "");
        assert_eq!(coz("").unwrap(), "");
    }

    #[test]
    fn eski_anahtarla_sifrelenen_veri_cozulur() {
        // v1 (MAC tabanli) anahtarla sifrelenmis veriyi simule et.
        let cipher = Aes256Gcm::new_from_slice(eski_anahtar()).unwrap();
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let sifrelenmis = cipher.encrypt(nonce, "eski-veri".as_bytes()).unwrap();
        let mut birlesik = nonce_bytes.to_vec();
        birlesik.extend_from_slice(&sifrelenmis);
        let kayit = STANDARD.encode(&birlesik);

        assert_eq!(coz(&kayit).unwrap(), "eski-veri");
    }

    #[test]
    fn bozuk_veri_hata_dondurur() {
        assert!(coz("gecersiz-base64!!").is_err());
        assert!(coz(&STANDARD.encode(b"kisa")).is_err());
    }
}
