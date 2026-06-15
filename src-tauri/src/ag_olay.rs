//! Olay-tabanli yeniden baglanma tetikleyicisi.
//!
//! Windows'un IP arayuz degisikligi bildirimine (`NotifyIpInterfaceChange`)
//! abone olur. Bir arayuz degistiginde (Wi-Fi baglanma/kopma, Ethernet,
//! DHCP yenileme) verilen `Notify` uyandirilir; yeniden baglanma dongusu
//! 12 saatlik aralik yerine olayi aninda yakalar. Boylece dusen GSB oturumu
//! saniyeler icinde geri acilir ve surec gercek bir ag degisikligi olana
//! kadar tamamen uyur (anketten daha az bos is).
//!
//! Kayit basarisiz olursa (ornegin API erisilemezse) sessizce vazgecilir;
//! 12 saatlik anket guvenlik agi olarak devrede kalir.

use std::sync::Arc;
use tokio::sync::Notify;

#[cfg(windows)]
pub fn ag_degisikligini_dinle(notify: Arc<Notify>) {
    use std::ffi::c_void;
    use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
    use windows::Win32::NetworkManagement::IpHelper::{
        NotifyIpInterfaceChange, MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE,
    };
    use windows::Win32::Networking::WinSock::AF_UNSPEC;

    unsafe extern "system" fn geri_cagir(
        baglam: *const c_void,
        _satir: *const MIB_IPINTERFACE_ROW,
        _tip: MIB_NOTIFICATION_TYPE,
    ) {
        if baglam.is_null() {
            return;
        }
        // baglam, sizdirilmis Arc<Notify>'in ham isaretcisidir; surec boyu gecerli.
        // notify_one herhangi bir is parcacigindan cagrilabilir (Notify: Sync).
        let notify = &*(baglam as *const Notify);
        notify.notify_one();
    }

    // Arc'i sizdir: isaretci, kayit suresince (surec omru) gecerli kalmali.
    let baglam = Arc::into_raw(notify) as *const c_void;
    let mut tutamac = HANDLE::default();

    // initialnotification = BOOLEAN(0): acilista bir kez tetiklenmesin; yalnizca
    // gercek arayuz degisikliklerinde uyandir.
    let sonuc = unsafe {
        NotifyIpInterfaceChange(
            AF_UNSPEC,
            Some(geri_cagir),
            Some(baglam),
            false,
            &mut tutamac,
        )
    };

    if sonuc != ERROR_SUCCESS {
        // Kayit basarisiz: Arc'i geri alip birak (sizinti olmasin). 12 saatlik
        // anket devrede kaldigi icin islevsel kayip olmaz.
        unsafe { drop(Arc::from_raw(baglam as *const Notify)) };
    }
    // Basariliysa `tutamac` bilincli olarak kapatilmaz: bildirimin surec boyu
    // acik kalmasi icin tutamaci serbest birakmiyoruz (CancelMibChangeNotify2
    // cagrilmadigi surece aktif kalir).
}

#[cfg(not(windows))]
pub fn ag_degisikligini_dinle(_notify: Arc<Notify>) {
    // Windows disi platformlarda yerel ag-olayi API'si yok; yeniden baglanma
    // dongusu yalnizca zamanlanmis aralikla calisir.
}
