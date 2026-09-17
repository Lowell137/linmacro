# LinMacro ⚡ Auto Clicker

A modern, lightweight, lightning-fast Auto Clicker for Linux (**Wayland & X11**).

- **Clean & Focused:** Directly set your CPS (1 to 5,000) or use **🚀 Unlimited**.
- **Wayland & X11:** Dedicated virtual mouse device recognized natively by `libinput` and `mutter`.
- **Modes:** Toggle or Hold.
- **Emergency Killswitch:** `Pause/Break` key immediately aborts clicking.

---

## Prerequisites & Dependencies

Before running LinMacro from source, install the required build dependencies for your distribution:

### Arch Linux
```bash
sudo pacman -S base-devel pkgconf fontconfig libx11 libxi wayland
sudo modprobe uinput
sudo usermod -aG input $USER
```

### Fedora
```bash
sudo dnf install @development-tools pkgconfig fontconfig-devel libX11-devel libXi-devel wayland-devel
sudo modprobe uinput
sudo usermod -aG input $USER
```

### Debian / Ubuntu / Mint
```bash
sudo apt update && sudo apt install build-essential pkg-config libfontconfig1-dev libx11-dev libxi-dev libwayland-dev
sudo modprobe uinput
sudo usermod -aG input $USER
```
*(Relogin required after adding your user to the input group).*

---

## How to Run

Download the `LinMacro-x86_64.AppImage` from [Releases](https://github.com/Lowell137/linmacro/releases), make it executable, and run:

```bash
chmod +x LinMacro-x86_64.AppImage
./LinMacro-x86_64.AppImage
```

Or run from source:
```bash
cargo run --release
```
