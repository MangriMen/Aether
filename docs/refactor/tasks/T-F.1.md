# T-F.1 — Инстанс помечается `Installed` до установки содержимого пака

| Поле | Значение |
| --- | --- |
| Статус | not_started |
| Шаг плана | — (вне плана; баг, найденный при разборе ветки `fix/fix-instance-intall-stage-during-modpack-installing`) |
| Репозиторий | aether |
| Зависит от | — |
| Модель | Flash — правка однозначна и полностью описана ниже |
| Ревью перед мержем | нет |

## Суть
`InstallInstanceUseCase::mark_installed` безусловно ставит
`InstanceInstallStage::Installed` сразу после установки Minecraft и лоадера — даже если инстанс
создан под пак и содержимое пака ещё не ставилось. Стадия `PackInstalling`, заведённая ровно под
этот случай, на пути **первичной** установки не выставляется никем.

Проверено на HEAD (`grep` по всему репозиторию):

- `install_stage.rs:8` — `PackInstalling` существует, док: *«Instance created for pack, but the
  pack hasn't been fully installed yet»*.
- `update_instance.rs:62` — **единственное** место, где она выставляется (путь обновления).
- `install_instance.rs:48` — путь первичной установки ставит `Installed` без условий.
- `launch_instance.rs:107` — запуск блокируется по `PackInstalling` / `Installing`
  (`InstanceStillInstalling`). На первичной установке этот guard сработать не может.

**Окно достижимо**, а не гипотетично. Путь Modrinth-модпака
(`modrinth_content_provider.rs:404-423`): `NewInstance { pack_info: Some(..),
skip_install_instance: None }` → `CreateInstanceUseCase::setup_instance` зовёт
`instance_install_service.execute(...)` → стадия становится `Installed` → и только **после**
этого идёт `deploy_modpack_files`. В этом промежутке инстанс выглядит полностью установленным.

Последствия: пользователь может запустить наполовину собранный модпак (без модов), а
`instance_event_handler.rs:56` и health-check в `launch_instance.rs:132` тоже принимают решения
по `Installed`.

## Что сделать
Правка на четыре строки, взята с ветки
`fix/fix-instance-intall-stage-during-modpack-installing` (коммит `fe4c1b94`) — это её
единственная часть, не относящаяся к отвергнутой концепции `PackManager` (см. Q12):

```rust
// packages/core/aether-core/src/features/instance/app/use_cases/instance/install_instance.rs
if instance.pack_info.is_some() {
    instance.install_stage = InstanceInstallStage::PackInstalling;
} else {
    instance.install_stage = InstanceInstallStage::Installed;
}
```

Дополнительно проверить (правка может быть неполной):

1. **Кто снимает `PackInstalling` после успешной установки пака.** На ветке за это отвечал
   новый `install_pack`, которого на HEAD нет. Нужно убедиться, что на путях
   `deploy_modpack_files` (Modrinth) и `Importer::import` (плагины) стадия доводится до
   `Installed`, иначе инстанс залипнет в `PackInstalling` и перестанет запускаться —
   это было бы хуже исходного бага.
2. **Путь отказа.** `handle_failed_installation` сбрасывает в `NotInstalled` только если стадия
   `!= Installed`; с новым условием туда будет попадать и `PackInstalling` — проверить, что
   это желаемое поведение.

## Файлы
- `packages/core/aether-core/src/features/instance/app/use_cases/instance/install_instance.rs`
- `packages/core/aether-core/src/features/instance/infra/content_providers/modrinth/content_provider/modrinth_content_provider.rs` — довод стадии до `Installed`
- `packages/core/aether-core/src/features/instance/app/use_cases/instance/import_instance.rs` — то же для importer-пути

## Как проверить
- Unit-тест: `InstallInstanceUseCase` с `pack_info: Some(..)` оставляет стадию `PackInstalling`,
  с `pack_info: None` — `Installed`. Заготовки моков есть в `create_instance_test.rs`.
- Unit-тест: после успешной установки пака стадия становится `Installed` (регрессия на залипание
  из пункта 1 выше).
- `cargo test --workspace` — зелёный.
- Ручной сценарий (нужен GUI, может быть недоступен — тогда `blocked`): начать установку
  Modrinth-модпака и во время скачивания модов попробовать запустить инстанс. Ожидаемо:
  отказ `InstanceStillInstalling`. До фикса запуск проходит.

## Проверка перед мержем (компенсация уровня модели)
Задача на Flash, раздел обязателен:
- [ ] Убедиться, что **все** пути установки пака доводят стадию до `Installed` — иначе инстанс
      залипнет в `PackInstalling` навсегда. Это главный риск этой правки, а не сама правка.
- [ ] Проверить `handle_failed_installation`: `PackInstalling` теперь попадает в ветку сброса
      в `NotInstalled`.
- [ ] Проверить, что фронт не показывает `PackInstalling` как ошибку или вечный спиннер
      (`grep` по `apps/frontend/src` показал, что стадия во фронте не упоминается явно — значит
      попадает в ветку «по умолчанию», убедиться, что она осмысленна).

## Ловушки
- Не тащить с ветки ничего сверх этих четырёх строк: там же лежит концепция `PackManager`,
  которую план отверг (Q12). Правка самодостаточна.
- Делать **до** Фазы 2: T-2.3.B и T-2.5 будут переписывать логику стадий, и после них тот же
  фикс придётся переносить в другой код.

## Осталось
— нет (задача не начиналась).

## Журнал
<!-- ГГГГ-ММ-ДД | модель | что сделано | коммит -->
2026-09-22 | Claude Opus 5 | Карточка заведена по итогам разбора ветки `fix/fix-instance-intall-stage-during-modpack-installing`. Баг подтверждён на HEAD по коду (не воспроизводился в GUI): `PackInstalling` выставляется только в `update_instance`, путь первичной установки ставит `Installed` безусловно, окно достижимо через Modrinth-модпак. Код не трогался. | не закоммичено
