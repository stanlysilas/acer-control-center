#!/usr/bin/env bash
set -u

# Debian and RPM run this hook during upgrades as well as removals. Preserve
# the live service and DKMS registration when replacing an older package.
case "${1:-}" in
    upgrade|failed-upgrade|1|2)
        exit 0
        ;;
esac

if command -v systemctl >/dev/null 2>&1; then
    systemctl disable --now acer-control-daemon.service >/dev/null 2>&1 || true
    systemctl daemon-reload || true
fi

if command -v dkms >/dev/null 2>&1; then
    dkms remove -m acer-keyboard -v 0.1.0 --all >/dev/null 2>&1 || true
fi

if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
fi

exit 0
