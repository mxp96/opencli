# OpenCLI

Command-line interface (CLI) tool untuk [open.mp](https://open.mp/) Manajemen Server dan Pawn project building dengan sistem package management.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Test](https://github.com/mxp96/open-cli/actions/workflows/test.yml/badge.svg)](https://github.com/mxp96/open-cli/actions/workflows/test.yml)

## Dokumentasi

- [Wiki Home](https://github.com/mxp96/open-cli/wiki)
- [Package Management](https://github.com/mxp96/open-cli/wiki)
- [Compiler Options](https://github.com/mxp96/open-cli/wiki/Compiler-Options)
- [Docker Guide](docs/DOCKER.md)
- [Contributing](docs/CONTRIBUTING.md)

## Fitur-Fitur

- **Package Management** - Installasi pustaka seperti sscanf, mysql dari GitHub
- **Compiler Management** - Compiler otomatis download dan pengelola penyimpanan sementara
- **Security First** - Verifikasi integritas dengan hash Argon2
- **Progress Tracking** - Real-time download dan build monitoring
- **Build Performance** - Lihat seberapa cepat proyek Kamu dikompilasi
- **Comprehensive Logging** - Log aktivitas lengkap untuk debugging

## Installasi

### Dari Release

Download binary terbaru untuk platform Anda dari [Releases](https://github.com/mxp96/open-cli/releases).

**Linux/macOS:**
```bash
tar -xzf opencli-*.tar.gz
sudo mv opencli /usr/local/bin/
opencli --version
```

**Windows:**
Ekstrak ZIP dan tambahkan ke PATH.

### Dari Source

```bash
git clone https://github.com/mxp96/open-cli
cd open-cli
cargo build --release
```

Binary akan berada di `target/release/opencli`.

### Menggunakan Docker

```bash
docker pull ghcr.io/mxp96/open-cli:latest
docker run --rm -v $(pwd):/workspace ghcr.io/mxp96/open-cli:latest --help
```

## Mulai Cepat

```bash
# Siapkan proyek baru
opencli setup

# Instal kompiler Pawn
opencli install compiler

# Instal paket
opencli package install Y-Less/sscanf

# Bangun proyek
opencli build

# Jalankan server
opencli run
```

## Package Management

### Instal Paket

```bash
# Instal semua paket dari opencli.toml
opencli package install

# Instal paket tertentu
opencli package install Y-Less/sscanf
opencli package install "Y-Less/sscanf=2.13.8"
opencli package install Y-Less/sscanf --target components

# Dengan batasan versi
opencli package install "Y-Less/sscanf=^2.13.7"
```

### Kelola Paket

```bash
# Daftar paket yang terinstal
opencli package list

# Hapus paket
opencli package remove Y-Less/sscanf

# Perbarui paket
opencli package update Y-Less/sscanf
opencli package update --all

# Periksa integritas
opencli package check
```

### Batasan Versi

```toml
[packages]
"owner/repo" = "^x.y.z"              # Pembaruan yang kompatibel
"owner/repo" = "~x.y.z"              # Hanya pembaruan patch
"owner/repo" = ">=x.y.z, <a.b.c"     # Batasan jangkauan
"owner/repo" = "latest"              # Selalu terbaru
"owner/repo" = "x.y.z"               # Versi persisnya
```

## Konfigurasi

Buat `opencli.toml` dengan `opencli setup`:

```toml
[build]
entry_file = "gamemodes/gamemode.pwn"
output_file = "gamemodes/gamemode.amx"
compiler_version = "v3.10.11"

[build.includes]
paths = ["include"]

[build.args]
args = ["-d3", "-;+", "-(+", "-\\+", "-Z+"]

[packages]
"Y-Less/sscanf" = { version = "^2.13.8", target = "components" }
```

## Membangun

```bash
# Bangunan bawaan
opencli build

# Output verbose
opencli build --verbose

# Paksa pengunduhan ulang kompiler
opencli build --force-download

# Perbarui konfigurasi kompiler
opencli build --update-config
```

## Pengembangan

```bash
# Format kode
cargo fmt --all
make docker-format  # Menggunakan Docker

# Jalankan linter
cargo clippy --all-targets --all-features

# Jalankan pengujian
cargo test --release

# Pengembangan Docker
docker compose up dev
```

Lihat [CONTRIBUTING.md](docs/CONTRIBUTING.md) untuk lebih jelasnya.

## Persyaratan

- Rust 1.89.0+ (untuk membangun dari source)
- Internet connection (awal pertama setup)
- open.mp server binary (untuk menjalankan server)

## Kontributor

Terima kasih kepada semua pihak yang telah berkontribusi sehingga proyek ini bisa terlaksana:

[![Contributors](https://contrib.rocks/image?repo=mxp96/open-cli)](https://github.com/mxp96/open-cli/graphs/contributors)

<!-- CONTRIBUTORS-LIST:START -->
Dibuat oleh [contrib.rocks](https://contrib.rocks).
<!-- CONTRIBUTORS-LIST:END -->

## Lisensi

Lihat [LICENSE](LICENSE) untuk selengkapnya.

> Terinspirasi oleh [sampctl](https://github.com/Southclaws/sampctl)
