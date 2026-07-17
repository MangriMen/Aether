**Промпт для ИИ-агента:**

Мы проводим масштабный рефакторинг ядра лаунчера Aether. Наша цель — устранить дублирование логики между модами и модпаками, реализовав паттерн, где плагины выступают исключительно как **источники данных**, а распаковка/установка делегируется **хэндлерам ядра**. Строго соблюдай принципы гексагональной архитектуры (Hexagonal Architecture) и изоляции бизнес-логики от инфраструктуры.

Выполни рефакторинг по следующим шагам:

### Шаг 1: Доменные модели и контракты (Domain Layer)

1. **Удали** трейты `PackManager` и `ContentProvider` (и связанные с ними структуры метаданных).
2. **Создай** единый трейт `ContentSource` в `core/domain/capabilities.rs` (или аналогичном файле):

```rust
#[async_trait]
pub trait ContentSource: Send + Sync {
    fn metadata(&self) -> &ContentSourceCapabilityMetadata;
    async fn search(&self, params: ContentSearchParams) -> Result<ContentSearchResult, InstanceError>;
    async fn get_version_info(&self, content_id: &str, version_id: &str) -> Result<VersionInfo, InstanceError>;
}

```

3. **Определи** структуры данных для `get_version_info`:

```rust
pub struct VersionInfo {
    pub version_id: String,
    pub version_name: String,
    pub payload: VersionPayload,
}

pub enum VersionPayload {
    Asset(DownloadInstruction),
    Modpack {
        format_id: String,
        manifest_bytes: Vec<u8>,
    }
}

```

### Шаг 2: Обработчики жизненного цикла (Core / Domain Services)

1. **Создай** трейт `PackLifecycleHandler` для обработки специфичных форматов сборок:

```rust
#[async_trait]
pub trait PackLifecycleHandler: Send + Sync {
    fn format_id(&self) -> &str;
    async fn deploy_pack(
        &self,
        manifest_bytes: &[u8],
        instance_id: &str,
        ctx: &DownloadContext
    ) -> Result<(), InstanceError>;

    async fn update_pack(
        &self,
        new_manifest_bytes: &[u8],
        instance_id: &str,
        ctx: &DownloadContext
    ) -> Result<(), InstanceError>;
}

```

2. Подготовь простой in-memory реестр для хэндлеров (ключ — `String` (format_id), значение — `Arc<dyn PackLifecycleHandler>`).

### Шаг 3: Инфраструктура плагинов (Infrastructure Layer)

1. **Удали** `PluginPackManagerProxy`.
2. **Переименуй** `PluginContentProviderProxy` в `PluginContentSourceProxy` и обнови реализацию.

- Метод `get_version_info` должен вызывать WASM-функцию через Extism, передавая аргументы и получая результат через сериализацию `Msgpack`.
- Плагин теперь возвращает сырые байты манифеста модпака прямо через `Msgpack` сериализацию внутри `VersionPayload::Modpack`.

3. Обнови `PluginInfrastructureListener`: он должен слушать события `PluginState::Loaded` и регистрировать только `ContentSource` (удалить любую логику, связанную с `PackManager`).

### Шаг 4: Слой приложения (Use Cases)

1. **Удали** `InstallPackUseCase`.
2. **Обнови** `InstallContentUseCase`. Теперь это единая точка входа.

- Входные параметры: `source_id: ProviderId`, `content_id: String`, `version_id: String`, `target_instance_id: Option<String>`.
- **Логика:**

1. Резолвит `ContentSource` из `CapabilityRegistry`.
2. Вызывает `get_version_info`.
3. Выполняет `match` по `payload`.
4. `VersionPayload::Asset(instruction)`: проверяет наличие `target_instance_id` (ошибка, если нет), вызывает `ContentFileService::install_content_file` или аналогичный метод загрузки атомарного файла.
5. `VersionPayload::Modpack { format_id, manifest_bytes }`:

- Если `target_instance_id` равен `None`, вызывает `CreateInstanceUseCasePort` для создания нового пустого инстанса.
- Ищет нужный `PackLifecycleHandler` в реестре по `format_id`.
- Вызывает `handler.deploy_pack(manifest_bytes, ...)` передавая скачивание внутрь.

### Шаг 5: DI Контейнер (Services & Composition Root)

1. В `AetherContainer` и `PluginRegistry` замени старые реестры на единый `Arc<dyn CapabilityRegistry<Arc<dyn ContentSource>>>`.
2. Добавь в контейнер реестр `PackLifecycleHandler` и пробрось его в `InstallContentUseCase`.

Выводи измененный код частями, фокусируясь на правильном маппинге DTO и строгой типизации. Начни с Шага 1 и Шага 2 (Доменные трейты и Core Services).
