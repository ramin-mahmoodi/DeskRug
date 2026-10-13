# DeskRug

An interactive, physics-based Persian carpet lying peacefully on your Windows desktop.

Built with **Tauri v2**, **Rust**, and **WebGL / Three.js**.

---

## ✨ Features

- **Realistic Cloth Physics**: Real-time Verlet integration with structural, shear, and bending constraints, plus custom self-collision preventing mesh self-penetration.
- **Interactive Drag & Fold**: Grab and toss any corner or edge with your mouse. The rug folds, settles, and rests naturally on your desktop floor.
- **Classic Persian Carpet Styles**:
  - Tabriz (تبریز)
  - Nain (نائین)
  - Heriz (هریس)
  - Kilim (گلیم)
  - Gabbeh (گبه)
  - Turkmen (ترکمن)
  - Bakhtiari (بختیاری)
  - And modern classic variants (Navy, Emerald, Great Wave, Zen).
- **Customizable Sizes & Weights**:
  - Sizes: Small, Medium, Large
  - Weights: Light, Normal, Heavy
- **Ultra Lightweight & Efficient**:
  - Standalone binary size: **~9.2 MB**
  - Memory footprint: **~48 MB RAM** (75% lower than Electron equivalents)
  - Zero CPU usage when idle; physics and rendering sleep automatically once settled.
- **Native Windows Desktop Integration**:
  - Transparent borderless overlay.
  - Smart click-through detection: clicks on empty desktop pass right through to your desktop icons; hovering or clicking on the rug interacts with the cloth.
  - Native Windows System Tray menu with full controls in English.

---

## 🚀 Building from Source

### Prerequisites
1. **Rust**: [Install Rust](https://www.rust-lang.org/tools/install) (latest stable).
2. **Node.js**: [Install Node.js](https://nodejs.org/) (v18+ recommended).
3. **Windows 10 / 11** with WebView2 runtime (pre-installed on modern Windows).

### Build Release Binary
```bash
# Navigate to the Rust Tauri directory
cd src-tauri

# Build optimized release binary
cargo build --release
```

The compiled standalone executable will be located at:
```
src-tauri/target/release/deskrug.exe
```

---

## 🎮 Controls

- **Left Mouse Drag**: Grab and lift any part of the carpet.
- **Right Click**: Open contextual controls (Flatten, Center, Pattern, Size, Weight, Exit).
- **System Tray**: Right-click the DeskRug tray icon near your Windows clock for quick commands.

---

## 📜 License

MIT License. See [LICENSE](LICENSE) for details.
