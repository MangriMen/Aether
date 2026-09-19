План рефакторинга: ContentProvider, установка контента, плагины
Основан на: plugin-architecture-audit.md (аудит) + audit_clarification.md (верификация по исходникам Extism v1.30.0, репозиторию packwiz-plugin и коду aether-core) + решения, принятые в обсуждении 2026-09-20.

Контекст решений: активных пользователей мало, фундамент не зафиксирован — можно менять и aether-core, и плагины без оглядки на обратную совместимость. feat/optimize/packManagers отбрасывается как отдельная концепция (его полезная часть покрывается ниже). Планируется второй WASM-провайдер — CurseForge — поэтому ProviderRef/resolve_file не откладываются "на будущее", а строятся сразу, во второй фазе.

Принятые архитектурные решения (не дискутируются заново)
InstallPlan — единый декларативный контракт для одиночной установки и модпаков. InstallTarget::Existing | New покрывает разницу между install и update — отдельная capability packManagers не нужна.
resolve_pack_metadata — лёгкий отдельный вызов (без полного резолва файлов) для preview перед установкой и для check-updates. Единственное, что стоило забрать из feat/optimize.
ContentSource::ProviderRef + ContentProvider::resolve_file — механизм, которым один плагин (например, packwiz) ссылается на контент другого провайдера (Modrinth, CurseForge) без прямого вызова плагин→плагин. Резолвит ядро, обходя план.
packwiz переписывается на нативный парсинг pack.toml, jar (packwiz-installer.jar) через run_command — не используется. Delegated-путь как промежуточный вариант не нужен (мало пользователей — можно сразу сделать правильно).
CurseForge — WASM-плагин, а не встроенный провайдер, собирается сразу на новый контракт (resolve_install_plan/resolve_file), без оглядки на старый install_atomic.
Фаза 0 — Безопасность (не ломает ничего существующего)
Делается в первую очередь независимо от прочих фаз — самые серьёзные находки аудита (A1, A2, A3, A4, A5, A7, A8, N1) здесь, и ни одна не требует переписывания плагинов.

Шаг Что делать Закрывает Примечание
0.1 Применить лимиты: Manifest::with_memory_max, Manifest::with_timeout, PluginBuilder::with_fuel_limit A3 Важно: with_memory_max/with_timeout — методы Manifest, не PluginBuilder. with_fuel_limit — метод PluginBuilder. Не перепутать (в исходном аудите была ошибка компиляции на этом месте)
0.2 Wasm::with_hash(sha256) при загрузке плагина; считать sha256 при скачивании из GitHub-релиза A4 Хеш опционален для локально устанавливаемых плагинов на этапе разработки
0.3 ZipPluginExtractor: entry.enclosed_name() вместо archive.extract() A5 Простая точечная правка
0.4 Вызывать manifest.validate() (в т.ч. runtime.validate()) в EnablePluginUseCase при загрузке A6, A7 api.features реализовывать пока не обязательно — сначала просто не давать абсолютные allowed_paths
0.5 run_command → allowlist программ + таймаут + лимит вывода + current_dir/env изоляция A1 Не убирать функцию совсем — она пригодится другим плагинам в будущем, но без allowlist это RCE. Fallback на сырую строку в plugin_path_to_host — убрать, ошибка вместо fallback
0.6 ★ Приоритет из-за N1. Собственная host-функция http_get с reqwest::redirect::Policy::none(), проверкой allowed_hosts в хосте (а не только в Extism), запретом на приватные/loopback IP N1 (обход allowed_hosts через редирект в 10 хопов) Встроенный http_request Extism оставить как fallback для простых случаев с документированным риском редиректов
Итог фазы: ни один существующий плагин (packwiz) не ломается. Все изменения — сужение прав и добавление проверок, а не смена контрактов.

Фаза 1 — Host-функции для I/O
Добавляет инфраструктуру, которая понадобится Фазе 2 и переписанному packwiz. Ничего не удаляет из старого пути.

Шаг Что делать
1.1 http_post_json(url, body) -> HttpResponse — тот же контроль, что у http_get (0.6), но POST + JSON-only
1.2 download_to_cache(url, hash) -> CachedFile — скачивание в /cache с верификацией хеша, ретраями, единым RequestClient
1.3 write_instance_file(instance_id, relative_path, bytes) / read_instance_file(instance_id, relative_path) — единственный способ писать/читать в инстанс без прямого WASI-доступа на запись
1.4 Декларация используемых host-функций в манифесте плагина (api.host_functions: [...]), проверка при загрузке, что все запрошенные функции существуют — закрывает отсутствие версионирования контракта (A10 из исходного аудита)
Фаза 1.5 — Сужение путей (зависит от 1.3)
Критическая зависимость, подтверждённая кодом packwiz: ensure_resource_in_instance_directory пишет в инстанс через std::fs::copy. Если сделать ro: на instances_dir раньше, чем появится write_instance_file, packwiz сломается на preload_resources. Поэтому эта фаза идёт строго после 1.3, а не вместе с Фазой 0, как в первой версии плана.

Шаг Что делать
1.5.1 allowed_paths: ro: на instances_dir вместо чтения+записи. /cache остаётся rw (плагину нужно туда писать временные файлы)
1.5.2 Мигрировать packwiz на write_instance_file вместо std::fs::copy/std::fs::write в инстанс
Фаза 2 — InstallPlan + ProviderRef (основной рефакторинг)
2.1. Доменная модель
pub struct InstallPlan {
pub target: InstallTarget,
pub files: Vec<InstallFileEntry>,
pub overrides: Option<OverrideSource>,
}

pub enum InstallTarget {
Existing { instance_id: String },
New(NewInstanceSpec),
}

pub struct InstallFileEntry {
pub source: ContentSource,
pub relative_path: String,
pub content_type: ContentType,
pub optional: bool,
pub size: Option<u64>,
}

pub enum ContentSource {
Direct { url: String, hash: Option<FileHash> },
ProviderRef { provider_id: ProviderId, content_id: String, version: Option<String> },
}

pub enum FileHash { Sha1(String), Sha512(String) }

pub enum OverrideSource {
Archive { archive_path: PathBuf, subdir: String },
Directory { path: PathBuf },
}
2.2. Трейт ContentProvider #[async_trait]
pub trait ContentProvider: Send + Sync {
async fn search(&self, params: ContentSearchParams) -> Result<ContentSearchResult, InstanceError>;
async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError>;
async fn list_versions(&self, content_id: String) -> Result<Vec<ContentVersion>, InstanceError>;

    /// Резолвинг одиночного файла — используется executor'ом при обходе ProviderRef.
    async fn resolve_file(
        &self,
        content_id: &str,
        version: Option<&str>,
        game_version: &str,
        loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError>;

    /// Резолвинг полного плана установки — для одиночного контента и для модпаков одинаково.
    async fn resolve_install_plan(&self, params: ResolveInstallPlanParams) -> Result<InstallPlan, InstanceError>;

    /// Лёгкий предпросмотр без резолва файлов — для UI и check-updates.
    async fn resolve_pack_metadata(&self, source: PackSource) -> Result<PackMetadata, InstanceError>;

    async fn check_compatibility(&self, ...) -> Result<..., InstanceError>;

}
resolve_pack_metadata реализуют только провайдеры модпак-форматов (packwiz и т.п.) — для одиночного контента (Modrinth mod, CF mod) можно вернуть UnsupportedOperation, UI просто не показывает preview.

2.3. InstallPlanExecutor
impl InstallPlanExecutor {
async fn execute(&self, plan: InstallPlan) -> Result<(), InstanceError> {
let instance_id = match plan.target {
InstallTarget::New(spec) => self.instance_service.create(spec).await?,
InstallTarget::Existing { instance_id } => instance_id,
};

        // Резолв ProviderRef → Direct (с ограничением глубины рекурсии на случай кривого манифеста)
        let resolved_files = self.resolve_all_refs(plan.files, depth_limit: 3).await?;

        // Диффинг для update: сравнить resolved_files с уже установленными ContentFile
        let (to_download, to_remove) = self.diff_against_installed(&instance_id, &resolved_files).await?;

        for file in to_download {
            emit(Progress::Downloading { .. });
            let temp = self.downloader.download(&file.url, &file.hash).await?;
            emit(Progress::Verifying { .. });
            self.hash_service.verify(&temp, &file.hash)?;
            emit(Progress::Installing { .. });
            self.write_instance_file(&instance_id, &file.relative_path, temp).await?;
            self.pack_storage.save_content_file(&instance_id, &file).await?;
        }
        for path in to_remove {
            self.remove_content_file(&instance_id, &path).await?;
        }
        if let Some(overrides) = plan.overrides {
            self.extract_overrides(&instance_id, overrides).await?;
        }
        emit(Progress::Complete { instance_id });
        Ok(())
    }

}
Install и update — один и тот же путь. Разница не в контракте плагина (он всегда просто резолвит "актуальный план"), а в том, что делает InstallPlanExecutor: для нового инстанса — скачать всё, для существующего — сравнить с уже установленными ContentFile и скачать только дельту.

2.4. Порядок миграции провайдеров
Шаг Действие
2.4.1 Мигрировать ModrinthContentProvider на resolve_install_plan + resolve_file. Он встроенный (не WASM) — не требует изменений в плагин-API, делается первым как эталон
2.4.2 Написать новый CurseForge-плагин (WASM) сразу на resolve_install_plan/resolve_file/resolve_pack_metadata. Никакого install_atomic — ему не с чем быть обратно совместимым, он ещё не существует
2.4.3 Переписать packwiz: убрать run_command/jar полностью. Парсить pack.toml (простой TOML) нативно в Rust внутри плагина. Записи с прямым download.url → ContentSource::Direct. Записи с CurseForge project/file id → ContentSource::ProviderRef { provider_id: "curseforge", .. }. Добавить resolve_pack_metadata для preview/update-check
2.4.4 Обновить PluginContentProviderProxy — прокидывает новые методы трейта в вызовы плагина через Msgpack
Фаза 3 — Чистка
Шаг Что делать
3.1 Удалить install_atomic/install_modpack из трейта ContentProvider (все провайдеры уже мигрированы — обратная совместимость не нужна)
3.2 Удалить LoadConfig::Native из схемы манифеста (не реализован, заявлен как обход всей sandbox-модели) или явно задокументировать «не поддерживается»
3.3 Разделить PluginError на PluginError (жизненный цикл + исполнение) и PluginProviderError (скачивание плагина из GitHub) — сейчас смешаны разные bounded context'ы
3.4 Убрать block_in_place/создание нового Runtime из execute_async; вынести вызов плагина в spawn_blocking на уровне PluginContentProviderProxy. Закрывает блокировку tokio-воркера (A2) и держание Mutex на время синхронного вызова (A14)
3.5 Структурированные ошибки на границе WASM: различать Trap/Timeout/OutOfFuel/OutOfMemory/PluginError/NonZeroExit через call_get_error_code + анализ текста ошибки
Сводная схема зависимостей
Фаза 0 (безопасность)
0.1 лимиты → 0.2 хеш wasm → 0.3 zip-slip → 0.4 validate() → 0.5 run_command allowlist → 0.6 http_get (приоритет из-за N1)
│
▼
Фаза 1 (I/O host-функции)
1.1 http_post_json → 1.2 download_to_cache → 1.3 write/read_instance_file → 1.4 host_functions в манифесте
│
▼
Фаза 1.5 (сужение путей) — требует 1.3
1.5.1 ro: на instances_dir → 1.5.2 packwiz на write_instance_file
│
▼
Фаза 2 (InstallPlan + ProviderRef)
2.1 domain model → 2.2 трейт ContentProvider → 2.3 InstallPlanExecutor
→ 2.4.1 Modrinth → 2.4.2 CurseForge (новый) → 2.4.3 packwiz нативный → 2.4.4 Proxy
│
▼
Фаза 3 (чистка)
3.1 удалить старые методы → 3.2 удалить LoadConfig::Native → 3.3 разделить ошибки
→ 3.4 spawn_blocking → 3.5 структурированные ошибки
Фазы 0 и 1 можно вести параллельно с проектированием Фазы 2 (они не блокируют друг друга). Фаза 1.5 обязательно после 1.3. Фаза 3 — только после того, как все провайдеры (Modrinth, CurseForge, packwiz) реально работают на новом контракте.

Что явно НЕ делаем (отклонённые варианты)
packManagers/feat/optimize как отдельная capability — избыточен, InstallTarget уже покрывает разницу install/update.
Delegated-план / jar-путь для packwiz — не нужен как постоянное или даже промежуточное решение, пользователей мало, можно сразу написать нативный парсер pack.toml.
invoke_provider (прямой вызов плагин→плагин через хост) — реентерабельность запрещена в Extism 1.30.0 ("cannot make reentrant calls into plugin"), ProviderRef решает ту же задачу без этого риска.
Полный отказ от WASI (вариант "всё через host-функции") — слишком большой breaking relative к выгоде; ro:-монтирование + write_instance_file дают достаточный аудит записи.
