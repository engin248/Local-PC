#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    lokal_bilgisayar_kontrol_paneli::run();
}
