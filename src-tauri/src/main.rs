// GSB WiFi AutoLogin - Rust & Tauri
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ag_olay;
mod commands;
mod config;
mod crypto;
mod errors;
mod gunluk;
mod network;
mod parser;

use commands::AppState;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};

fn pencereyi_goster(app: &tauri::AppHandle) {
    if let Some(pencere) = app.get_webview_window("main") {
        let _ = pencere.show();
        let _ = pencere.unminimize();
        let _ = pencere.set_focus();
    }
}

fn tepsi_olustur(app: &tauri::App) -> tauri::Result<()> {
    let goster = MenuItem::with_id(app, "goster", "Pencereyi Göster", true, None::<&str>)?;
    let baglan = MenuItem::with_id(app, "baglan", "Bağlan", true, None::<&str>)?;
    let cikis_yap = MenuItem::with_id(app, "cikis-yap", "Çıkış Yap", true, None::<&str>)?;
    let ayrac = PredefinedMenuItem::separator(app)?;
    let kapat = MenuItem::with_id(app, "kapat", "Uygulamadan Çık", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&goster, &baglan, &cikis_yap, &ayrac, &kapat])?;

    let mut tepsi = TrayIconBuilder::with_id("ana-tepsi")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("GSB WiFi AutoLogin — Bağlı değil")
        .on_menu_event(|app, olay| match olay.id.as_ref() {
            "goster" => pencereyi_goster(app),
            "baglan" => {
                pencereyi_goster(app);
                let _ = app.emit("tepsi-baglan", ());
            }
            "cikis-yap" => {
                let _ = app.emit("tepsi-cikis", ());
            }
            "kapat" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tepsi, olay| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = olay
            {
                pencereyi_goster(tepsi.app_handle());
            }
        });

    if let Some(ikon) = app.default_window_icon() {
        tepsi = tepsi.icon(ikon.clone());
    }

    tepsi.build(app)?;
    Ok(())
}

fn main() {
    // Autostart girisinden "--sessiz" ile gelindiyse pencere acilmadan,
    // tepside baslar.
    let sessiz = std::env::args().any(|arg| arg == "--sessiz");

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--sessiz"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::new().expect("State olusturulamadi"))
        .setup(move |app| {
            tepsi_olustur(app)?;
            if !sessiz {
                pencereyi_goster(app.handle());
            }
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(commands::yeniden_baglanma_dongusu(handle));
            // Yerel ag-olayi dinleyicisi: IP arayuzu degisince yeniden baglanma
            // dongusunu aninda uyandirir (Wi-Fi baglandigi an giris denenir).
            ag_olay::ag_degisikligini_dinle(app.state::<AppState>().ag_olay.clone());
            Ok(())
        })
        .on_window_event(|pencere, olay| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = olay {
                if config::ayarlari_oku().tepsiye_kucul {
                    let _ = pencere.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::giris,
            commands::cikis,
            commands::bilgi_yenile,
            commands::kayitli_kullanici,
            commands::profilleri_listele,
            commands::profil_yukle,
            commands::profil_sil,
            commands::app_bilgisi,
            commands::github_ac,
            commands::github_link_ac,
            commands::yeni_versiyon_kontrol,
            commands::guncelleme_kontrol,
            commands::guncelleme_kur,
            commands::maksimum_cihaz_isle,
            commands::tc_maskele,
            commands::ayarlari_al,
            commands::ayarlari_kaydet,
            commands::gsb_aginda,
            commands::profil_takma_ad_ayarla,
            commands::log_satiri_yaz,
            commands::log_klasoru_ac,
            commands::kota_gecmisi_al,
            commands::tani_calistir,
        ])
        .run(tauri::generate_context!())
        .expect("Uygulama baslatilirken hata olustu");
}
