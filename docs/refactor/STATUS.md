# Статус рефакторинга установки контента

Индекс задач. Детали каждой — в `tasks/<ID>.md`. Протокол работы — `../../AGENT_INSTRUCTIONS.md`.
Решения владельца — `OPEN_QUESTIONS.md`. Расхождения плана с кодом — `FINDINGS.md`.

Обновлено: 2026-09-22 — T-0.1 (лимиты Extism), T-0.3 (zip-slip) и T-0.6 (host-функция `http_get`,
приоритетная находка N1) выполнены; T-0.4 (manifest.validate()) реализована и покрыта тестами, но
`blocked` — не выполнен ручной сценарий из карточки (нет GUI/Tauri и доступа к `../packwiz-plugin`
в той сессии); T-0.2 (sha256 wasm, TOFU) реализована и покрыта тестами, тоже `blocked` по той же
причине — ручной сценарий требует GUI и плагина из GitHub-релиза; T-0.5 (ужесточение
`run_command`: allowlist по `java_dir()`, таймаут, лимит вывода, env-изоляция) реализована и
покрыта 20 тестами, `blocked` по той же причине — обязательный ручной сценарий (обновление
packwiz-пака) требует GUI; по итогам T-0.6 заведена T-1.5.C (packwiz всё ещё ходит через
встроенный `http_request`, см. F-16), по итогам T-0.2 — T-0.2.A (пометка «не верифицирован» в UI,
см. F-19), по итогам T-0.5 — F-21 (поле в `OutputDto` ломает ABI, флаг «обрезано» добавлять
в T-1.4) и F-22 (allowlist отсекает Java, найденную вне ядра); реализация остальных задач
не начиналась. Флаг «обрезано» по F-21 вписан в карточку T-1.4 (раздел
«Дополнение из T-0.5»), по F-22 заведены T-0.5.A и Q10.

**Фаза 0 по коду закрыта целиком.** Три её задачи (T-0.2, T-0.4, T-0.5) ждут только ручной
проверки владельцем — у всех трёх один и тот же блокер: нужен запуск GUI/Tauri.
T-0.5.A — не часть этого закрытия, а последствие самого решения Q7; ничего не блокирует.

Статусы: `not_started` · `in_progress` · `done` · `blocked` · `needs_decision` · `ОТЛОЖЕНО`.
`ОТЛОЖЕНО` = задача остаётся в плане, но **не входит в текущий проход** (см. Q1, Q3).
Репозитории: `aether` (этот), `packwiz` (`../packwiz-plugin`), `cf` (`../curseforge-plugin`).
Модели: `Flash` = DeepSeek V4.1 Flash · `Sonnet` = Claude Sonnet 5 · `Opus` = Claude Opus 5.
Критерии выбора модели — в `AGENT_INSTRUCTIONS.md`, раздел 7.

## Сводка

| Показатель | Значение |
| --- | --- |
| Всего задач | 37 |
| Активных | 29 |
| Отложено | 8 (CurseForge — 3, packwiz-native — 4, T-3.1 — 1) |
| Активных на Flash | 1 |
| Активных на Sonnet | 10 |
| Активных на Opus (с обязательным ревью) | 16 |

## Фаза 0 — Безопасность

| ID | Задача | Репо | Шаг плана | Зависит от | Модель | Ревью | Статус |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T-0.1 | Лимиты Extism: memory / timeout / fuel | aether + packwiz | 0.1 | — | Flash | да | done |
| T-0.2 | Проверка sha256 wasm (TOFU) | aether | 0.2 | — | Opus | да | blocked |
| T-0.2.A | Пометка «плагин не верифицирован» в UI (F-19) | aether | — | T-0.2 | Sonnet | нет | not_started |
| T-0.3 | Zip-slip в `ZipPluginExtractor` | aether | 0.3 | — | Flash | нет | done |
| T-0.4 | Вызов `manifest.validate()` при включении плагина | aether | 0.4 | — | Sonnet | нет | blocked |
| T-0.5 | Ужесточение `run_command` | aether | 0.5 | — | Opus | да | blocked |
| T-0.5.A | Признак «Java управляется ядром» вместо сравнения по директории (F-22) | aether | — | Q10 | Opus | да | needs_decision |
| T-0.6 | Host-функция `http_get` (SSRF/редиректы) | aether | 0.6 | — | Opus | да | done |

## Фаза 1 — Host-функции для I/O

| ID | Задача | Репо | Шаг плана | Зависит от | Модель | Ревью | Статус |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T-1.1 | `http_post_json` | aether | 1.1 | T-0.6 | Sonnet | да | not_started |
| T-1.2 | `download_to_cache` | aether | 1.2 | T-0.6 | Opus | да | not_started |
| T-1.3 | `write_instance_file` / `read_instance_file` | aether | 1.3 | — | Opus | да | not_started |
| T-1.4 | Декларация `api.hostFunctions` в манифесте + флаг «обрезано» в `OutputDto` (F-21) | aether + packwiz | 1.4 | T-1.1, T-1.2, T-1.3 | Opus | да | not_started |

## Фаза 1.5 — Сужение путей

| ID | Задача | Репо | Шаг плана | Зависит от | Модель | Ревью | Статус |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T-1.5.A | packwiz: запись в инстанс через host-функцию | packwiz | 1.5.2 | T-1.3 | Sonnet | нет | not_started |
| T-1.5.B | `ro:` на `instances_dir` | aether | 1.5.1 | T-1.5.A | Sonnet | да | not_started |
| T-1.5.C | packwiz: `http_request` → `http_get` (F-16) | packwiz | — | T-0.6 | Sonnet | нет | not_started |

## Фаза 2 — InstallPlan + ProviderRef

| ID | Задача | Репо | Шаг плана | Зависит от | Модель | Ревью | Статус |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T-2.1.A | Доменная модель `InstallPlan` | aether | 2.1 | — | Opus | да | not_started |
| T-2.1.B | Модели резолва: params / `PackMetadata` / `ResolvedFile` | aether | 2.1–2.2 | T-2.1.A | Opus | да | not_started |
| T-2.2 | Расширение трейта `ContentProvider` | aether | 2.2 | T-2.1.B | Opus | да | not_started |
| T-2.3.A | `InstallPlanExecutor`: скелет + резолв `ProviderRef` | aether | 2.3 | T-2.2 | Opus | да | not_started |
| T-2.3.B | Диффинг install/update против `PackStorage` | aether | 2.3 | T-2.3.A | Opus | да | not_started |
| T-2.3.C | Применение `overrides` (архив / директория) | aether | 2.3 | T-2.3.A | Sonnet | нет | not_started |
| T-2.3.D | Прогресс-события выполнения плана | aether | 2.3 | T-2.3.A | Sonnet | нет | not_started |
| T-2.4.1 | Modrinth на новый контракт (+ `resolve_pack_metadata`) | aether | 2.4.1 | T-2.3.B | Opus | да | not_started |
| T-2.4.4 | Контракт плагина: `ProviderHandlers` + `PluginContentProviderProxy` | aether + plugin-api | 2.4.4 | T-2.2 | Opus | да | not_started |
| T-2.5 | Подключение executor к use case / Tauri / фронту | aether | 2.3–2.4 | T-2.4.1 | Sonnet | нет | not_started |
| T-2.4.2.A | CurseForge-плагин: приведение скелета в рабочий вид | cf | 2.4.2 | Q3 при возобновлении | Sonnet¹ | нет | **ОТЛОЖЕНО — CurseForge отложен** |
| T-2.4.2.B | CurseForge: `search` / `get_content` / `list_versions` | cf | 2.4.2 | T-2.4.2.A | Sonnet¹ | нет | **ОТЛОЖЕНО — CurseForge отложен** |
| T-2.4.2.C | CurseForge: `resolve_file` / `resolve_install_plan` / metadata | cf | 2.4.2 | T-2.4.2.B | Opus¹ | да | **ОТЛОЖЕНО — CurseForge отложен** |
| T-2.4.3.A | packwiz: нативный парсер `pack.toml` + индекс | packwiz | 2.4.3 | — | Sonnet¹ | нет | **ОТЛОЖЕНО — packwiz-native отложен (Q1)** |
| T-2.4.3.B | packwiz: `resolve_install_plan` / `resolve_file` | packwiz | 2.4.3 | T-2.4.3.A, T-2.4.4, T-2.4.2.C | Opus¹ | да | **ОТЛОЖЕНО — packwiz-native отложен (Q1)** |
| T-2.4.3.C | packwiz: `resolve_pack_metadata` | packwiz | 2.4.3 | T-2.4.3.A | Sonnet¹ | нет | **ОТЛОЖЕНО — packwiz-native отложен (Q1)** |
| T-2.4.3.D | packwiz: удаление jar/`run_command`, судьба importer/updater | packwiz + aether | 2.4.3 | T-2.4.3.B, T-2.4.3.C | Opus¹ | да | **ОТЛОЖЕНО — packwiz-native отложен (Q1)** |

¹ Модель проставлена на будущее; для текущего прохода задача неактуальна.

## Фаза 3 — Чистка

| ID | Задача | Репо | Шаг плана | Зависит от | Модель | Ревью | Статус |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T-3.2 | Убрать `LoadConfig::Native` | aether | 3.2 | — | Flash | нет | not_started |
| T-3.3 | Разделить `PluginError` / `PluginProviderError` | aether | 3.3 | — | Sonnet | нет | not_started |
| T-3.4 | Убрать `block_in_place` из host-функций | aether | 3.4 | — | Opus | да | not_started |
| T-3.5 | Структурированные ошибки на границе WASM | aether | 3.5 | T-3.3 | Opus | да | not_started |
| T-3.1 | Удалить `install_atomic` / `install_modpack` | aether + plugin-api | 3.1 | T-2.4.1, T-2.4.2.C, T-2.4.3.D | Sonnet¹ | нет | **ОТЛОЖЕНО — требует мигрированных packwiz и CF** |

## Рекомендуемый порядок старта

1. **T-0.3**, **T-0.4** — механические, ничего не ломают, можно параллельно.
2. **T-0.1**, **T-0.6** — лимиты песочницы и самая серьёзная находка (N1).
3. **T-0.2**, **T-0.5** — оставшаяся безопасность Фазы 0.
4. **T-1.3 → T-1.5.A → T-1.5.B** — закрывает прямую запись плагина в инстанс.
5. **T-2.1.A → T-2.1.B → T-2.2** — фундамент Фазы 2; проектируется сразу расширяемым
   под будущий CurseForge (Q3), но без его реализации.

Фаза 3 в части T-3.1 остаётся отложенной, пока packwiz и CurseForge не мигрированы.
Остальные задачи Фазы 3 (T-3.2 … T-3.5) от этого не зависят и активны.

Внимание: открыт **Q10** — первый вопрос после первичного разбора, ждёт решения владельца (блокирует T-0.5.A).
