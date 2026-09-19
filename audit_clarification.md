# Верификация аудита архитектуры плагинов

> Документ дополняет `plugin-architecture-audit.md`. Содержит результаты независимой проверки
> утверждений аудита по трём источникам: репозиторий эталонного плагина `packwiz-plugin`,
> исходники Extism тега `v1.30.0`, и актуальный код `aether-core`.
>
> **Дата верификации:** 2026-09-20
> **Статус:** верификация завершена. План работ строится отдельно, на основе этого документа.

---

## Содержание

1. [Репозиторий эталонного плагина](#1-репозиторий-эталонного-плагина-packwiz-plugin)
2. [Исходники Extism v1.30.0](#2-исходники-extism-v1300)
3. [Собственный код aether-core](#3-собственный-код-aether-core)
4. [Текущие версии зависимостей](#4-текущие-версии-зависимостей)
5. [Полный список провайдеров контента](#5-полный-список-провайдеров-контента)
6. [Сводка верификации](#6-сводка-верификации)
7. [Что должно измениться в плане миграции](#7-что-должно-измениться-в-плане-миграции)

---

## 1. Репозиторий эталонного плагина `packwiz-plugin`

### 1.1. Состояние ветвей

```
* bbc96fb (origin/feat/optimize, feat/optimize)  feat: update installation process for new api
* 8515a09 (HEAD -> ref/refactor-content-installing, ref/optimize-deps)  feat: optimize dependencies
* 9bbc4aa (origin/main, origin/HEAD, main)  chore: improve cache and add capabilities to ci
```

- Текущая активная ветка: **`ref/refactor-content-installing`** (HEAD).
- `main` — на коммите `9bbc4aa`.
- `feat/optimize` — **потомок** `ref/refactor-content-installing` (проверено `git merge-base --is-ancestor`, exit=0). Это не параллельная ветка, а продолжение.
- Анализ проводился по `main` через `git show`/`git diff` **без чекаута**.

### 1.2. Diff `main` → `ref/refactor-content-installing`

**21 файл, +69/−57. Все изменения — не архитектурные:**

| Файл                                                                                                   | Что изменилось                                                                                                                  | Относится к архитектуре?         |
| ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| `Cargo.toml` (workspace)                                                                               | добавлен `[profile.release]` (opt-level="z", lto, panic="abort")                                                                | Нет                              |
| `packwiz/Cargo.toml`                                                                                   | `edition 2021→2024`; `extism-pdk 1.3.0→1.4`, `extism-convert 1.10.0→1.30`; удалены `lazy_static`, `phf`, `serr`; `toml 0.8→1.1` | **Частично** — только версии PDK |
| `core/domain/error.rs`                                                                                 | удалён `impl From<serr::SerializedError>`                                                                                       | Нет                              |
| `packwiz/infra/constants.rs`                                                                           | `phf::Map` → `&[(&str, &str)]` + `get_redistributable_link()`                                                                   | Нет                              |
| `packwiz/infra/redistributable.rs`                                                                     | `.into_iter()` → `.iter()`, `.get(key)` → `get_redistributable_link(key)`                                                       | Нет                              |
| `instance/app/update.rs`, `import.rs`, `create_instance_from_pack.rs`, `get_command_to_update_pack.rs` | перестановка импортов (rustfmt edition 2024)                                                                                    | Нет                              |
| `*/infra/extism/adapter.rs`, `*/infra/host_functions.rs`                                               | перестановка импортов                                                                                                           | Нет                              |
| `xtask/**`                                                                                             | перестановка импортов                                                                                                           | Нет                              |

**Вывод: `ref/refactor-content-installing` — это чисто технический рефакторинг** (edition bump + удаление зависимостей + rustfmt). Ни одна из архитектурных тем аудита (доступ к ФС, сеть, `install_atomic`/`install_modpack`, `run_command`) в нём не затронута.

### 1.3. Diff `main` → `feat/optimize` (30 файлов, +216/−199) — **архитектурный рефакторинг**

**a) Новая capability `packManagers` вместо `importers` + `updaters`:**

```diff
-  "importers": [ { "id": "packwiz", ..., "handler": "import" } ],
-  "updaters":  [ { "id": "packwiz", ..., "handler": "update" } ]
+  "packManagers": [ {
+      "id": "packwiz",
+      "supportsInstall": true,
+      "supportsUpdate": true,
+      "supportsCheckUpdates": false,
+      "handlers": {
+        "install": "install",
+        "update": "update",
+        "checkUpdates": null,
+        "resolveMetadata": "resolve_pack_metadata"
+      }
+  } ]
```

**b) Разделение ответственности: ядро создаёт инстанс, плагин только устанавливает:**

```diff
-// create_instance_from_pack.rs (удалён)
-let instance_id = instance_create(new_instance)...;
-crate::api::settings::save_to_instance(&instance_id, &pack_settings)?;
+// install.rs (новый)
+/// Install a pack into an already-created instance.
+/// This function does NOT create the instance — the core is responsible for that.
+pub fn install(params: PackInstallParamsDto) -> crate::Result<()> {
+    let instance_id = params.instance_id;
+    let pack_path = match &params.pack_source {
+        PackSourceDto::LocalFile(path) | PackSourceDto::RemoteUrl(path) => path.clone(),
+    };
+    crate::api::settings::save_to_instance(&instance_id, &pack_settings)?;
+    packwiz::preload_resources(&instance_id)?;
+    update_pack_base(&instance_id, &pack_settings)?;
+}
```

**c) Новый handler `resolve_pack_metadata` — резолвинг метаданных без создания инстанса:**

```rust
pub fn resolve_pack_metadata(source: PackSourceDto) -> crate::Result<PackMetadataDto> {
    let pack = packwiz::get_pack_from_path_or_url(&source_str)?;
    let (mod_loader, loader_version) = extract_mod_loader(&pack.versions)?;
    Ok(PackMetadataDto { name, game_version, mod_loader, loader_version, pack_info })
}
```

**d) Новые DTO в `aether-core-plugin-api`:** `PackSourceDto`, `PackMetadataDto`, `PackInstallParamsDto`.

**e) `run_command` остаётся** — `update_pack_base` по-прежнему вызывает `host::run_command(command)`.

**f) Скачивание и запись на диск — без изменений:** `extism_pdk::http::request` + `std::fs::write` / `std::fs::copy`.

### 1.4. Относятся ли отличия к архитектурным вопросам аудита?

| Тема аудита                                     | `ref/refactor-content-installing` | `feat/optimize`                                                                                                             |
| ----------------------------------------------- | --------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Доступ к ФС (проблема 1)                        | Не затронуто                      | **Не затронуто** — `std::fs` остаётся                                                                                       |
| Сеть (проблема 2)                               | Не затронуто                      | **Не затронуто** — `extism_pdk::http` остаётся                                                                              |
| `install_atomic`/`install_modpack` (проблема 3) | Не затронуто                      | **Затронуто, но в другом направлении** — вводится `packManagers` с `install`/`update`/`resolveMetadata`, а не `InstallPlan` |
| `run_command` (A1)                              | Не затронуто                      | **Не затронуто** — остаётся без allowlist                                                                                   |
| Переиспользование провайдеров (проблема 4)      | Не затронуто                      | **Не затронуто**                                                                                                            |

**Важнейшая находка:** `feat/optimize` — это **незавершённый рефакторинг в направлении, отличном от того, что предлагает аудит**. Он:

- **совпадает** с аудитом в одном: ядро создаёт инстанс, плагин не вызывает `instance_create` (шаг в сторону «Plugin as Resolver, Core as Executor»);
- **расходится** с аудитом в главном: вместо `InstallPlan` (декларативный план) вводится `packManagers` с императивным `install`, который всё ещё запускает jar через `run_command` и пишет на диск через `std::fs`.

**Критично: `feat/optimize` не собирается против текущего `aether`.** Он ссылается на `PackSourceDto`, `PackMetadataDto`, `PackInstallParamsDto` из `aether-core-plugin-api`, которых в репозитории **нет** (проверено grep — найдено только `ModpackInstallParamsDto`). Также `capabilities.json` использует `packManagers`, которого нет в `PluginCapabilitiesDto` (там `importers`/`updaters`/`contentProviders`).

### 1.5. Ответы по `main`-ветке эталонного плагина

**Использует ли `extism-pdk` и какой версии:**

```toml
# packwiz/Cargo.toml (main)
extism-convert = "1.10.0"
extism-pdk = "1.3.0"
```

**Совместимость с хостом `extism` 1.30.0:** PDK 1.3.0 **совместим** с runtime 1.30.0 (PDK 1.4.1 — последняя; 1.3.0 старше, но не требует функций, которых нет в 1.30.0). В ветке рефакторинга PDK поднят до 1.4 — тоже совместимо.

**Как плагин скачивает файлы и пишет на диск:**

```rust
// packwiz/infra/redistributable.rs
let res = extism_pdk::http::request::<()>(&HttpRequest { url, headers, method: Some("GET") }, None)?;
std::fs::write(path, res.body())?;   // ← WASI

// packwiz/app/get_pack_from_path_or_url.rs
let data: Vec<u8> = if path_or_url.contains("://") {
    http::request::<()>(&HttpRequest::new(path_or_url).with_method("GET"), None)?.body().to_vec()
} else {
    std::fs::read(&path)?   // ← WASI
};
```

**Вывод:** HTTP — через **встроенный `extism:host/env::http_request`** (не через WASI-сокеты), запись/чтение — через **WASI `std::fs`**. Это **подтверждает** вывод аудита о том, что WASI-сокеты не используются, и что `allowed_hosts` — единственный сетевой контроль.

**Какие host-функции реально импортирует:**

```rust
// host/infra/host_functions.rs
pub fn log(level: u32, msg: String);
pub fn get_id() -> String;
pub fn run_command(command: Msgpack<CommandDto>) -> HostResult<OutputDto>;

// instance/infra/host_functions.rs
pub fn instance_get_dir(instance_id: String) -> HostResult<String>;
pub fn instance_plugin_get_dir(instance_id: String) -> HostResult<String>;
pub fn instance_create(new_instance: Msgpack<NewInstanceDto>) -> HostResult<String>;
pub fn list_content(instance_id: String) -> HostResult<HashMap<String, ContentFileDto>>;
pub fn enable_contents(instance_id: String, content_paths: Msgpack<Vec<String>>) -> HostResult<()>;
pub fn disable_contents(instance_id: String, content_paths: Msgpack<Vec<String>>) -> HostResult<()>;

// java/infra/host_functions.rs
pub fn get_java(version: u32) -> HostResult<JavaDto>;
pub fn install_java(version: u32) -> HostResult<JavaDto>;
```

**Расхождения с вашим списком:**

| Ваш список                | Фактически у packwiz           | Расхождение         |
| ------------------------- | ------------------------------ | ------------------- |
| `log`                     | ✅                             | —                   |
| `run_command`             | ✅                             | —                   |
| `instance_get_dir`        | ✅                             | —                   |
| `instance_create`         | ✅                             | —                   |
| `list_content`            | ✅                             | —                   |
| `enable/disable_contents` | ✅                             | —                   |
| —                         | **`get_id`**                   | **Вы не упомянули** |
| —                         | **`instance_plugin_get_dir`**  | **Вы не упомянули** |
| —                         | **`get_java`, `install_java`** | **Вы не упомянули** |

**Критично: `capabilities.json` на `main` НЕ содержит `contentProviders`.** Только `importers` (handler `import`) и `updaters` (handler `update`). То есть **packwiz на `main` — не content provider**, а importer + updater. Это означает, что проблема №3 аудита («два несовместимых контракта `install_atomic`/`install_modpack`») **не относится к packwiz** — он вообще не проходит через `ContentProvider`.

---

## 2. Исходники Extism v1.30.0

### a. `allowed_hosts` проверяется только внутри `http_request` — **ПОДТВЕРЖДЕНО**

`runtime/src/pdk.rs`, `fn http_request`:

```rust
let allowed_hosts = &data.manifest.allowed_hosts;
let host_str = url.host_str().unwrap_or_default();
let host_matches = if let Some(allowed_hosts) = allowed_hosts {
    allowed_hosts.iter().any(|url| {
        let pat = match glob::Pattern::new(url) { Ok(x) => x, Err(_) => return url == host_str };
        pat.matches(host_str)
    })
} else { false };

if !host_matches {
    return Err(Error::msg(format!("HTTP request to {} is not allowed", req.url)));
}
```

`allowed_hosts` читается **только** в `pdk.rs::http_request`. В `current_plugin.rs` (создание WASI-контекста) `allowed_hosts` не упоминается вообще. **Формулировка точна.**

### b. WASI p1 sockets не подключены — **ПОДТВЕРЖДЕНО, но формулировка требует уточнения**

**Что подтверждено:**

- `runtime/src/current_plugin.rs`, `CurrentPlugin::new`: `WasiCtx::new(random, clocks, sched, table)` — `inherit_network` не вызывается, `socket_addr_check` не настраивается.
- `wasi-common` 43 (зависимость Extism 1.30.0) **не имеет `sock_open`** в трейте `WasiSnapshotPreview1`. Проверено: в `crates/wasi-common/src/snapshots/preview_1.rs` (тег v43.0.0) реализованы только `sock_accept`, `sock_recv`, `sock_send`, `sock_shutdown`. `sock_open` отсутствует.
- `socket_addr_check` — это API **preview2** (`wasmtime_wasi::p2`), а Extism использует **p1** (`wasi-common`). То есть `socket_addr_check` в p1-контексте Extism неприменим в принципе.

**Уточнение формулировки:** «плагин физически не может открыть TCP-соединение через WASI» — **верно**, но причина не в отсутствии `inherit_network` (это p2-концепция), а в том, что **в p1-трейте `wasi-common` 43 нет `sock_open`**. `inherit_network` в p2 — это сахар над `socket_addr_check(|_,_| true)`, и он не имеет отношения к p1.

**Практическое следствие:** `std::net::TcpStream::connect` в WASI-плагине не слинкуется (нет импорта `sock_open`). Это подтверждается тем, что packwiz использует `extism_pdk::http::request`, а не `std::net`.

### c. `allowed_paths` поддерживает `ro:` — **ПОДТВЕРЖДЕНО**

`runtime/src/current_plugin.rs`:

```rust
if let Some(a) = &manifest.allowed_paths {
    for (k, v) in a.iter() {
        let readonly = k.starts_with("ro:");
        let dir_path = if readonly { &k[3..] } else { k };
        let dir = wasi_common::sync::dir::Dir::from_cap_std(
            wasi_common::sync::Dir::open_ambient_dir(dir_path, auth)?,
        );
        let file: Box<dyn wasi_common::dir::WasiDir> = if readonly {
            Box::new(readonly_dir::ReadOnlyDir::new(dir))
        } else {
            Box::new(dir)
        };
        ctx.push_preopened_dir(file, v)?;
    }
}
```

**Формулировка точна.** Дополнение: `ro:` — недокументированная фича (нет в `manifest/schema.json` и в docs), но реально работающая.

### d. `Manifest::new()` по умолчанию не разрешает HTTP — **ПОДТВЕРЖДЕНО**

`manifest/src/lib.rs`:

```rust
pub struct Manifest {
    ...
    #[serde(default)]
    /// Specifies which hosts may be accessed via HTTP, if this is empty then
    /// no hosts may be accessed. Wildcards may be used.
    pub allowed_hosts: Option<Vec<String>>,
    ...
}

impl Manifest {
    pub fn new(wasm: impl IntoIterator<Item = impl Into<Wasm>>) -> Manifest {
        Manifest { wasm: wasm.into_iter().map(|x| x.into()).collect(), ..Default::default() }
    }
}
```

`Default::default()` → `allowed_hosts: None`. В `http_request`: `else { false }` → запрос отклонён. **Формулировка точна.**

### e. `Plugin::call` синхронный, реентрантность запрещена — **ПОДТВЕРЖДЕНО**

`runtime/src/plugin.rs`:

```rust
pub fn call<'a, 'b, T: ToBytes<'a>, U: FromBytes<'b>>(
    &'b mut self,
    name: impl AsRef<str>,
    input: T,
) -> Result<U, Error> {
    let lock = self.instance.clone();
    let mut lock = lock.try_lock().map_err(|e| match e {
        TryLockError::Poisoned(_) => anyhow::anyhow!(
            "instance lock was poisoned; previous thread panicked while calling into wasm"
        ),
        TryLockError::WouldBlock => anyhow::anyhow!("cannot make reentrant calls into plugin"),
    })?;
    ...
}
```

**Формулировка точна**, включая точный текст ошибки `"cannot make reentrant calls into plugin"`.

### f. `function_exists` проверяет только сигнатуру — **ПОДТВЕРЖДЕНО**

`runtime/src/plugin.rs`:

```rust
pub fn function_exists(&self, function: impl AsRef<str>) -> bool {
    self.modules[MAIN_KEY]
        .get_export(function.as_ref())
        .map(|x| {
            if let Some(f) = x.func() {
                let (params, mut results) = (f.params(), f.results());
                match (params.len(), results.len()) {
                    (0, 1) => matches!(results.next(), Some(wasmtime::ValType::I32)),
                    (0, 0) => true,
                    _ => false,
                }
            } else { false }
        })
        .unwrap_or(false)
}
```

**Формулировка точна.** Дополнение: проверка идёт по `modules[MAIN_KEY]` — то есть по **статическому модулю**, а не по инстансу. Это значит, что `function_exists` работает до инстанцирования.

### g. `http_request` не следует редиректам и имеет лимит ответа — **ЧАСТИЧНО НЕВЕРНО**

**Лимит ответа — ПОДТВЕРЖДЕНО:**

```rust
let max = if let Some(max) = &data.manifest.memory.max_http_response_bytes {
    reader.take(*max + 1).read_to_end(&mut buf)?;
    *max
} else {
    reader.take(1024 * 1024 * 50 + 1).read_to_end(&mut buf)?;
    1024 * 1024 * 50
};

if buf.len() > max as usize {
    anyhow::bail!("HTTP response exceeds the configured maximum number of bytes: {max}")
}
```

**Дефолт = 50 MiB** (`1024 * 1024 * 50`). Формулировка точна.

**Редиректы — НЕВЕРНО.** Extism использует `ureq::run(req)`:

```rust
let agent = ureq::agent();
let config = agent.configure_request(r.body(())?).http_status_as_error(false);
let req = config.timeout_global(timeout).build();
ureq::run(req)
```

`ureq::run` → `Agent::new_with_defaults()` → `max_redirects` **по умолчанию 10** (подтверждено в `ureq` 3.x: `ConfigBuilder::max_redirects` — «Defaults to 10»). То есть **`http_request` СЛЕДУЕТ редиректам (до 10)**.

**Это важно:** аудит утверждал, что «Extism `http_request` не следует редиректам — это хороший дефолт». Это **неверно**. Редирект-обход `allowed_hosts` **возможен**: `allowed_hosts` проверяется только для исходного URL, а редирект на другой хост не проверяется. Это **усиливает** SSRF-риск, а не снижает его.

### h. `with_memory_max`, `with_timeout`, `with_fuel_limit`, `with_hash` — **ПОДТВЕРЖДЕНО, но с уточнением по расположению**

| API                                   | Где                                | Подтверждение                                                                                                                                             |
| ------------------------------------- | ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Manifest::with_memory_max(u32)`      | **`Manifest`**, не `PluginBuilder` | `manifest/src/lib.rs`: `pub fn with_memory_max(mut self, max: u32) -> Self { self.memory.max_pages = Some(max); self }`                                   |
| `Manifest::with_timeout(Duration)`    | **`Manifest`**                     | `manifest/src/lib.rs`: `pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self { self.timeout_ms = Some(timeout.as_millis() as u64); self }` |
| `PluginBuilder::with_fuel_limit(u64)` | **`PluginBuilder`**                | `runtime/src/plugin_builder.rs`: `pub fn with_fuel_limit(mut self, fuel: u64) -> Self { self.options.fuel = Some(fuel); self }`                           |
| `Wasm::with_hash(String)`             | **`Wasm`**                         | `manifest/src/lib.rs`: `pub fn with_hash(mut self, hash: impl Into<String>) -> Self { self.meta_mut().hash = Some(hash.into()); self }`                   |

**Уточнение:** аудит в §5.2 писал «Применить `memory_limit`, `timeout`, `fuel_limit`, `Wasm::with_hash`» и в §6 шаг 0.1 предлагал `builder.with_memory_max(pages)` — **это неверно**: `with_memory_max` вызывается на `Manifest`, а не на `PluginBuilder`. В коде шага 0.1 это привело бы к ошибке компиляции.

---

## 3. Собственный код aether-core

### A1: `run_command` без allowlist + fallback на сырую строку — **ПОДТВЕРЖДЕНО**

```rust
// host_functions/features/core.rs
pub(crate) async fn handle_run_command(
    plugin_id: &str,
    command: CommandDto,
    container: &AetherContainer,
) -> crate::Result<OutputDto> {
    let command_for_log = command.clone();
    log::debug!("Processing command from plugin: {command_for_log:?}");

    let host_command =
        plugin_utils::plugin_command_to_host(plugin_id, &command, &container.location_info())?;
    let mut cmd = host_command.to_tokio_command();

    log::debug!("Running command: {host_command:?}");
    let output = cmd.output().await.map_err(|_err| {
        crate::ErrorKind::CoreError(format!("Failed to run command: {cmd:?}")).as_error()
    })?;
    ...
}
```

```rust
// plugin_utils.rs
let resolved_program = plugin_path_to_host(id, &command.program, location_info).map_or_else(
    |_| command.program.clone(),   // ← fallback на сырую строку
    |p| p.to_string_lossy().to_string(),
);
```

Grep по `allowlist|allowed_programs|allowed_commands|whitelist` в `aether-core/src/**` — **0 совпадений**. Нет таймаута, нет лимита вывода. **Подтверждено полностью.**

### A2: `execute_async` использует `block_in_place` / создаёт новый Runtime — **ПОДТВЕРЖДЕНО**

```rust
// shared/execute_async/infra/mod.rs
pub fn execute_async<F: Future>(future: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(future)),
        _ => tokio::runtime::Runtime::new()
            .expect("Failed to create runtime")
            .block_on(future),
    }
}
```

Вызывается из **12 мест** в host-функциях (проверено grep):

- `core.rs:86` — `run_command`
- `instance.rs:115,129,144,157,169,181` — `instance_get_dir`, `instance_plugin_get_dir`, `instance_create`, `list_content`, `enable_contents`, `disable_contents`
- `java.rs:48,60` — `get_java`, `install_java`

**Подтверждено полностью.**

### A7: `manifest.validate()` только в тестах — **ПОДТВЕРЖДЕНО**

Grep по `manifest\.validate|\.validate\(&PLUGIN_API_VERSION|runtime\.validate` в `aether-core/src/**`:

```
plugin_manifest.rs:97:        self.runtime.validate()?;          ← определение метода
plugin_manifest.rs:286:            manifest.validate(&semver::Version::new(0, 1, 0)),   ← тест
plugin_manifest.rs:316:            manifest.validate(&semver::Version::new(1, 0, 0)),   ← тест
plugin_manifest.rs:345:        assert!(manifest.validate(&semver::Version::new(0, 1, 0)).is_ok());  ← тест
```

**Ни одного вызова в `EnablePluginUseCase` или где-либо в реальном пути загрузки.** Подтверждено полностью.

### A8: `get_default_allowed_paths` монтирует весь `instances_dir` — **ПОДТВЕРЖДЕНО**

```rust
// plugin_utils.rs
pub fn get_default_allowed_paths(
    location_info: &LocationInfo,
    plugin_id: &str,
) -> HashMap<String, PathBuf> {
    HashMap::from([
        (
            location_info.plugin_cache_dir(plugin_id).to_string_lossy().to_string(),
            PathBuf::from("/cache".to_owned()),
        ),
        (
            location_info.instances_dir().to_string_lossy().to_string(),
            PathBuf::from("/instances"),
        ),
    ])
}
```

`instances_dir()` = `config_dir.join(INSTANCES_FOLDER_NAME)` — корень всех инстансов. **Подтверждено полностью.**

### A14: `PluginContentProviderProxy::call_plugin` держит `Mutex` на всё время `plugin.call` — **ПОДТВЕРЖДЕНО**

```rust
async fn call_plugin<I, O>(&self, handler_name: &str, input: I) -> Result<O, InstanceError>
where I: Serialize, O: DeserializeOwned,
{
    let mut plugin = self.instance.lock().await;   // ← tokio::sync::Mutex
    let plugin_id = plugin.get_id();

    if !plugin.supports(handler_name) { ... }

    plugin
        .call::<Msgpack<I>, Msgpack<O>>(handler_name, Msgpack(input))   // ← синхронный вызов
        .map(|res| res.0)
        .map_err(|err| { ... InstanceError::ContentProviderError { reason: err.to_string() } })
}
```

`self.instance` — `Arc<Mutex<dyn PluginInstance>>` (из `use tokio::sync::Mutex`). Лок держится до конца функции, включая синхронный `plugin.call`. **Подтверждено полностью.**

### A5: `ZipPluginExtractor` использует `archive.extract()` без `enclosed_name()` — **ПОДТВЕРЖДЕНО**

```rust
// zip_plugin_extractor/extractor.rs
let temp_dir = TempDir::new().map_err(IoError::from)?;
archive
    .extract(&temp_dir)
    .map_err(|_| PluginError::FileExtractionFailed { from: source_path })?;
```

Grep по `enclosed_name` в `aether-core/src/**` — **0 совпадений**. **Подтверждено полностью.**

---

## 4. Текущие версии зависимостей

`packages/core/aether-core/Cargo.toml` — **не изменился** с момента аудита:

```toml
extism = "1.30.0"
extism-convert = "1.30.0"
```

Корневой `Cargo.toml` — тоже без изменений в части extism. **Выводы о «последней версии» и API остаются в силе.**

---

## 5. Полный список провайдеров контента

| Провайдер                   | Тип                                                       | Как скачивает                                                                            | Использует `run_command`                             | Источник                                                                 |
| --------------------------- | --------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------ |
| **Modrinth**                | Встроенный (`ModrinthContentProvider`)                    | `RequestClient::fetch_bytes` → `write_async` в temp, затем `rename` в `instance_dir`     | **Нет**                                              | `instance/infra/content_providers/modrinth/`                             |
| **packwiz**                 | WASM-плагин (importer + updater, **не** content provider) | `extism_pdk::http::request` (встроенный `http_request`) + `std::fs::write`/`copy` (WASI) | **Да** — `java -jar packwiz-installer-bootstrap.jar` | `Z:\Projects\git\packwiz-plugin` (main)                                  |
| **packwiz (feat/optimize)** | WASM-плагин (`packManagers`)                              | То же                                                                                    | **Да**                                               | `feat/optimize` (не собирается)                                          |
| **CurseForge**              | **Не существует**                                         | —                                                                                        | —                                                    | Только упоминания в тестовых фикстурах (`'curseforge'` как строка в SQL) |

**Проверено:**

- `instance/infra/content_providers/mod.rs` содержит только `mod modrinth; pub use modrinth::ModrinthContentProvider;`
- `apps/desktop/src/core/app/container.rs:331-345` регистрирует **только** `ModrinthContentProvider`
- Grep по `curseforge|technic|ftb|atlauncher|gdlauncher|PrismLauncher|MultiMC` — совпадения только в тестовых SQL-фикстурах и `java/infra/discovery_paths.rs` (пути PrismLauncher для поиска Java, не провайдер)

**Итого: 2 провайдера — Modrinth (встроенный) и packwiz (WASM, importer/updater).** CurseForge и другие — не реализованы.

---

## 6. Сводка верификации

### 6.1. Подтвердилось точно (без оговорок)

| #   | Утверждение                                                                                                  |
| --- | ------------------------------------------------------------------------------------------------------------ |
| A1  | `run_command` без allowlist, с fallback на сырую строку, без таймаута и лимита вывода                        |
| A2  | `execute_async` использует `block_in_place` / создаёт новый Runtime; вызывается из 12 host-функций           |
| A5  | `ZipPluginExtractor` использует `archive.extract()` без `enclosed_name()`                                    |
| A7  | `manifest.validate()` вызывается только в тестах                                                             |
| A8  | `get_default_allowed_paths` монтирует весь `instances_dir`                                                   |
| A14 | `PluginContentProviderProxy::call_plugin` держит `tokio::sync::Mutex` на всё время синхронного `plugin.call` |
| 2a  | `allowed_hosts` проверяется только в `http_request`                                                          |
| 2c  | `allowed_paths` поддерживает `ro:`                                                                           |
| 2d  | `Manifest::new()` → `allowed_hosts = None` → HTTP запрещён                                                   |
| 2e  | `Plugin::call` синхронный, реентрантность запрещена, точный текст ошибки                                     |
| 2f  | `function_exists` проверяет только сигнатуру (0 параметров, 0/1 i32)                                         |
| 2h  | Все четыре API существуют                                                                                    |
| 4   | Версии `extism`/`extism-convert` не изменились                                                               |

### 6.2. Подтвердилось с оговорками / неточностями

| #      | Что неточно                                                                                             | Уточнение                                                                                                                                                                                                                                                                                                                                                                  |
| ------ | ------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **2b** | «WASI p1 sockets не подключены (`inherit_network` не вызывается, `socket_addr_check` не настраивается)» | **Вывод верен, обоснование неверно.** `inherit_network` и `socket_addr_check` — это **preview2** API (`wasmtime_wasi::p2`), а Extism использует **p1** (`wasi-common`). Реальная причина: в `wasi-common` 43 **нет `sock_open`** в трейте `WasiSnapshotPreview1` (есть только `sock_accept`/`sock_recv`/`sock_send`/`sock_shutdown`). Проверено по исходникам тега v43.0.0 |
| **2g** | «`http_request` не следует редиректам»                                                                  | **НЕВЕРНО.** Extism использует `ureq::run(req)` → `Agent::new_with_defaults()` → `max_redirects` **по умолчанию 10**. `http_request` **следует редиректам**. Лимит ответа 50 MiB — подтверждён                                                                                                                                                                             |
| **2h** | «`PluginBuilder::with_memory_max`»                                                                      | **НЕВЕРНО по расположению.** `with_memory_max` — метод **`Manifest`**, не `PluginBuilder`. `with_timeout` — тоже `Manifest`. `with_fuel_limit` — `PluginBuilder`. `with_hash` — `Wasm`. Шаг 0.1 плана миграции в текущем виде **не скомпилируется**                                                                                                                        |
| **A6** | «`api.features` не проверяется вообще»                                                                  | **Подтверждено** (grep по `api\.features                                                                                                                                                                                                                                                                                                                                   | features\.contains | requested_features`— 0 совпадений). Но в аудите это подано как отдельная находка, хотя фактически это **следствие** того, что`manifest.validate()` не вызывается (A7) |

### 6.3. Не подтвердилось вообще

| #                       | Утверждение                                                                                                                                         | Реальность                                                                                                                                                                                                                                                                            |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Проблема №3 (аудит)** | «Установка одиночного контента и установка модпака идут по двум разным путям: разным методам трейта провайдера» — как описание проблемы **packwiz** | **packwiz на `main` вообще не является content provider.** Его `capabilities.json` содержит только `importers` и `updaters`, без `contentProviders`. Проблема №3 относится **только** к гипотетическим будущим WASM-провайдерам контента, но не к существующему packwiz               |
| **§5.2 п.3 (аудит)**    | «WASI остаётся, но только read-only… Запись — только через host-функции» — как решение проблемы №1                                                  | Решение остаётся валидным, но **не учитывает**, что packwiz пишет в инстанс через `std::fs::copy` (`ensure_resource_in_instance_directory`) и `std::fs::write` (`download_redistributable`). Переход на `ro:` **сломает packwiz** без предварительного добавления host-функций записи |
| **§6 шаг 0.1 (аудит)**  | `builder.with_memory_max(pages)`                                                                                                                    | Не скомпилируется — метод на `Manifest`                                                                                                                                                                                                                                               |
| **§6 шаг 0.1 (аудит)**  | «`with_timeout` … дефолт 30 мин»                                                                                                                    | `with_timeout` — на `Manifest`, не на `PluginBuilder`. Технически применимо, но код в шаге написан для `builder`                                                                                                                                                                      |

### 6.4. Новые находки, которых не было в аудите

| #      | Находка                                                                                                                                         | Значимость                                                                                                                                                                                                                                                                           |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **N1** | **`http_request` следует редиректам (до 10), и `allowed_hosts` проверяется только для исходного URL**                                           | **Высокая.** Это реальный обход `allowed_hosts`: плагин с `allowed_hosts: ["github.com"]` может запросить `https://github.com/redirect?to=http://169.254.169.254/...` и хост выполнит запрос к metadata-сервису. Аудит утверждал обратное («не следует редиректам — хороший дефолт») |
| **N2** | **`feat/optimize` — незавершённый архитектурный рефакторинг в направлении, отличном от аудита**                                                 | **Высокая.** Вводит `packManagers` (императивный `install`) вместо `InstallPlan` (декларативный). Совпадает с аудитом только в одном: ядро создаёт инстанс. **Не собирается** против текущего `aether` (нет `PackSourceDto`/`PackMetadataDto`/`PackInstallParamsDto`/`packManagers`) |
| **N3** | **packwiz использует `get_id`, `instance_plugin_get_dir`, `get_java`, `install_java`** — 4 host-функции, не упомянутые в вашем списке           | Средняя. Ваш список был неполным                                                                                                                                                                                                                                                     |
| **N4** | **`ref/refactor-content-installing` — чисто технический рефакторинг** (edition 2024, удаление `phf`/`lazy_static`/`serr`, rustfmt)              | Низкая, но важно: не путать с `feat/optimize`                                                                                                                                                                                                                                        |
| **N5** | **`PluginCapabilitiesDto` не имеет `packManagers`** — то есть `feat/optimize` требует изменений и в `aether-core-plugin-api`, и в `aether-core` | Средняя. Объём работ по `feat/optimize` больше, чем выглядит                                                                                                                                                                                                                         |
| **N6** | **`function_exists` проверяет `modules[MAIN_KEY]` (статический модуль), а не инстанс**                                                          | Низкая. Работает до инстанцирования — это скорее плюс                                                                                                                                                                                                                                |

---

## 7. Что должно измениться в плане миграции

### 7.1. Обязательные изменения

#### 1. Шаг 0.1 — исправить API-вызовы

```rust
// Было (не скомпилируется):
builder = builder.with_memory_max(pages);

// Должно быть:
// memory_limit применяется к Manifest, а не к PluginBuilder
let wasm_manifest = Manifest::new([wasm_file])
    .with_allowed_hosts(...)
    .with_allowed_paths(...)
    .with_memory_max(pages)                    // ← Manifest
    .with_timeout(Duration::from_secs(1800));  // ← Manifest

// fuel_limit — на PluginBuilder:
let mut builder = PluginBuilder::new(&wasm_manifest)
    .with_functions(...)
    .with_wasi(true)
    .with_fuel_limit(1_000_000_000);           // ← PluginBuilder
```

#### 2. Добавить в Фазу 0 новый шаг: запрет редиректов в `http_request`

Это **невозможно** сделать через API Extism 1.30.0 — `http_request` использует `ureq::run` с дефолтным агентом, и `max_redirects` не настраивается через манифест. Варианты:

- (a) Не использовать встроенный `http_request` вообще — перейти на свою host-функцию `http_get` с `redirect::Policy::none()` (это уже есть в Фазе 1, но теперь становится **приоритетом Фазы 0**, а не Фазы 1);
- (b) Принять риск и документировать его;
- (c) Проверять `allowed_hosts` **после** запроса (по финальному URL) — но это уже поздно, запрос выполнен.

**Рекомендация:** перенести `http_get` из Фазы 1 в Фазу 0 и сделать его **обязательным** для плагинов, которым нужен HTTP. Встроенный `http_request` оставить как fallback с документированным риском.

#### 3. Шаг 0.5 (`ro:` на instances) — переставить после Фазы 1

Аудит уже отмечал эту зависимость, но теперь она **подтверждена конкретным кодом packwiz**:

- `ensure_resource_in_instance_directory` → `std::fs::copy(Path::new("/cache").join(resource_name), &file)` — запись в инстанс;
- `download_redistributable` → `std::fs::write(path, res.body())` — запись в `/cache` (это ок, `/cache` остаётся rw).

То есть `ro:` на `/instances` **сломает packwiz** на шаге `preload_resources`. Нужно сначала добавить `write_instance_file` (шаг 1.3), потом переключать.

#### 4. Проблема №3 — переформулировать

Аудит описывал её как «два несовместимых контракта для одиночной установки и установки модпака». Фактически:

- **packwiz не участвует** в этой проблеме (он importer/updater);
- проблема существует **только** для `ContentProvider` (Modrinth + гипотетические WASM-провайдеры);
- `feat/optimize` предлагает **третье** решение (`packManagers`), которое не совпадает ни с текущим состоянием, ни с `InstallPlan`.

**Рекомендация:** перед Фазой 2 нужно **явно решить**, что делать с `feat/optimize`:

- (a) **Отменить** `feat/optimize` и идти по `InstallPlan` — тогда `packManagers` не нужен;
- (b) **Принять** `packManagers` как основу и адаптировать `InstallPlan` под него — тогда `InstallPlan` становится частью `packManagers`, а не отдельной capability;
- (c) **Слить** — `packManagers` для «управления модпаками» (install/update/checkUpdates/resolveMetadata), `InstallPlan` для «резолвинга файлов» внутри `install`.

Это **блокирующее решение** для Фазы 2.

#### 5. Проблема №4 — переформулировать

Аудит предлагал `ContentSource::ProviderRef` для переиспользования провайдеров. Но:

- packwiz не резолвит отдельные файлы — он делегирует jar;
- `feat/optimize` вводит `resolveMetadata`, но не `resolveFile`;
- для `ProviderRef` нужен `resolve_file` в `ContentProvider`, которого нет ни в текущем коде, ни в `feat/optimize`.

**Рекомендация:** проблема №4 остаётся валидной, но её решение (`ProviderRef` + `resolve_file`) требует **нового** метода, которого нет ни в одной из ветвей. Это не «миграция», а новая разработка.

### 7.2. Изменения, которые НЕ нужны

- **§5.2 п.1** (`InstallPlan` + `ProviderRef`) — остаётся валидным, но требует решения по `feat/optimize` (см. п.4 выше).
- **§5.2 п.2** (`http_get`/`download_to_cache`) — остаётся валидным, но **приоритет повышается** из-за N1.
- **§5.2 п.4** (`run_command` → `run_declared_process`) — остаётся валидным и **подтверждено** кодом packwiz (`java -jar`).
- **§5.2 п.5-12** — остаются валидными.

### 7.3. Обновлённый порядок фаз

```
Фаза 0 (безопасность, не ломает):
  0.1 лимиты (ИСПРАВИТЬ: with_memory_max/with_timeout на Manifest)
  0.2 хеш wasm
  0.3 zip-slip fix
  0.4 manifest.validate() при загрузке
  0.5 run_command allowlist + декларация
  0.6 ★ НОВОЕ: http_get с redirect::Policy::none() — приоритет из-за N1

Фаза 1 (host-функции I/O):
  1.1 http_post_json (http_get уже в 0.6)
  1.2 download_to_cache
  1.3 write_instance_file / read_instance_file
  1.4 host_functions в манифесте

Фаза 1.5 (★ НОВОЕ: сужение путей):
  1.5.1 ro: на instances_dir (теперь безопасно — есть write_instance_file)
  1.5.2 миграция packwiz на write_instance_file

Фаза 2 (InstallPlan) — БЛОКИРОВАНА решением по feat/optimize:
  2.0 ★ РЕШЕНИЕ: отменить / принять / слить feat/optimize
  2.1-2.7 как было

Фаза 3 (чистка):
  3.1-3.5 как было
```

---

## 8. Итог

**Подтверждено точно:** 13 утверждений (A1, A2, A5, A7, A8, A14, 2a, 2c, 2d, 2e, 2f, 2h, версии зависимостей).

**Подтверждено с неточностями:** 4 (2b — неверное обоснование; 2g — **фактически неверно** про редиректы; 2h — неверное расположение `with_memory_max`; A6 — следствие A7).

**Не подтвердилось:** проблема №3 в применении к packwiz (он не content provider); шаг 0.1 плана не скомпилируется.

**Новые находки:** 6 (N1–N6), из которых **N1 (редиректы в `http_request`) — критичная** и меняет приоритеты, а **N2 (`feat/optimize`) — блокирующая** для Фазы 2.

**Главный вывод:** аудит в целом верен, но требует трёх правок перед построением плана работ:

1. Исправить API-вызовы в шаге 0.1 (`with_memory_max`/`with_timeout` — на `Manifest`).
2. Повысить приоритет `http_get` до Фазы 0 из-за обхода `allowed_hosts` через редиректы.
3. Принять решение по `feat/optimize` до Фазы 2 — это блокирующий вопрос.
