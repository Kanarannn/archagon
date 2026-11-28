
use std::{path::Path, process::Command};

fn spawn(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<(), String> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("Gagal membuka {program}: {e}"))
}

pub fn open_terminal(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("Direktori {} tidak ditemukan.", dir.display()));
    }

    let path = dir.to_string_lossy().into_owned();
    let candidates = vec![
        ("kitty", vec!["--directory".into(), path.clone()]),
        ("foot", vec![format!("--working-directory={path}")]),
        ("alacritty", vec!["--working-directory".into(), path.clone()]),
        ("konsole", vec!["--workdir".into(), path.clone()]),
        ("xfce4-terminal", vec![format!("--working-directory={path}")]),
        ("xterm", vec!["-e".into(), "bash".into(), "-lc".into(), format!("cd '{}' && exec bash", path.replace('\'', "'\\''"))]),
    ];
    for (bin, args) in candidates {
        let available = Command::new("sh")
            .args(["-c", "command -v \"$1\" >/dev/null 2>&1", "sh", bin])
            .status().map(|s| s.success()).unwrap_or(false);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        if available && spawn(bin, &args, Some(dir)).is_ok() { return Ok(()); }
    }
    Err("Terminal tidak ditemukan atau gagal dibuka. Pasang salah satu: kitty, foot, alacritty, konsole, xfce4-terminal, atau xterm.".into())
}

#[tauri::command]
pub fn quick_action(action: String) -> Result<String, String> {
    match action.as_str() {
        "web" => {
            spawn("/usr/bin/xdg-open", &["http://localhost"], None)?;
        }

        "releases" => {
            spawn("/usr/bin/xdg-open", &["https://github.com/Kanarannn/archagon/releases/latest"], None)?;
        }

        "database" => {
            spawn(
                "/usr/bin/xdg-open",
                &["http://localhost/phpmyadmin"],
                None,
            )?;
        }

        "root" => {
            spawn("/usr/bin/xdg-open", &["/srv/http"], None)?;
        }

        "terminal" => {
            open_terminal(Path::new("/srv/http"))?;
        }

        _ => return Err("Quick action tidak dikenal.".into()),
    }

    Ok("Aksi dibuka.".into())
}

#[tauri::command]
pub fn open_config(kind: String) -> Result<(), String> {
    let path = match kind.as_str() {
        "apache" => "/etc/httpd/conf/httpd.conf",
        "php" => "/etc/php/php.ini",
        "mariadb" => "/etc/my.cnf",
        _ => return Err("Konfigurasi tidak dikenal.".into()),
    };

    if !Path::new(path).exists() {
        return Err(format!("File {path} tidak ditemukan."));
    }

    spawn("/usr/bin/xdg-open", &[path], None)
}
