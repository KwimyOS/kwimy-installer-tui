<p align="center">
	<a href="https://github.com/KwimyOS"><img src="https://i.imgur.com/4PjBVpt.png" style="border-radius: 50%;" height="200" width="200" alt="Kwimy"></a>
</p>

<h4 align="center">TUI installer for <a href="https://github.com/KwimyOS">Kwimy Linux</a></h4>

![kwimy-installer screenshot](https://i.imgur.com/MhuQo6t.png)

| Name 1 | Name 2 | Name 3 |
| --- | --- | --- |
| ![Screenshot 1](https://i.imgur.com/i3bYHpt.png) | ![Screenshot 2](https://i.imgur.com/41opvUy.png) | ![Screenshot 3](https://i.imgur.com/a08KXY7.png) |

| Name 4 | Name 5 | Name 6 |
| --- | --- | --- |
| ![Screenshot 4](https://i.imgur.com/mTa5TRV.png) | ![Screenshot 5](https://i.imgur.com/4knGrF1.png) | ![Screenshot 6](https://i.imgur.com/Np8IIHy.png) |

### Build (local)

```sh
cargo build --manifest-path kwimy-installer-tui/Cargo.toml
```

### Run (local)

```sh
cargo run --manifest-path kwimy-installer-tui/Cargo.toml
```

See the Env Vars section below for local overrides

### Env Vars (local dev)

Copy `.env.example` to `.env` in the repo root and edit as needed. The installer loads it on startup

| Variable | Default | Purpose |
| --- | --- | --- |
| `KWIMY_SKIP_NETWORK` | `0` | Skip the network step when set to `1` |
| `KWIMY_OFFLINE_ONLY` | `0` | Force offline-only install when set to `1` |
| `KWIMY_DEV_GPU` | empty | Override GPU detection (comma-separated, e.g. `nvidia,intel,amd`) |
| `KWIMY_DEV_ALLOW_NONROOT` | `0` | Allow running the installer without root when set to `1` |
| `KWIMY_OUTER_GAP` | `24` | Adjusts terminal wrapper outer gap used by live scripts |
| `KWIMY_SKIP_OFFLINE_REPO` | `0` | Skip building the ISO offline repo when set to `1` |
| `KWIMY_PACMAN_MIRROR` | empty | Base URL for pacman mirrors (e.g. `https://mirror.kwimy.com/stable`) |
| `KWIMY_PACMAN_MIRRORLIST` | empty | Full mirrorlist contents, overrides `KWIMY_PACMAN_MIRROR` when set |

### Config

The installer reads `kwimy-installer-tui/config.toml` at build time (embedded into the binary).
Use it to manage:
- Base package lists (`[packages]`)
- App selection lists (`[selections]` for browsers, editors, terminals, compositors)

### Live Installer

- Select target disk
- Provide keyboard layout, timezone, hostname, user, and passwords, etc
- Installer configures LUKS + Btrfs + GRUB (UEFI/BIOS). Currently supports only Btrfs
- Installer runs inside Kitty terminal on Labwc (Wayland)
- Wallpaper: `kwimy-iso/airootfs/usr/share/backgrounds/kwimy/1.jpg`
- Boot splash theme: `kwimy-iso/airootfs/usr/share/plymouth/themes/kwimy-splash`
- GRUB theme: `kwimy-iso/grub/themes/kwimy-vimix-grub`
- Pacman mirrors: `kwimy-iso/airootfs/etc/pacman.d/mirrorlist`
- Offline repo: `kwimy-iso/airootfs/opt/kwimy-repo` is configured in `kwimy-iso/airootfs/etc/pacman.conf` and is preferred during install when present
- Offline repo key: place `kwimy-repo.gpg` at repo root or in `kwimy-iso/airootfs/opt/kwimy-repo` to bundle it into the ISO
- Offline-only mode: `KWIMY_OFFLINE_ONLY=1` forces install to use only `kwimy-offline` and fail if anything is missing

### Dev Run Notes

- `sudo -E KWIMY_SKIP_NETWORK=1 ./kwimy` bypasses the Network step and continues.
- `sudo KWIMY_DEV_GPU=nvidia,intel ./kwimy` overrides GPU detection for dev runs.

or use env variables
