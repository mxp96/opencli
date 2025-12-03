# OpenCLI

Command-Line-Interface (CLI) Werkzeug für [open.mp](https://open.mp/) Serververwaltung und Pawn-Projektbau mit Paketverwaltungssystem.

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Dokumentation

- [Wiki Startseite](https://github.com/mxp96/opencli/wiki)
- [Paketverwaltung](https://github.com/mxp96/opencli/wiki)
- [Compiler-Optionen](../../docs/compiler-options.md)
- [Docker-Anleitung](../../docs/DOCKER.md)
- [Mitwirken](../../docs/CONTRIBUTING.md)

## Funktionen

- **Paketverwaltung** - Bibliotheken wie sscanf, mysql von GitHub installieren
- **Compiler-Verwaltung** - Automatischer Compiler-Download und Caching
- **Sicherheit zuerst** - Integritätsprüfung mit Argon2-Hash
- **Fortschrittsverfolgung** - Echtzeit-Download und Build-Überwachung
- **Build-Performance** - Sehen Sie, wie schnell Ihre Projekte kompiliert werden
- **Umfassende Protokollierung** - Vollständige Aktivitätsprotokolle zum Debuggen

## Installation

### Von Release

Laden Sie die neueste Binärdatei für Ihre Plattform von [Releases](https://github.com/mxp96/opencli/releases) herunter.

**Linux/macOS:**
```bash
tar -xzf opencli-*.tar.gz
sudo mv opencli /usr/local/bin/
opencli --version
```

**Windows:**
Extrahieren Sie die ZIP-Datei und fügen Sie sie zu PATH hinzu.

### Aus dem Quellcode

```bash
git clone https://github.com/mxp96/opencli
cd opencli
cargo build --release
```

Die Binärdatei befindet sich in `target/release/opencli`.

### Mit Docker

```bash
docker pull ghcr.io/mxp96/opencli:latest
docker run --rm -v $(pwd):/workspace ghcr.io/mxp96/opencli:latest --help
```

## Schnellstart

```bash
# Neues Projekt einrichten
opencli setup

# Pawn-Compiler installieren
opencli install compiler

# Pakete installieren
opencli package install Y-Less/sscanf

# Projekt bauen
opencli build

# Server starten
opencli run
```

## Paketverwaltung

### Pakete installieren

```bash
# Alle Pakete aus opencli.toml installieren
opencli package install

# Bestimmtes Paket installieren
opencli package install Y-Less/sscanf
opencli package install "Y-Less/sscanf=2.13.8"
opencli package install Y-Less/sscanf --target components

# Mit Versionseinschränkungen
opencli package install "Y-Less/sscanf=^2.13.7"
```

### Pakete verwalten

```bash
# Installierte Pakete auflisten
opencli package list

# Paket entfernen
opencli package remove Y-Less/sscanf

# Pakete aktualisieren
opencli package update Y-Less/sscanf
opencli package update --all

# Integrität prüfen
opencli package check
```

### Versionseinschränkungen

```toml
[packages]
"owner/repo" = "^x.y.z"              # Kompatible Updates
"owner/repo" = "~x.y.z"              # Nur Patch-Updates
"owner/repo" = ">=x.y.z, <a.b.c"     # Bereichseinschränkung
"owner/repo" = "latest"              # Immer neueste Version
"owner/repo" = "x.y.z"               # Exakte Version
```

## Konfiguration

Erstellen Sie `opencli.toml` mit `opencli setup`:

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

## Bauen

```bash
# Standard-Build
opencli build

# Ausführliche Ausgabe
opencli build --verbose

# Compiler-Neudownload erzwingen
opencli build --force-download

# Compiler-Konfiguration aktualisieren
opencli build --update-config
```

## Entwicklung

```bash
# Code formatieren
cargo fmt --all
make docker-format  # Mit Docker

# Linter ausführen
cargo clippy --all-targets --all-features

# Tests ausführen
cargo test --release

# Docker-Entwicklung
docker compose up dev
```

Siehe [CONTRIBUTING.md](docs/CONTRIBUTING.md) für weitere Details.

## Anforderungen

- Rust 1.89.0+ (zum Bauen aus dem Quellcode)
- Internetverbindung (erstmalige Einrichtung)
- open.mp Serverbinärdatei (zum Ausführen von Servern)

## Mitwirkende

Danke an alle Mitwirkenden, die dieses Projekt möglich gemacht haben:

[![Contributors](https://contrib.rocks/image?repo=mxp96/opencli)](https://github.com/mxp96/opencli/graphs/contributors)

<!-- CONTRIBUTORS-LIST:START -->
Erstellt mit [contrib.rocks](https://contrib.rocks).
<!-- CONTRIBUTORS-LIST:END -->

## Lizenz

Siehe [LICENSE](LICENSE) für Details.

> Inspiriert von [sampctl](https://github.com/Southclaws/sampctl)

