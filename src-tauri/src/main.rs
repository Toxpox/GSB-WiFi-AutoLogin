// GSB WiFi AutoLogin - Rust & Tauri
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod crypto;
mod errors;
mod network;
mod parser;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new().expect("State olusturulamadi"))
        .invoke_handler(tauri::generate_handler![
            commands::giris,
            commands::cikis,
            commands::kayitli_kullanici,
            commands::maksimum_cihaz_isle,
            commands::tc_maskele,
        ])
        .run(tauri::generate_context!())
        .expect("Uygulama baslatilirken hata olustu");
}
