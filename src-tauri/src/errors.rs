use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Clone)]
#[serde(tag = "tip", content = "detay")]
pub enum GSBError {
    #[error("{kullanici_mesaji}")]
    AgHatasi {
        mesaj: String,
        kullanici_mesaji: String,
    },

    #[error("{kullanici_mesaji}")]
    DNSHatasi {
        host: String,
        kullanici_mesaji: String,
    },

    #[error("Sunucu yanitlamiyor")]
    ZamanAsimi,

    #[error("{kullanici_mesaji}")]
    GirisBasarisiz {
        mesaj: String,
        kullanici_mesaji: String,
    },

    #[error("Maksimum cihaz limitine ulasildi")]
    MaksimumCihaz {
        cihaz_bilgisi: CihazBilgisi,
        #[serde(skip_serializing)]
        html: String,
    },

    #[error("Portal yapisi degismis")]
    PortalDegisti,

    #[error("{kullanici_mesaji}")]
    AyarHatasi {
        mesaj: String,
        kullanici_mesaji: String,
    },
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct CihazBilgisi {
    pub baslangic: String,
    pub mac: String,
    pub konum: String,
}

impl From<GSBError> for String {
    fn from(e: GSBError) -> String {
        serde_json::to_string(&e).unwrap_or_else(|_| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hata_serialize() {
        let e = GSBError::AgHatasi {
            mesaj: "timeout".into(),
            kullanici_mesaji: "Ag hatasi".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("AgHatasi"));
    }
}
