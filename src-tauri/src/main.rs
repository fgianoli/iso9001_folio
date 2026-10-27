// Su Windows non apre la finestra nera del terminale
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    commesse_app_lib::run()
}
