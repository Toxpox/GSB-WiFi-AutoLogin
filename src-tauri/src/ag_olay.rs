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

        let notify = &*(baglam as *const Notify);
        notify.notify_one();
    }

    let baglam = Arc::into_raw(notify) as *const c_void;
    let mut tutamac = HANDLE::default();

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
        unsafe { drop(Arc::from_raw(baglam as *const Notify)) };
    }
}

#[cfg(not(windows))]
pub fn ag_degisikligini_dinle(_notify: Arc<Notify>) {}
