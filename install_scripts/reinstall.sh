#!/usr/bin/env bash
#
# Archagon installer / reinstaller untuk Arch Linux.
#
# Pemakaian:
#   sudo ./install_scripts/reinstall.sh
#   sudo ./install_scripts/reinstall.sh --uninstall
#   sudo ./install_scripts/reinstall.sh --skip-check   # lewati validasi build
#
# Untuk pengujian tanpa root (mis. packaging):
#   DESTDIR=/tmp/archagon-test ./install_scripts/reinstall.sh
#
set -euo pipefail

APP_NAME="archagon"
APP_TITLE="Archagon"
APP_COMMENT="Arch Linux local development server manager"

PROJECT_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$PROJECT_ROOT/src-tauri/target/release/archagon"
ICON_DIR="$PROJECT_ROOT/src-tauri/icons"

DESTDIR="${DESTDIR:-}"
BIN_DEST="${DESTDIR}/usr/local/bin/${APP_NAME}"
DESKTOP_DIR="${DESTDIR}/usr/share/applications"
DESKTOP_DEST="${DESKTOP_DIR}/${APP_NAME}.desktop"
ICON_ROOT="${DESTDIR}/usr/share/icons/hicolor"

# pasangan: file sumber di src-tauri/icons -> direktori tema hicolor
ICON_MAP=(
  "32x32.png:32x32"
  "64x64.png:64x64"
  "128x128.png:128x128"
  "128x128@2x.png:256x256@2"
  "icon.png:512x512"
)

MODE="install"
SKIP_CHECK=0

if [[ -t 1 ]]; then
  C_RESET=$'\033[0m'; C_BLUE=$'\033[1;34m'; C_GREEN=$'\033[1;32m'
  C_YELLOW=$'\033[1;33m'; C_RED=$'\033[1;31m'
else
  C_RESET=""; C_BLUE=""; C_GREEN=""; C_YELLOW=""; C_RED=""
fi

info() { printf '%s==>%s %s\n' "$C_BLUE" "$C_RESET" "$*"; }
ok()   { printf '%s  ok%s %s\n' "$C_GREEN" "$C_RESET" "$*"; }
warn() { printf '%s  !!%s %s\n' "$C_YELLOW" "$C_RESET" "$*" >&2; }
die()  { printf '%sError:%s %s\n' "$C_RED" "$C_RESET" "$*" >&2; exit 1; }

usage() {
  cat <<EOF
Pemakaian: sudo $0 [opsi]

Opsi:
  --uninstall    Hapus Archagon dari sistem
  --skip-check   Lewati validasi bahwa binary adalah build produksi
  -h, --help     Tampilkan bantuan ini

Lingkungan:
  DESTDIR=path   Pasang ke root alternatif (untuk pengujian/packaging,
                 tidak memerlukan root dan tidak memperbarui cache sistem)
EOF
}

for arg in "$@"; do
  case "$arg" in
    --uninstall|uninstall|-u) MODE="uninstall" ;;
    --skip-check)             SKIP_CHECK=1 ;;
    -h|--help|help)           usage; exit 0 ;;
    *)                        die "Opsi tidak dikenal: $arg (gunakan --help)" ;;
  esac
done

if [[ -z "$DESTDIR" && $EUID -ne 0 ]]; then
  die "Jalankan dengan root: sudo $0 ${*:-}"
fi

update_caches() {
  if [[ -n "$DESTDIR" ]]; then
    return 0
  fi

  info "Memperbarui cache ikon & desktop database"
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$ICON_ROOT" >/dev/null 2>&1 || warn "gtk-update-icon-cache gagal (abaikan)"
  fi
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || warn "update-desktop-database gagal (abaikan)"
  fi
}

uninstall() {
  info "Menghapus Archagon dari sistem"
  rm -f -- "$BIN_DEST" "$DESKTOP_DEST"
  local entry src dest
  for entry in "${ICON_MAP[@]}"; do
    dest="${entry##*:}"
    rm -f -- "$ICON_ROOT/${dest}/apps/${APP_NAME}.png"
  done
  update_caches
  ok "Archagon dihapus."
}

check_build() {
  [[ -f "$BINARY" ]] || die "Binary tidak ditemukan: $BINARY
Build dulu dengan: cd '$PROJECT_ROOT' && npm run tauri build"
  [[ -x "$BINARY" ]] || die "Binary tidak executable: $BINARY"

  if (( SKIP_CHECK == 0 )); then
    # Build produksi meng-embed aset frontend (mis. /assets/index-xxxx.js).
    # Build dev tidak, sehingga app akan mencoba devUrl dan gagal connect.
    if ! grep -aqE '/assets/index-[A-Za-z0-9_-]+\.(js|css)' "$BINARY"; then
      die "Binary ini bukan build produksi (mode dev / devUrl).
Build ulang dengan: cd '$PROJECT_ROOT' && npm run tauri build
(atau jalankan ulang dengan --skip-check jika memang disengaja)"
    fi
  fi

  # Peringatan (bukan error) jika sumber lebih baru daripada binary.
  if find "$PROJECT_ROOT/src" "$PROJECT_ROOT/index.html" \
          "$PROJECT_ROOT/src-tauri/src" "$PROJECT_ROOT/src-tauri/icons" \
          -newer "$BINARY" -print -quit 2>/dev/null | grep -q .; then
    warn "Ada source yang lebih baru daripada binary. Pertimbangkan rebuild:"
    warn "  npm run tauri build"
  fi
}

install_app() {
  info "Memeriksa build"
  check_build
  ok "Binary: $BINARY"

  info "Memasang binary"
  mkdir -p -- "$(dirname -- "$BIN_DEST")"
  # Pasang ke file sementara lalu rename agar aman walau app sedang berjalan.
  install -m 755 "$BINARY" "${BIN_DEST}.new"
  mv -f -- "${BIN_DEST}.new" "$BIN_DEST"
  ok "$BIN_DEST"

  info "Memasang ikon"
  local entry src dest
  for entry in "${ICON_MAP[@]}"; do
    src="${entry%%:*}"
    dest="${entry##*:}"
    [[ -f "$ICON_DIR/$src" ]] || die "Ikon tidak ditemukan: $ICON_DIR/$src"
    mkdir -p -- "$ICON_ROOT/${dest}/apps"
    install -m 644 "$ICON_DIR/$src" "$ICON_ROOT/${dest}/apps/${APP_NAME}.png"
  done
  ok "Ikon hicolor terpasang"

  info "Memasang desktop entry"
  mkdir -p -- "$DESKTOP_DIR"
  cat > "$DESKTOP_DEST" <<EOF
[Desktop Entry]
Type=Application
Name=${APP_TITLE}
GenericName=Local Server Manager
Comment=${APP_COMMENT}
Exec=/usr/local/bin/${APP_NAME}
TryExec=/usr/local/bin/${APP_NAME}
Icon=${APP_NAME}
Terminal=false
Categories=Development;WebDevelopment;
Keywords=apache;mariadb;mysql;php;localhost;server;
StartupNotify=true
StartupWMClass=${APP_NAME}
EOF
  chmod 644 "$DESKTOP_DEST"

  if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate "$DESKTOP_DEST" || warn "desktop-file-validate melaporkan masalah"
  fi
  ok "$DESKTOP_DEST"

  update_caches

  if [[ -z "$DESTDIR" ]] && pgrep -x "$APP_NAME" >/dev/null 2>&1; then
    warn "Archagon sedang berjalan. Tutup lalu buka ulang untuk memakai versi baru."
  fi

  printf '\n%sSelesai!%s Archagon berhasil dipasang.\n' "$C_GREEN" "$C_RESET"
  printf 'Jalankan dari menu aplikasi atau dengan perintah: %s\n\n' "$APP_NAME"
  printf 'Catatan:\n'
  printf '  - Jangan jalankan Archagon sebagai root. Tombol service memakai pkexec/Polkit.\n'
  printf '  - Launcher/shell yang sudah berjalan bisa meng-cache ikon. Jika ikon belum muncul,\n'
  printf '    restart shell/launcher atau logout lalu login.\n'
}

case "$MODE" in
  install)   install_app ;;
  uninstall) uninstall ;;
esac
