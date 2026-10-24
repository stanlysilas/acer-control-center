#!/usr/bin/env bash
set -u

PACKAGE_NAME="acer-keyboard"
PACKAGE_VERSION="0.1.0"
KERNEL_VERSION="$(uname -r)"

if ! getent group acercontrol >/dev/null 2>&1; then
    if command -v groupadd >/dev/null 2>&1; then
        groupadd --system acercontrol || true
    fi
fi

if command -v dkms >/dev/null 2>&1; then
    if command -v mokutil >/dev/null 2>&1 && mokutil --sb-state 2>&1 | grep -qi 'SecureBoot enabled'; then
        if [[ ! -s /var/lib/dkms/mok.key || ! -s /var/lib/dkms/mok.pub ]]; then
            dkms generate_mok || echo "WARNING: DKMS could not create a signing key; Secure Boot may reject the module."
        fi
    fi

    if ! dkms status -m "$PACKAGE_NAME" -v "$PACKAGE_VERSION" 2>/dev/null | grep -q .; then
        dkms add -m "$PACKAGE_NAME" -v "$PACKAGE_VERSION" || true
    fi

    if [[ -e "/lib/modules/$KERNEL_VERSION/build" ]]; then
        dkms install -m "$PACKAGE_NAME" -v "$PACKAGE_VERSION" -k "$KERNEL_VERSION" || \
            echo "WARNING: DKMS could not build for kernel $KERNEL_VERSION; install matching kernel headers and run: sudo dkms autoinstall -k $KERNEL_VERSION"
        depmod -a "$KERNEL_VERSION" || true
    else
        echo "WARNING: matching kernel headers are missing; install them and run: sudo dkms autoinstall -k $KERNEL_VERSION"
    fi
else
    echo "WARNING: DKMS is missing; install DKMS and matching kernel headers to enable hardware control."
fi

if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
    udevadm trigger || true
fi

for module in acer_io acer_keyboard acer_wmi acer_acpi; do
    modprobe "$module" || true
done

if command -v busctl >/dev/null 2>&1; then
    busctl call --system org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig >/dev/null 2>&1 || true
fi

if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
    systemctl enable --now acer-control-daemon.service || \
        echo "WARNING: the daemon could not start. Check module status and journalctl -u acer-control-daemon.service."
fi

if command -v mokutil >/dev/null 2>&1 && mokutil --sb-state 2>&1 | grep -qi 'SecureBoot enabled'; then
    echo "Secure Boot is enabled. Enroll the DKMS signing certificate if the module is rejected; see the README."
fi

exit 0
