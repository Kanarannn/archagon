
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

#[tauri::command]
pub fn quick_action(action: String) -> Result<String, String> {
    match action.as_str() {
        "web" => {
            spawn("/usr/bin/xdg-open", &["http://localhost"], None)?;
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
            let dir = Path::new("/srv/http");

            if !dir.is_dir() {
                return Err("Direktori /srv/http tidak ditemukan.".into());
            }

            let candidates: &[(&str, &[&str])] = &[
                ("kitty", &["--directory", "/srv/http"]),
                ("foot", &["--working-directory=/srv/http"]),
                ("alacritty", &["--working-directory", "/srv/http"]),
                ("konsole", &["--workdir", "/srv/http"]),
                ("xfce4-terminal", &["--working-directory=/srv/http"]),
                ("xterm", &["-e", "bash", "-lc", "cd /srv/http && exec bash"]),
            ];

            let mut opened = false;

            for (bin, args) in candidates {
                let available = Command::new("sh")
                    .args([
                        "-c",
                        "command -v \"$1\" >/dev/null 2>&1",
                        "sh",
                        bin,
                    ])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);

                if available && spawn(bin, args, Some(dir)).is_ok() {
                    opened = true;
                    break;
                }
            }

            if !opened {
                return Err(
                    "Terminal tidak ditemukan atau gagal dibuka. Pasang salah satu: kitty, foot, alacritty, konsole, xfce4-terminal, atau xterm."
                        .into(),
                );
            }
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
