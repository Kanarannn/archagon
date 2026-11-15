use serde::Serialize;
use std::process::Command;

const SERVICES: [&str; 2] = ["httpd", "mariadb"];

#[derive(Serialize)]
pub struct ServiceStatus {
    httpd: bool,
    mariadb: bool,
}

fn active(service: &str) -> bool {
    Command::new("/usr/bin/systemctl")
        .args(["is-active", "--quiet", service])
        .status().map(|s| s.success()).unwrap_or(false)
}

fn pkexec_systemctl(args: &[&str]) -> Result<String, String> {
    let output = Command::new("/usr/bin/pkexec")
        .arg("/usr/bin/systemctl")
        .args(args)
        .output()
        .map_err(|e| format!("Tidak bisa menjalankan pkexec: {e}. Pastikan polkit terpasang."))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if err.is_empty() { "Perintah dibatalkan atau gagal (Polkit/systemd).".into() } else { err })
    }
}

#[tauri::command]
pub fn get_service_status() -> ServiceStatus {
    ServiceStatus { httpd: active("httpd"), mariadb: active("mariadb") }
}

#[tauri::command]
pub fn start_all() -> Result<String, String> {
    pkexec_systemctl(&["start", "httpd", "mariadb"])?;
    Ok("Apache dan MariaDB berhasil diminta untuk start.".into())
}

#[tauri::command]
pub fn stop_all() -> Result<String, String> {
    pkexec_systemctl(&["stop", "httpd", "mariadb"])?;
    Ok("Apache dan MariaDB berhasil diminta untuk stop.".into())
}

fn validate_service(service: &str) -> Result<&'static str, String> {
    match service { "httpd" => Ok("httpd"), "mariadb" => Ok("mariadb"), _ => Err("Service tidak diizinkan.".into()) }
}

#[tauri::command]
pub fn start_service(service: String) -> Result<String, String> {
    let service = validate_service(&service)?;
    pkexec_systemctl(&["start", service])?;
    Ok(format!("{service} started"))
}
#[tauri::command]
pub fn stop_service(service: String) -> Result<String, String> {
    let service = validate_service(&service)?;
    pkexec_systemctl(&["stop", service])?;
    Ok(format!("{service} stopped"))
}
#[tauri::command]
pub fn reload_service(service: String) -> Result<String, String> {
    let service = validate_service(&service)?;
    pkexec_systemctl(&["reload", service])?;
    Ok(format!("{service} reloaded"))
}
