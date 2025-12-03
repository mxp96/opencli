# OpenCLI

Интерфейс командной строки (CLI) для управления сервером [open.mp](https://open.mp/) и сборки Pawn-проектов с системой управления пакетами.

[![Лицензия: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Тесты](https://github.com/mxp96/open-cli/actions/workflows/test.yml/badge.svg)](https://github.com/mxp96/open-cli/actions/workflows/test.yml)

## Документация

- [Главная Wiki](https://github.com/mxp96/open-cli/wiki)
- [Управление пакетами](https://github.com/mxp96/open-cli/wiki)
- [Опции компилятора](https://github.com/mxp96/open-cli/wiki/Compiler-Options)
- [Руководство по Docker](docs/DOCKER.md)
- [Участие в разработке](docs/CONTRIBUTING.md)

## Возможности

- **Управление пакетами** — Установка библиотек, таких как sscanf, mysql, из GitHub
- **Управление компилятором** — Автоматическая загрузка компилятора и управление временным хранилищем
- **Безопасность прежде всего** — Проверка целостности с помощью хэша Argon2
- **Отслеживание прогресса** — Мониторинг загрузки и сборки в реальном времени
- **Производительность сборки** — Посмотрите, как быстро компилируется ваш проект
- **Полное логирование** — Подробные логи активности для отладки

## Установка

### Из релизов

Загрузите последнюю версию бинарного файла для вашей платформы из раздела [Релизы](https://github.com/mxp96/open-cli/releases).

**Linux/macOS:**
```bash
tar -xzf opencli-*.tar.gz
sudo mv opencli /usr/local/bin/
opencli --version
```

**Windows:**
Разархивируйте ZIP и добавьте путь в переменную PATH.

### Из исходного кода

```bash
git clone https://github.com/mxp96/open-cli
cd open-cli
cargo build --release
```

Бинарный файл будет находиться в `target/release/opencli`.

### Использование Docker

```bash
docker pull ghcr.io/mxp96/open-cli:latest
docker run --rm -v $(pwd):/workspace ghcr.io/mxp96/open-cli:latest --help
```

## Быстрый старт

```bash
# Создайте новый проект
opencli setup

# Установите компилятор Pawn
opencli install compiler

# Установите пакет
opencli package install Y-Less/sscanf

# Соберите проект
opencli build

# Запустите сервер
opencli run
```

## Управление пакетами

### Установка пакетов

```bash
# Установите все пакеты из opencli.toml
opencli package install

# Установите конкретный пакет
opencli package install Y-Less/sscanf
opencli package install "Y-Less/sscanf=2.13.8"
opencli package install Y-Less/sscanf --target components

# С ограничением версии
opencli package install "Y-Less/sscanf=^2.13.7"
```

### Управление пакетами

```bash
# Список установленных пакетов
opencli package list

# Удаление пакета
opencli package remove Y-Less/sscanf

# Обновление пакета
opencli package update Y-Less/sscanf
opencli package update --all

# Проверка целостности
opencli package check
```

### Ограничения версий

```toml
[packages]
"owner/repo" = "^x.y.z"              # Совместимые обновления
"owner/repo" = "~x.y.z"              # Только обновления патчей
"owner/repo" = ">=x.y.z, <a.b.c"     # Диапазонное ограничение
"owner/repo" = "latest"              # Всегда последняя версия
"owner/repo" = "x.y.z"               # Точная версия
```

## Конфигурация

Создайте `opencli.toml` с помощью `opencli setup`:

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

## Сборка

```bash
# Обычная сборка
opencli build

# Подробный вывод
opencli build --verbose

# Принудительная перезагрузка компилятора
opencli build --force-download

# Обновление конфигурации компилятора
opencli build --update-config
```

## Разработка

```bash
# Форматирование кода
cargo fmt --all
make docker-format  # С использованием Docker

# Запуск линтера
cargo clippy --all-targets --all-features

# Запуск тестов
cargo test --release

# Разработка в Docker
docker compose up dev
```

Смотрите [CONTRIBUTING.md](docs/CONTRIBUTING.md) для подробностей.

## Требования

- Rust 1.89.0+ (для сборки из исходного кода)
- Интернет-соединение (при первоначальной настройке)
- Бинарный файл сервера open.mp (для запуска сервера)

## Участники

Благодарим всех, кто внес свой вклад в реализацию этого проекта:

[![Участники](https://contrib.rocks/image?repo=mxp96/open-cli)](https://github.com/mxp96/open-cli/graphs/contributors)

<!-- CONTRIBUTORS-LIST:START -->
Создано с помощью [contrib.rocks](https://contrib.rocks).
<!-- CONTRIBUTORS-LIST:END -->

## Лицензия

Подробнее см. в файле [LICENSE](LICENSE).

> Вдохновлено [sampctl](https://github.com/Southclaws/sampctl)
