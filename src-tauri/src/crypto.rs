use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

fn anahtar_turet() -> [u8; 32] {
    let mac = mac_address::get_mac_address()
        .ok()
        .flatten()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let host = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let girdi = format!("gsb-wifi-autologin:{}:{}", mac, host);
    let mut anahtar = [0u8; 32];
    pbkdf2_hmac::<Sha256>(girdi.as_bytes(), b"gsb-salt-v1", 100_000, &mut anahtar);
    anahtar
}

pub fn sifrele(metin: &str) -> Result<String, String> {
    if metin.is_empty() {
        return Ok(String::new());
    }
    let anahtar = anahtar_turet();
    let cipher = Aes256Gcm::new_from_slice(&anahtar).map_err(|e| e.to_string())?;
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let sifrelenmis = cipher
        .encrypt(nonce, metin.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut birlesik = nonce_bytes.to_vec();
    birlesik.extend_from_slice(&sifrelenmis);
    Ok(STANDARD.encode(&birlesik))
}

pub fn coz(sifreli: &str) -> Result<String, String> {
    if sifreli.is_empty() {
        return Ok(String::new());
    }
    let anahtar = anahtar_turet();
    let cipher = Aes256Gcm::new_from_slice(&anahtar).map_err(|e| e.to_string())?;
    let ham = STANDARD.decode(sifreli).map_err(|e| e.to_string())?;
    if ham.len() < 12 {
        return Err("Veri cok kisa".into());
    }
    let (nonce_bytes, icerik) = ham.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let acik = cipher
        .decrypt(nonce, icerik)
        .map_err(|e| e.to_string())?;
    String::from_utf8(acik).map_err(|e| e.to_string())
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
}
