# Aether Launcher

## Contents
- [English](#english)
- [Русский](#русский)

## English

Aether Launcher is a convenient launcher for managing your Minecraft installations.

<!-- TODO: add screenshots here (to be provided by the owner) -->

### Features
- Support for multiple languages (including English and Russian)
- User-friendly interface
- Easy setup and usage
- Plugin and extension support
- Currently, only offline accounts are supported for evaluation purposes

### Planned Features
- Integration with various services
- Support for licensed accounts

### Installation
To download the installer, visit the [releases page](https://github.com/MangriMen/Aether/releases).

Supported OS: Windows, Linux (macOS is not supported yet).

The Windows installer is not code-signed, so Windows SmartScreen will show a warning.

### Disclaimer
- **Aether** is intended for **educational and experimental purposes only**.
- We **do not condone piracy** — users are encouraged to purchase a legitimate Minecraft license.
- Respect all relevant licensing agreements and support Minecraft developers.
- **NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.**

### License
Aether is licensed under the [GNU General Public License v3.0 only](LICENSE). The plugin API crates in `packages/core` (`aether-core-plugin-api`, `register-schema`, `register-schema-derive`) are dual-licensed under [MIT](packages/core/LICENSE-MIT) or [Apache-2.0](packages/core/LICENSE-APACHE), so plugins that depend on them are not subject to the GPL because of that dependency. Licenses of bundled third-party dependencies are shipped with the application (`THIRD_PARTY_LICENSES.md`).

### Architecture
See [ARCHITECTURE.md](ARCHITECTURE.md) for an overview of the project structure.

### Building from source

**Requirements:** Rust 1.95+ (edition 2024), Node.js >= 24, Bun 1.3.11.

```sh
bun install
bun run desktop:dev
```

- The build does not need a database: the SQL query metadata is committed in the `.sqlx` directories, so `sqlx` builds offline.
- `bun run db:setup` is only needed when you change SQL queries or migrations. It requires `sqlx-cli` (`cargo install sqlx-cli`) and a `.env` file: copy `.env.example` to `.env`.

---

## Русский

Aether Launcher - это удобный лаунчер для управления вашими установками Minecraft.

<!-- TODO: добавить скриншоты (предоставит владелец) -->

### Особенности
- Поддержка нескольких языков (включая английский и русский)
- Удобный интерфейс
- Легкая настройка и использование
- Поддержка плагинов и расширений
- В текущий момент поддерживаются только offline аккаунты для ознакомления

### Планируется в будущем
- Интеграция с различными сервисами
- Поддержка лицензионных аккаунтов

### Установка
Для загрузки установщика перейдите на [страницу релизов](https://github.com/MangriMen/Aether/releases).

Поддерживаемые ОС: Windows, Linux (macOS пока не поддерживается).

Установщик для Windows не подписан, поэтому Windows SmartScreen покажет предупреждение.

### Отказ от ответственности
- **Aether** предназначен **только для образовательных и экспериментальных целей**.
- Мы **не одобряем пиратство** — пользователям рекомендуется приобретать легальную лицензию Minecraft.
- Соблюдайте все соответствующие лицензионные соглашения и поддерживайте разработчиков Minecraft.
- **NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.** (Не является официальным продуктом Minecraft, не одобрен Mojang или Microsoft и не связан с ними.)

### Лицензия
Aether распространяется под лицензией [GNU General Public License v3.0 only](LICENSE). Крейты API плагинов в `packages/core` (`aether-core-plugin-api`, `register-schema`, `register-schema-derive`) имеют двойную лицензию [MIT](packages/core/LICENSE-MIT) или [Apache-2.0](packages/core/LICENSE-APACHE), поэтому плагины, которые от них зависят, не подпадают под GPL из-за этой зависимости. Лицензии сторонних зависимостей поставляются вместе с приложением (`THIRD_PARTY_LICENSES.md`).

### Архитектура
Обзор структуры проекта см. в [ARCHITECTURE.md](ARCHITECTURE.md).

### Сборка из исходников

**Требования:** Rust 1.95+ (edition 2024), Node.js >= 24, Bun 1.3.11.

```sh
bun install
bun run desktop:dev
```

- Для сборки база данных не нужна: метаданные SQL-запросов закоммичены в каталогах `.sqlx`, поэтому `sqlx` собирается офлайн.
- `bun run db:setup` нужен только при изменении SQL-запросов или миграций. Требуется `sqlx-cli` (`cargo install sqlx-cli`) и файл `.env`: скопируйте `.env.example` в `.env`.
