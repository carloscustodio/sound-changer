fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![your_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn your_command() {
    // Call your audio manager functions here, including PowerShell commands
}