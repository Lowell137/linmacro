# LinMacro ⚡

A modern, high-performance, standalone Linux macro automation tool for **Wayland & X11**.

- **Distro-Independent:** No GTK/Qt runtime dependencies. Pure Rust + `egui` compiling into a single static binary.
- **Wayland & X11 Support:** Direct kernel-level input listening (`evdev`) and synthetic emission (`/dev/uinput`).
- **Live Recording:** Records keypresses and millisecond delays on the fly, transforming them into macros instantly.
- **Execution Modes:**
  - **Run Once:** Triggers macro once per keypress.
  - **Toggle Loop:** Starts looping on keypress, stops on the next keypress.
  - **Hold Loop:** Loops continuously while the trigger key is held down.
- **Emergency Killswitch:** Default `Pause/Break` key immediately aborts all active loops without system locks.

---

## How to Run

Run directly from the release binary or shell launcher:

```bash
cd ~/Projects/linmacro
./run.sh
```

Or run via cargo:
```bash
cargo run --release
```

---

## Permissions & Setup

LinMacro works without root privileges:
1. **/dev/uinput (Virtual Input):** User sessions on modern Linux systems (`systemd-logind`) have automatic `uaccess` ACL permissions.
2. **/dev/input/event* (Hardware Keystroke Detection):** Active keyboard and mouse devices on seat0 have session user access.
