# LinMacro ⚡ Auto Clicker

A modern, lightweight, lightning-fast Auto Clicker for Linux (**Wayland & X11**).

- **Clean & Focused:** Directly set your CPS (1 to 5,000) or use **🚀 Unlimited**.
- **Wayland & X11:** Dedicated virtual mouse device recognized natively by `libinput` and `mutter`.
- **Modes:** Toggle or Hold.
- **Emergency Killswitch:** `Pause/Break` key immediately aborts clicking.

---

## How to Run (Precompiled Binary - No FUSE Required)

Download `linmacro-linux-x86_64.tar.gz` from [Releases](https://github.com/Lowell137/linmacro/releases), extract it, and run:

```bash
tar -xzvf linmacro-linux-x86_64.tar.gz
./linmacro
```

---

## Prerequisites (for hotkeys & virtual input)

```bash
sudo modprobe uinput
sudo usermod -aG input $USER
```
*(Relogin required after adding your user to the input group).*

---

## Build from Source

```bash
cargo build --release
./target/release/linmacro
```
