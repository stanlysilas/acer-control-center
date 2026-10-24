#!/usr/bin/env bash

set -e

APP_ID="acer-control-center"
DKMS_NAME="acer-keyboard"

if [[ $EUID -ne 0 ]]; then
    echo "Please run this script with sudo."
    exit 1
fi

echo "=========================================="
echo " Acer Control Center Uninstaller"
echo "=========================================="
echo

echo "==> Stopping daemon..."

systemctl stop acer-control-daemon.service 2>/dev/null || true
systemctl disable acer-control-daemon.service 2>/dev/null || true

echo "==> Removing systemd service..."

rm -f /usr/lib/systemd/system/acer-control-daemon.service
rm -f /etc/systemd/system/acer-control-daemon.service

systemctl daemon-reload
systemctl reset-failed 2>/dev/null || true

echo "==> Removing D-Bus policy..."

rm -f /usr/share/dbus-1/system.d/io.github.stanlysilas.AcerControl.conf

echo "==> Removing udev rules..."

rm -f /etc/udev/rules.d/99-acercontrol.rules
rm -f /usr/lib/modules-load.d/acer-control-center.conf
rm -f /etc/modules-load.d/acer-control-center.conf

udevadm control --reload-rules
udevadm trigger

echo "==> Removing executables..."

rm -f /usr/bin/acer-control-center
rm -f /usr/bin/acer-control-daemon

echo "==> Removing desktop entry..."

rm -f "/usr/share/applications/$APP_ID.desktop"

echo "==> Removing icons..."

rm -f "/usr/share/icons/hicolor/128x128/apps/$APP_ID.png"
rm -f "/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"
rm -f "/usr/share/icons/hicolor/scalable/apps/$APP_ID.svg"
rm -f /usr/share/icons/hicolor/256x256/apps/acer-control-center.png
rm -f /usr/share/icons/hicolor/128x128/apps/acer-control-center.png
rm -f /usr/share/icons/hicolor/scalable/apps/acer-control-center.svg
rm -f /usr/share/pixmaps/acer-control-center.png
rm -rf /usr/share/acer-control-center

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi

echo "==> Removing shared library..."

rm -f /usr/lib/libacercontrol.so*
rm -f /usr/lib64/libacercontrol.so*
rm -f /usr/local/lib/libacercontrol.so*
rm -f /usr/local/lib64/libacercontrol.so*
rm -f /etc/modules-load.d/acer-control-center.conf

ldconfig

echo "==> Removing DKMS module..."

echo "==> Removing loaded kernel module..."

modprobe -r acer_io acer_acpi acer_wmi acer_keyboard 2>/dev/null || true

echo "==> Removing DKMS module..."

if command -v dkms >/dev/null 2>&1; then
    while IFS= read -r dkms_entry; do
        case "$dkms_entry" in
            "$DKMS_NAME"/*)
                version="${dkms_entry#"$DKMS_NAME"/}"
                version="${version%%,*}"
                dkms remove "$DKMS_NAME/$version" --all || true
                ;;
        esac
    done < <(dkms status)
fi

rm -rf /usr/src/acer-keyboard-*

depmod -a

echo
echo "=========================================="
echo " Acer Control Center has been removed."
echo "=========================================="
echo
echo "A reboot is recommended."
