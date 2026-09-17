# LinMacro ⚡ Auto Clicker

A modern, lightweight, lightning-fast Auto Clicker for Linux (**Wayland & X11**).

- **Clean & Focused:** Directly set your CPS (1 to 5,000) or use **🚀 Unlimited**.
- **Wayland & X11:** Dedicated virtual mouse device recognized natively by `libinput` and `mutter`.
- **Modes:** Toggle or Hold.
- **Emergency Killswitch:** `Pause/Break` key immediately aborts clicking.

---

## Prerequisites (Dependencies)

To run LinMacro or its AppImage across Linux distributions, make sure `uinput` kernel module is available and your user has permissions:

### 1. Arch Linux
```bash
sudo modprobe uinput
sudo usermod -aG input $USER
```
*(Relogin required after adding to the input group).*

### 2. Fedora
```bash
sudo modprobe uinput
sudo usermod -aG input $USER
```

### 3. Debian / Ubuntu / Mint
```bash
sudo modprobe uinput
sudo usermod -aG input $USER
```

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
