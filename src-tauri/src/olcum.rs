use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asama {
    Dns,
    TcpConnect,
    LoginDenemesi,
    LoginToplam,
    Body,
    Parse,
    YenidenBaglanmaToplam,
}

impl Asama {
    fn anahtar(self) -> &'static str {
        match self {
            Asama::Dns => "dns_ms",
            Asama::TcpConnect => "tcp_connect_ms",
            Asama::LoginDenemesi => "login_attempt_ms",
            Asama::LoginToplam => "login_total_ms",
            Asama::Body => "body_ms",
            Asama::Parse => "parse_ms",
            Asama::YenidenBaglanmaToplam => "reconnect_total_ms",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sonuc {
    Basarili,
    Basarisiz,
}

impl Sonuc {
    fn etiket(self) -> &'static str {
        match self {
            Sonuc::Basarili => "ok",
            Sonuc::Basarisiz => "hata",
        }
    }
}

pub fn olcum_satiri(asama: Asama, sure: Duration, sonuc: Sonuc, ek: Option<&str>) -> String {
    let temel = format!(
        "{}={} sonuc={}",
        asama.anahtar(),
        sure.as_millis(),
        sonuc.etiket()
    );
    match ek {
        Some(ek) if !ek.is_empty() => format!("{} {}", temel, ek),
        _ => temel,
    }
}

pub fn kaydet(asama: Asama, sure: Duration, sonuc: Sonuc, ek: Option<&str>) {
    crate::gunluk::yaz("olcum", &olcum_satiri(asama, sure, sonuc, ek));
}

pub struct AsamaOlcer {
    asama: Asama,
    baslangic: Instant,
}

impl AsamaOlcer {
    pub fn basla(asama: Asama) -> Self {
        Self {
            asama,
            baslangic: Instant::now(),
        }
    }

    pub fn gecen(&self) -> Duration {
        self.baslangic.elapsed()
    }

    pub fn bitir<T, E>(self, sonuc: &Result<T, E>) {
        let durum = if sonuc.is_ok() {
            Sonuc::Basarili
        } else {
            Sonuc::Basarisiz
        };
        kaydet(self.asama, self.gecen(), durum, None);
    }

    pub fn bitir_ek<T, E>(self, sonuc: &Result<T, E>, ek: &str) {
        let durum = if sonuc.is_ok() {
            Sonuc::Basarili
        } else {
            Sonuc::Basarisiz
        };
        kaydet(self.asama, self.gecen(), durum, Some(ek));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn olcum_satiri_beklenen_anahtarlari_tasir() {
        let satir = olcum_satiri(
            Asama::Dns,
            Duration::from_millis(1500),
            Sonuc::Basarili,
            None,
        );
        assert_eq!(satir, "dns_ms=1500 sonuc=ok");
    }

    #[test]
    fn basarisiz_olcum_isaretlenir() {
        let satir = olcum_satiri(
            Asama::LoginToplam,
            Duration::from_millis(53_000),
            Sonuc::Basarisiz,
            Some("retry_count=3"),
        );
        assert_eq!(satir, "login_total_ms=53000 sonuc=hata retry_count=3");
    }

    #[test]
    fn bos_ek_satiri_kirletmez() {
        let satir = olcum_satiri(
            Asama::Parse,
            Duration::from_millis(1),
            Sonuc::Basarili,
            Some(""),
        );
        assert_eq!(satir, "parse_ms=1 sonuc=ok");
    }

    #[test]
    fn tum_asamalar_benzersiz_anahtar_uretir() {
        let asamalar = [
            Asama::Dns,
            Asama::TcpConnect,
            Asama::LoginDenemesi,
            Asama::LoginToplam,
            Asama::Body,
            Asama::Parse,
            Asama::YenidenBaglanmaToplam,
        ];
        let mut anahtarlar: Vec<_> = asamalar.iter().map(|a| a.anahtar()).collect();
        anahtarlar.sort_unstable();
        let toplam = anahtarlar.len();
        anahtarlar.dedup();
        assert_eq!(anahtarlar.len(), toplam);
    }

    #[test]
    fn olcer_gecen_sureyi_raporlar() {
        let olcer = AsamaOlcer::basla(Asama::Body);
        std::thread::sleep(Duration::from_millis(5));
        assert!(olcer.gecen() >= Duration::from_millis(5));
    }

    #[test]
    fn olcum_satiri_log_bicimiyle_uyumlu() {
        let satir = olcum_satiri(
            Asama::LoginToplam,
            Duration::from_millis(1234),
            Sonuc::Basarili,
            Some("retry_count=0"),
        );
        assert!(!satir.contains('\n') && !satir.contains('\r'));
        assert!(satir.len() < 500);

        let alanlar: Vec<_> = satir.split(' ').collect();
        assert_eq!(alanlar.len(), 3);
        for alan in alanlar {
            assert!(
                alan.contains('='),
                "her alan anahtar=deger olmali: {}",
                alan
            );
        }
    }
}
