# Acer Control Center

Modern Linux control center for supported Acer gaming laptops.

## Features

- Native Rust interface built with egui/eframe (X11 and Wayland)
- Dashboard
- Keyboard RGB control
- Live keyboard preview
- System D-Bus daemon
- DKMS kernel driver
- Live fan monitoring
- System and kernel-driver status
- Manual GitHub release checks from Settings
- Automatic installation

## Supported Hardware

Currently tested on

- Acer ALG AL15G-53

Additional Clevo-based Acer laptops may work but are currently untested. The
current hardware backend supports the Acer ALG AL15G-53 only.

The graphical application does not require GTK or Libadwaita. It uses the
system's X11 or Wayland session and communicates with the existing system
daemon over D-Bus. Linux graphics libraries (including X11/Wayland and
xkbcommon) are still supplied by the distribution. The source installer
supports Debian/Ubuntu, Arch, Fedora/RHEL, and openSUSE package managers. It
builds the application from Rust sources and requires Rust 1.88 or newer; when
the distribution Rust package is older, the installer sets up the stable
toolchain under `/opt/acer-control-center`.

Prebuilt packages are produced for Debian (`.deb`), RPM (`.rpm`), and
Arch (`.pkg.tar.zst`). Packages include the application, system daemon, DKMS
driver sources, desktop entry, and service integration. Kernel headers matching
the running kernel are needed to build the DKMS driver. If the package hook
reports missing headers, install them and run `sudo dkms autoinstall`.

These packages cover the listed packaging families, not every Linux
distribution. Other distributions need a compatible package manager and
dependencies or a distribution-specific recipe.

## Installation

Clone the repository

```bash
git clone https://github.com/stanlysilas/acer-control-center.git
cd acer-control-center
sudo ./install.sh
```

or

```bash
wget https://raw.githubusercontent.com/stanlysilas/acer-control-center/master/install.sh
chmod +x install.sh
sudo ./install.sh
```

or

**Download the package for your distribution from the [Releases](https://github.com/stanlysilas/acer-control-center/releases/latest) page. (Recommended)**

After installation, run

```bash
sudo usermod -aG acercontrol yourusername
```

And reboot the laptop when convenient.

The application version and device status are available in **Settings**. Use
**Check for updates** to query the latest published GitHub release; the app
provides a release link and leaves downloading and installation to you.

Keyboard color, brightness, breathing animation, and the selected fan mode are
restored by the system daemon after it starts. The keyboard inactivity timeout
is saved per user. When enabled, a small background process starts in the
system tray at desktop login; choosing **Off** removes that login startup
entry. The **Minimize to system tray when closing the window** option is
available in Settings. When enabled, closing the window exits the UI process
while leaving the tray and inactivity controller running. The tray menu can
reopen the window or quit the background process. This separate process model
also supports Wayland, where native windows cannot be hidden after mapping. If
the desktop session has no StatusNotifier tray host, the app opens visibly so
it stays accessible.

If Secure Boot is enabled, or if you plan to enable it later, complete the MOK enrollment described in the Additional Setup section.

The installer does not reboot the system automatically.

## Additional Setup

### Secure Boot and DKMS Module Signing

Acer Control Center uses DKMS to build its kernel modules. The modules are signed with a local DKMS Machine Owner Key (MOK) so they can be loaded when UEFI Secure Boot is enabled.

The source installer creates the DKMS signing key when necessary. Package
installs also create it when Secure Boot is enabled. MOK enrollment is
intentionally left to the user.

The installer does not change Secure Boot settings and does not reboot the system automatically.

If Secure Boot is enabled, or if you plan to enable it later, enroll the DKMS certificate:

```bash
sudo mokutil --import /var/lib/dkms/mok.pub
```

You will be asked to create a temporary enrollment password.

Reboot when convenient and complete the enrollment through the UEFI MOK Manager:

1. Select `Enroll MOK`.
2. Select `Continue`.
3. Select `Yes`.
4. Enter the password created with `mokutil --import`.
5. Reboot.

After the system starts, verify that the certificate is enrolled:

```bash
sudo mokutil --test-key /var/lib/dkms/mok.pub
```

You can also verify that the Acer module is signed:

```bash
modinfo acer_keyboard | grep -E '^(signer|sig_key|sig_hashalgo):'
```

A correctly signed module should show:

```text
signer:         DKMS module signing key
sig_key:        <your enrolled MOK fingerprint>
sig_hashalgo:   sha512
```

If Secure Boot is enabled and the MOK has not been enrolled, the kernel will reject the Acer modules with an error such as:

```text
Key was rejected by service
```

No reinstallation is required after enrolling the MOK. The existing signed modules can be loaded normally.

### Kernel Taint and Device Security

Because Acer Control Center's DKMS modules are built outside the upstream Linux kernel tree, Linux marks the running kernel as tainted when an Acer module is loaded.

Therefore, even when:

- Secure Boot is enabled
- the Acer modules are cryptographically signed
- the MOK is enrolled
- the modules load successfully

Fedora's device security report may still show:

```text
Linux Kernel Verification: ! Fail (Tainted)
```

This is expected for an out-of-tree DKMS driver.

The kernel's taint value distinguishes an out-of-tree module from an unsigned module. An out-of-tree module sets the `4096` taint bit, while an unsigned module sets the `8192` taint bit.

You can inspect the current taint state with:

```bash
cat /proc/sys/kernel/tainted
```

You can verify Secure Boot with:

```bash
mokutil --sb-state
```

And verify the Acer module signature with:

```bash
modinfo acer_keyboard | grep -E '^(signer|sig_key|sig_hashalgo):'
```

A kernel taint caused by a correctly signed out-of-tree module does not mean that Secure Boot rejected the module or that the module is unsigned.

Removing this particular taint would require the driver to become an upstream/in-tree Linux kernel driver. The taint should not be manually cleared because it is a diagnostic state maintained by the kernel.

## Uninstall

```bash
sudo ./uninstall.sh
```

## Project Structure

```text
driver/
daemon/
gui/
lib/
packaging/
```

## Current Status

### Implemented

- Dashboard
- Keyboard RGB
- D-Bus daemon
- Driver communication
- Live fan monitoring

### Planned

- Performance profiles
- Battery management
- Manual fan control
- Firmware updates

## License

GPL-3.0-or-later

## Discord Community

The **[AcerCtl](https://discord.gg/XjnjN25QED)** Discord server is used for community discussion, support, bug reports, feature requests, and project announcements.