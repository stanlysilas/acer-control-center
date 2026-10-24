#!/usr/bin/env bash

set -euo pipefail

if [[ $EUID -ne 0 ]]; then
    echo "Please run this script with sudo."
    echo
    echo "Usage:"
    echo "sudo ./install.sh"
    exit 1
fi

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
DRIVER_DIR="$ROOT_DIR/driver"
DKMS_NAME="acer-keyboard"
DKMS_VERSION="0.1.0"
KERNEL_VERSION="$(uname -r)"
APP_ID="acer-control-center"
LIB_DIR="/usr/local/lib64"
MESON_INSTALL_ARGS=()

if [[ -f /etc/debian_version || -f /etc/arch-release ]]; then
    LIB_DIR="/usr/lib"
    MESON_INSTALL_ARGS=(--prefix=/usr --libdir=lib)
elif [[ -f /etc/redhat-release ]] || command -v zypper >/dev/null 2>&1; then
    LIB_DIR="/usr/lib64"
    MESON_INSTALL_ARGS=(--prefix=/usr --libdir=lib64)
fi

install_packages() {
    if [[ -f /etc/arch-release ]]; then
        pacman -Sy --noconfirm
        pacman -S --needed --noconfirm git gcc make dkms linux-headers mokutil openssl base-devel meson ninja rust cargo rustup curl ca-certificates pkgconf libx11 libxkbcommon wayland
    elif [[ -f /etc/debian_version ]]; then
        apt update
        apt install -y git gcc g++ make dkms mokutil openssl curl ca-certificates build-essential linux-headers-"$KERNEL_VERSION" meson ninja-build cargo rustc pkg-config libx11-dev libxkbcommon-dev libwayland-dev
    elif [[ -f /etc/redhat-release ]]; then
        dnf install -y git gcc gcc-c++ make dkms mokutil openssl curl ca-certificates kernel-devel kernel-headers meson ninja-build cargo rust pkgconf-pkg-config libX11-devel libxkbcommon-devel wayland-devel
    elif command -v zypper >/dev/null 2>&1; then
        zypper install -y git gcc gcc-c++ make dkms mokutil openssl curl ca-certificates kernel-devel kernel-headers meson ninja cargo rust pkg-config libX11-devel libxkbcommon-devel wayland-devel
    else
        echo "Unsupported Linux distribution."
        exit 1
    fi
}

rustc_is_new_enough() {
    local version major minor
    version="$(rustc --version 2>/dev/null | awk '{print $2}')"
    [[ "$version" =~ ^([0-9]+)\.([0-9]+) ]] || return 1
    major="${BASH_REMATCH[1]}"
    minor="${BASH_REMATCH[2]}"
    (( major > 1 || (major == 1 && minor >= 88) ))
}

ensure_rust_toolchain() {
    if rustc_is_new_enough; then
        echo "Rust $(rustc --version | awk '{print $2}') meets the application requirement (1.88+)."
        return
    fi

    echo "The distribution Rust package is older than 1.88; installing the stable Rust toolchain under /opt/acer-control-center."
    export RUSTUP_HOME="/opt/acer-control-center/rustup"
    export CARGO_HOME="/opt/acer-control-center/cargo"
    export PATH="$CARGO_HOME/bin:$PATH"
    mkdir -p "$RUSTUP_HOME" "$CARGO_HOME"

    if [[ ! -x "$CARGO_HOME/bin/rustup" ]]; then
        local installer
        installer="$(mktemp)"
        curl --fail --location --silent --show-error https://sh.rustup.rs --output "$installer"
        sh "$installer" -y --no-modify-path --profile minimal --default-toolchain stable
        rm -f "$installer"
    fi

    rustup toolchain install stable --profile minimal
    export RUSTUP_TOOLCHAIN=stable
    if ! rustc_is_new_enough; then
        echo "ERROR: Rust 1.88 or newer is required to build Acer Control Center." >&2
        exit 1
    fi
    echo "Using $(rustc --version)."
}

prepare_dkms() {
    echo
    echo "==> Preparing DKMS driver..."

    [[ -f "$DRIVER_DIR/dkms.conf" ]] || { echo "ERROR: $DRIVER_DIR/dkms.conf was not found."; exit 1; }

    make -C "$DRIVER_DIR" clean || true

    if dkms status -m "$DKMS_NAME" -v "$DKMS_VERSION" >/dev/null 2>&1; then
        echo "==> Removing previous $DKMS_NAME/$DKMS_VERSION DKMS registration..."
        dkms remove -m "$DKMS_NAME" -v "$DKMS_VERSION" --all || true
    fi

    echo "==> Adding $DKMS_NAME/$DKMS_VERSION to DKMS..."
    dkms add "$DRIVER_DIR"

    echo "==> Installing DKMS driver for kernel $KERNEL_VERSION..."
    dkms install -m "$DKMS_NAME" -v "$DKMS_VERSION" -k "$KERNEL_VERSION"
    depmod -a "$KERNEL_VERSION"
}

prepare_mok() {
    local mok_key="/var/lib/dkms/mok.key"
    local mok_cert="/var/lib/dkms/mok.pub"

    echo
    echo "==> Checking kernel module signing..."

    if [[ ! -f "$mok_key" || ! -f "$mok_cert" ]]; then
        echo "==> Generating the persistent DKMS module signing key..."
        dkms generate_mok
    fi

    if [[ ! -f "$mok_key" || ! -f "$mok_cert" ]]; then
        echo "WARNING: DKMS signing key was not found."
        echo "Installation will continue, but Secure Boot may reject the module."
        return 0
    fi

    if ! mokutil --sb-state 2>&1 | grep -qi 'SecureBoot enabled'; then
        echo "Secure Boot is disabled or its state is unavailable; MOK enrollment is not needed now."
        return 0
    fi

    if mok_certificate_in_list "$mok_cert" --list-enrolled; then
        echo "MOK signing certificate is already enrolled."
        return 0
    fi

    if mok_certificate_in_list "$mok_cert" --list-new; then
        echo "MOK signing certificate enrollment is already queued for the next boot."
        return 0
    fi

    echo
    echo "The DKMS signing certificate is not currently enrolled."
    echo
    echo "Enrolling it allows this driver to load when Secure Boot is enabled."
    echo "The installer will NOT reboot the system."
    echo "You can complete the enrollment during a later boot."
    echo

    while true; do
        read -r -p "Queue MOK enrollment for the next boot? [Y/n]: " answer
        answer="${answer:-Y}"

        case "$answer" in
            [Yy]|[Yy][Ee][Ss])
                mokutil --import "$mok_cert"
                echo
                echo "MOK enrollment has been queued."
                echo "Complete the enrollment through MOK Manager during your next boot."
                echo "The installer will continue."
                return 0
                ;;
            [Nn]|[Nn][Oo])
                echo
                echo "MOK enrollment skipped."
                echo "Installation will continue."
                echo "WARNING: If Secure Boot is enabled, the Acer kernel module may be rejected."
                return 0
                ;;
            *)
                echo "Please answer Y or N."
                ;;
        esac
    done
}

mok_certificate_in_list() {
    local certificate="$1"
    local mokutil_option="$2"
    local fingerprint

    fingerprint="$(openssl x509 -in "$certificate" -noout -fingerprint -sha1 2>/dev/null \
        | sed 's/^[^=]*=//' | tr '[:lower:]' '[:upper:]')" || return 1
    [[ -n "$fingerprint" ]] || return 1
    mokutil "$mokutil_option" 2>/dev/null \
        | tr '[:lower:]' '[:upper:]' \
        | grep -Fq "SHA1 FINGERPRINT: $fingerprint"
}

install_packages
ensure_rust_toolchain
if ! getent group acercontrol >/dev/null 2>&1; then
    groupadd --system acercontrol
fi
# Create/check the signing key before DKMS builds modules, so the build can
# sign them with the same certificate whose enrollment is checked here.
prepare_mok
prepare_dkms

echo
echo "==> Building libacercontrol..."

if [[ -d "$ROOT_DIR/lib/build" ]]; then
    meson setup "$ROOT_DIR/lib/build" "$ROOT_DIR/lib" --reconfigure "${MESON_INSTALL_ARGS[@]}"
else
    meson setup "$ROOT_DIR/lib/build" "$ROOT_DIR/lib" "${MESON_INSTALL_ARGS[@]}"
fi

meson compile -C "$ROOT_DIR/lib/build"

# Install the versioned shared library and linker symlinks where both the
# runtime linker and the GUI build can find them on this distribution.
meson install -C "$ROOT_DIR/lib/build"
ldconfig

echo
echo "==> Building daemon..."
ACERCONTROL_LIB_DIR="$LIB_DIR" ACERCONTROL_RUNTIME_LIB_DIR="$LIB_DIR" \
    cargo build --release --manifest-path "$ROOT_DIR/daemon/Cargo.toml"
install -Dm755 "$ROOT_DIR/daemon/target/release/acer-control-daemon" /usr/bin/acer-control-daemon

echo
echo "==> Building GUI..."
ACERCONTROL_LIB_DIR="$LIB_DIR" ACERCONTROL_RUNTIME_LIB_DIR="$LIB_DIR" \
    cargo build --release --manifest-path "$ROOT_DIR/gui/Cargo.toml"
install -Dm755 "$ROOT_DIR/gui/target/release/acer-control-center" /usr/bin/acer-control-center

echo
echo "==> Installing GUI resources..."
install -d /usr/share/acer-control-center
cp -r "$ROOT_DIR/gui/resources/." /usr/share/acer-control-center/

echo
echo "==> Installing udev rules..."
install -Dm644 "$ROOT_DIR/packaging/udev/99-acercontrol.rules" /etc/udev/rules.d/99-acercontrol.rules
install -Dm644 "$ROOT_DIR/packaging/modules-load/acer-control-center.conf" /usr/lib/modules-load.d/acer-control-center.conf
udevadm control --reload-rules
udevadm trigger

for module in acer_io acer_keyboard acer_wmi acer_acpi; do
    modprobe "$module" || true
done

echo
echo "==> Installing D-Bus policy..."
install -Dm644 "$ROOT_DIR/packaging/dbus/io.github.stanlysilas.AcerControl.conf" /usr/share/dbus-1/system.d/io.github.stanlysilas.AcerControl.conf

busctl call --system org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig >/dev/null || true

echo
echo "==> Installing systemd service..."
install -Dm644 "$ROOT_DIR/packaging/systemd/acer-control-daemon.service" /usr/lib/systemd/system/acer-control-daemon.service
systemctl daemon-reload
systemctl enable acer-control-daemon.service
systemctl restart acer-control-daemon.service

echo
echo "==> Adding desktop entry..."
install -Dm644 "$ROOT_DIR/gui/resources/icons/acer-control-center.svg" \
    "/usr/share/icons/hicolor/scalable/apps/$APP_ID.svg"
install -Dm644 "$ROOT_DIR/gui/resources/icons/acer-control-center.png" \
    "/usr/share/icons/hicolor/128x128/apps/$APP_ID.png"
rm -f "/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"
install -Dm644 "$ROOT_DIR/packaging/desktop/$APP_ID.desktop" \
    "/usr/share/applications/$APP_ID.desktop"
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi

echo
echo "=========================================="
echo " Acer Control Center installed successfully"
echo "=========================================="
echo

if [[ -f /var/lib/dkms/mok.pub ]] && mok_certificate_in_list /var/lib/dkms/mok.pub --list-enrolled; then
    echo "MOK status: enrolled."
elif [[ -f /var/lib/dkms/mok.pub ]] && mok_certificate_in_list /var/lib/dkms/mok.pub --list-new; then
    echo "MOK status: enrollment is queued for the next boot."
elif ! mokutil --sb-state 2>&1 | grep -qi 'SecureBoot enabled'; then
    echo "MOK status: Secure Boot is disabled; enrollment is not needed now."
else
    echo "IMPORTANT: MOK enrollment is pending."
    echo "Reboot when convenient and complete the MOK Manager enrollment."
    echo "The installer has NOT rebooted the system."
fi

echo
echo "A reboot is recommended before first use."
echo "If MOK enrollment is pending, the reboot is required to complete it."
echo
echo "Launch the GUI using:"
echo
echo "acer-control-center"
echo
echo " or from the Applications list"
