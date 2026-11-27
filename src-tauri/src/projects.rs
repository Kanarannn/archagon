use serde::Serialize;
use std::{fs, fs::OpenOptions, io::Write, path::{Path, PathBuf}};

const ROOT: &str = "/srv/http";
const VHOSTS: &str = "/etc/httpd/conf/extra/archagon-vhosts.conf";

#[derive(Serialize)]
pub struct Project {
    name: String,
    kind: &'static str,
    path: String,
}

fn project_dir(name: &str) -> Result<PathBuf, String> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err("Nama project tidak valid.".into());
    }
    let root = Path::new(ROOT);
    if !root.is_dir() { return Err("Direktori /srv/http tidak ditemukan.".into()); }
    let path = root.join(name);
    if !fs::symlink_metadata(&path).map(|m| m.file_type().is_dir()).unwrap_or(false) {
        return Err("Project tidak ditemukan di /srv/http.".into());
    }
    Ok(path)
}

fn project_type(path: &Path) -> &'static str {
    let Ok(entries) = fs::read_dir(path) else { return "Unknown" };
    let (mut laravel, mut wordpress, mut node, mut php, mut html) = (false, false, false, false, false);
    for entry in entries.flatten() {
        if !entry.file_type().map(|kind| kind.is_file()).unwrap_or(false) { continue; }
        let file = entry.file_name();
        let name = file.to_string_lossy();
        laravel |= name == "artisan";
        wordpress |= name == "wp-config.php";
        node |= name == "package.json";
        php |= name == "index.php" || entry.path().extension().is_some_and(|ext| ext == "php");
        html |= name == "index.html";
    }
    if laravel { "Laravel" } else if wordpress { "WordPress" } else if node { "Node.js" }
    else if php { "PHP" } else if html { "HTML" } else { "Unknown" }
}

#[tauri::command]
pub fn list_projects() -> Result<Vec<Project>, String> {
    let root = Path::new(ROOT);
    if !root.is_dir() { return Err("Direktori /srv/http tidak ditemukan.".into()); }
    let mut projects = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() { continue; }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else { continue };
        let path = entry.path();
        projects.push(Project { name, kind: project_type(&path), path: path.to_string_lossy().into_owned() });
    }
    projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(projects)
}

#[tauri::command]
pub fn create_project(name: String, kind: String) -> Result<(), String> {
    if name.len() > 50 || !name.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        || !name.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') {
        return Err("Nama harus dimulai dengan huruf atau angka kecil, lalu hanya boleh berisi huruf kecil, angka, atau tanda hubung (maks. 50 karakter).".into());
    }
    let file = match kind.as_str() {
        "blank" => None,
        "php" => Some(("index.php", "<?php\n\n")),
        "html" => Some(("index.html", "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>New project</title></head><body></body></html>\n")),
        _ => return Err("Jenis project tidak dikenal.".into()),
    };
    let root = Path::new(ROOT);
    if !root.is_dir() { return Err("Direktori /srv/http tidak ditemukan.".into()); }
    let path = root.join(&name);
    fs::create_dir(&path).map_err(|e| if e.kind() == std::io::ErrorKind::AlreadyExists { "Folder project sudah ada.".into() } else { format!("Gagal membuat project: {e}") })?;
    if let Some((filename, content)) = file {
        let file_path = path.join(filename);
        let mut output = OpenOptions::new().write(true).create_new(true).open(&file_path)
            .map_err(|e| format!("Gagal membuat file project: {e}"))?;
        if let Err(error) = output.write_all(content.as_bytes()) {
            let _ = fs::remove_file(file_path);
            let _ = fs::remove_dir(path);
            return Err(format!("Gagal membuat file project: {error}"));
        }
    }
    Ok(())
}

#[tauri::command]
pub fn open_project_folder(name: String) -> Result<(), String> {
    let path = project_dir(&name)?;
    std::process::Command::new("/usr/bin/xdg-open").arg(&path).spawn().map(|_| ()).map_err(|e| format!("Gagal membuka folder: {e}"))
}

#[tauri::command]
pub fn open_project_terminal(name: String) -> Result<(), String> {
    crate::system::open_terminal(&project_dir(&name)?)
}

fn url_segment(name: &str) -> String {
    let mut encoded = String::new();
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) { encoded.push(byte as char); }
        else { encoded.push_str(&format!("%{byte:02X}")); }
    }
    encoded
}

#[tauri::command]
pub fn open_project(name: String) -> Result<(), String> {
    project_dir(&name)?;
    let domain = format!("{name}.test");
    let configured = name.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        && fs::read_to_string(VHOSTS).map(|s| s.lines().any(|line| line.trim() == format!("ServerName {domain}"))).unwrap_or(false)
        && fs::read_to_string("/etc/hosts").map(|s| s.lines().any(|line| {
            let mut words = line.split('#').next().unwrap_or("").split_whitespace();
            words.next().is_some_and(|ip| ip == "127.0.0.1" || ip == "::1") && words.any(|host| host == domain)
        })).unwrap_or(false);
    let url = if configured { format!("http://{domain}") } else { format!("http://localhost/{}", url_segment(&name)) };
    std::process::Command::new("/usr/bin/xdg-open").arg(url).spawn().map(|_| ()).map_err(|e| format!("Gagal membuka browser: {e}"))
}
