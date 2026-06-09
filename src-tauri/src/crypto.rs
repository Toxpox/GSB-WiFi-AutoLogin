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

pub fn sifrele(metin: &str) -> Result<String, String> {
    if metin.is_empty() {
        return Ok(String::new());
    }
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
    coz_ile(anahtar(), sifreli).or_else(|_| coz_ile(eski_anahtar(), sifreli))
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
