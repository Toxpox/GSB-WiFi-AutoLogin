use crate::config;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_BAYT: u64 = 1024 * 1024;
const MAX_SATIR_KARAKTER: usize = 500;

static KILIT: Mutex<()> = Mutex::new(());

pub fn log_dosya_yolu() -> PathBuf {
    config::log_dizini().join("uygulama.log")
}

fn satir_bicimle(zaman: &str, tip: &str, mesaj: &str) -> String {
    let temiz: String = mesaj
        .chars()
        .map(|c| if c == '\r' || c == '\n' { ' ' } else { c })
        .take(MAX_SATIR_KARAKTER)
        .collect();
    format!("[{}] [{}] {}", zaman, tip, temiz)
}

pub fn yaz(tip: &str, mesaj: &str) {
    let _kilit = KILIT.lock().unwrap_or_else(|e| e.into_inner());
    let zaman = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let satir = satir_bicimle(&zaman, tip, mesaj);
    let _ = yaz_ic(&satir);
}

fn yaz_ic(satir: &str) -> std::io::Result<()> {
    let dizin = config::log_dizini();
    fs::create_dir_all(&dizin)?;
    let yol = log_dosya_yolu();
    dondur_gerekirse(&yol);

    let mut dosya = OpenOptions::new().create(true).append(true).open(&yol)?;
    writeln!(dosya, "{}", satir)
}

fn dondur_gerekirse(yol: &Path) {
    let Ok(meta) = fs::metadata(yol) else {
        return;
    };
    if meta.len() <= MAX_BAYT {
        return;
    }
    let yedek = yol.with_extension("log.1");
    let _ = fs::remove_file(&yedek);
    if fs::rename(yol, &yedek).is_err() {
        let _ = fs::write(yol, b"");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn satir_tek_satira_indirgenir() {
        let satir = satir_bicimle("2026-06-10 12:00:00", "hata", "ilk\nikinci\rsatir");
        assert_eq!(satir, "[2026-06-10 12:00:00] [hata] ilk ikinci satir");
    }

    #[test]
    fn uzun_satir_kirpilir() {
        let uzun = "a".repeat(2 * MAX_SATIR_KARAKTER);
        let satir = satir_bicimle("z", "bilgi", &uzun);
        assert_eq!(satir.len(), "[z] [bilgi] ".len() + MAX_SATIR_KARAKTER);
    }

    #[test]
    fn test_logu_gercek_kullanici_dizinine_yazmaz() {
        let yol = log_dosya_yolu();
        assert!(
            yol.starts_with(std::env::temp_dir()),
            "test logu gecici dizinde olmali, gelen: {}",
            yol.display()
        );
        assert!(
            !yol.to_string_lossy().contains("GSB WiFi AutoLogin"),
            "test kosumu gercek kullanici log dosyasini kirletmemeli"
        );
    }

    #[test]
    fn yazilan_satir_dosyada_okunabilir() {
        yaz("olcum", "login_total_ms=1234 sonuc=ok retry_count=0");

        let icerik = fs::read_to_string(log_dosya_yolu()).expect("log dosyasi olusmaliydi");
        assert!(
            icerik.contains("[olcum] login_total_ms=1234 sonuc=ok retry_count=0"),
            "olcum satiri log dosyasina yazilmali, dosya: {}",
            icerik
        );
    }
}
