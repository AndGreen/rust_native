# Детальный план реализации VDOM-first

## 1. Цель и границы

### Цель
Перевести проект из MVP-состояния с full-rebuild обновлениями в VDOM-first архитектуру, где Rust владеет:
1. Каноническим UI-деревом.
2. Инкрементальным diff.
3. Эмиссией мутаций для нативных платформ.
4. Layout-расчетом и выдачей кадров (`LayoutFrame`).

### Что входит в этот план
1. Стабилизация сборки и базовых проверок.
2. Введение канонической схемы протокола (`native_schema`).
3. Введение `vdom_runtime` и транслятора diff -> `Mutation[]`.
4. Канонический layout pipeline в Rust (Taffy -> `LayoutFrame[]`).
5. Новый backend API с батчевым применением мутаций и кадров.
6. iOS incremental renderer.
7. Android parity renderer.
8. CI/тестовые гейты, snapshot-проверки и базовые perf-пороги.

### Что не входит в этот план
1. Расширенный production UX polish (сложные анимации, полная gesture/accessibility parity).
2. Широкое расширение каталога виджетов.
3. Автоматизация публикации в App Store / Play Store.

---

## 2. Текущее состояние (as-is)

1. Сборка не проходит из-за конфликтующих API в widgets.
2. `DiffEngine` фактически replace-only (`Patch::Replace`).
3. Runtime использует контракт backend `mount/update(View)` без канонических mutation batches.
4. iOS backend в `update()` очищает subtree и пересобирает UI заново.
5. Канонические `UiNodeId`, `Mutation`, `LayoutFrame`, `UiEvent` описаны в docs, но не выделены в отдельный runtime-схемный слой кода.
6. Недостаточно системных тестов для защиты контракта мутаций/layout и кроссплатформенной паритетности.

---

## 3. Целевое состояние (to-be)

1. Runtime на каждом frame выполняет строго:
   1. Render/diff в Rust.
   2. Эмиссия `Mutation[]`.
   3. Расчет `LayoutFrame[]`.
   4. Применение на backend: сначала мутации, потом layout, затем flush.
2. iOS и Android поддерживают одинаковую семантику протокола `v1`.
3. События native -> Rust идут через единый `UiEvent` контракт.
4. `counter` и `album_list` работают без регулярного full-tree remount.
5. CI блокирует merge при регрессиях по lint/test/snapshot/perf.

---

## 4. Критический путь

1. `P0-01` -> `P0-02` -> `P0-03` -> `P0-04` -> `P0-05` -> `P0-06` -> `P0-07`.
2. Без `P0-03` и `P0-05` невозможно корректно реализовать incremental backend.
3. `P1` частично может идти параллельно с `P0-06` и `P0-07`, но финальные snapshot/perf-гейты включаются после стабилизации P0.

---

## 5. Поэтапный backlog (P0/P1/P2)

### Формат задач
Каждая задача ниже содержит:
1. Описание.
2. Затрагиваемые crates/files.
3. Зависимости.
4. Риски.
5. Критерии приемки (DoD).
6. Оценку.

### P0 — обязательный путь к M1

#### P0-01 — Стабилизация компиляции и минимальный test baseline
- Приоритет: P0
- Оценка: 2-3 инженерных дня
- Описание:
1. Устранить конфликтующие API имена в `mf_widgets`.
2. Добавить минимальные unit/smoke тесты для widgets/core/runtime.
3. Добиться прохождения `cargo check` на хосте.
- Затрагиваемые crates/files:
1. `crates/widgets/src/text.rs`
2. `crates/widgets/src/image.rs`
3. `crates/widgets/src/lib.rs`
4. `crates/core/src/*` (минимальные smoke tests)
5. `crates/runtime/src/lib.rs` (минимальные smoke tests)
- Зависимости: отсутствуют.
- Риски:
1. Поломка текущего DSL-цепочного API в примерах.
- DoD:
1. `cargo check --workspace` проходит.
2. Есть минимум тестов на создание базовых View-деревьев и repaint-путь.
3. Примеры `counter` и `album_list` собираются.

#### P0-02 — Введение канонического schema-слоя (`native_schema`)
- Приоритет: P0
- Оценка: 3-4 инженерных дня
- Описание:
1. Добавить crate `crates/native_schema`.
2. Вынести канонические типы контракта (элементы, props, мутации, события, версия).
3. Подключить crate в workspace.
- Затрагиваемые crates/files:
1. `Cargo.toml` (workspace members/deps)
2. `crates/native_schema/Cargo.toml`
3. `crates/native_schema/src/lib.rs`
4. `crates/native_schema/src/mutation.rs`
5. `crates/native_schema/src/layout.rs`
6. `crates/native_schema/src/events.rs`
- Зависимости: `P0-01`.
- Риски:
1. Несогласованность названий между docs и кодом.
- DoD:
1. Все канонические типы компилируются и экспортируются из одного места.
2. Добавлены unit tests на базовые инварианты (валидность enum/версии).

#### P0-03 — Новый `vdom_runtime` (diff -> Mutation)
- Приоритет: P0
- Оценка: 8-10 инженерных дней
- Описание:
1. Добавить crate `crates/vdom_runtime`.
2. Реализовать хранение node graph с parent-child связями.
3. Реализовать стабильные `UiNodeId`.
4. Реализовать инкрементальный diff с эмиссией `Mutation[]`.
5. Фильтровать no-op мутации (`SetProp`/`SetText` без изменений).
- Затрагиваемые crates/files:
1. `Cargo.toml`
2. `crates/vdom_runtime/src/lib.rs`
3. `crates/vdom_runtime/src/tree.rs`
4. `crates/vdom_runtime/src/diff.rs`
5. `crates/vdom_runtime/src/translate.rs`
6. `crates/core/src/view.rs` (при необходимости адаптера)
- Зависимости: `P0-02`.
- Риски:
1. Некорректные move/replace сценарии и нарушения инвариантов.
- DoD:
1. Для `counter` update генерирует `SetText` без structural replace.
2. Для list insert/remove/reorder генерируются минимально необходимые structural mutations.
3. Есть детерминированные тесты последовательности мутаций.

#### P0-04 — Канонический layout pipeline (`LayoutFrame[]`)
- Приоритет: P0
- Оценка: 6-8 инженерных дней
- Описание:
1. Построить layout tree из канонических узлов/props.
2. Рассчитывать layout в Rust (Taffy).
3. Выдавать `LayoutFrame[]` для каждого `UiNodeId`.
4. Валидация: один frame на node, без дубликатов id, без отрицательных размеров.
- Затрагиваемые crates/files:
1. `crates/vdom_runtime/src/layout.rs` (или выделенный layout crate)
2. `crates/core/src/layout.rs` (адаптация/реэкспорт)
3. `crates/native_schema/src/layout.rs`
- Зависимости: `P0-03`.
- Риски:
1. Расхождения в измерениях текста между платформами.
- DoD:
1. `LayoutFrame` генерируется для всех узлов render-дерева.
2. Включены fallback-правила для unsupported v1 cases.
3. Есть snapshot tests для nested stacks/lists.

#### P0-05 — Новый backend contract (mutation/layout batches)
- Приоритет: P0
- Оценка: 3-5 инженерных дней
- Описание:
1. Эволюционировать `backend_api` от `mount/update(View)` к батчевому интерфейсу.
2. Ввести этапы применения: `apply_mutations` -> `apply_layout` -> `flush`.
3. Добавить канал событий backend -> runtime (`UiEvent`).
4. На переходный период оставить совместимый адаптер (если нужно для минимизации риска).
- Затрагиваемые crates/files:
1. `crates/backend_api/src/lib.rs`
2. `crates/runtime/src/lib.rs`
3. `crates/vdom_runtime/src/lib.rs`
- Зависимости: `P0-03`, `P0-04`.
- Риски:
1. Ломающее изменение интерфейса для существующих backend crate.
- DoD:
1. Runtime больше не вызывает `update(View)` как основной путь.
2. Все backend implementations компилируются через новый контракт.

#### P0-06 — iOS incremental executor
- Приоритет: P0
- Оценка: 8-10 инженерных дней
- Описание:
1. Убрать rebuild-all поток в iOS backend.
2. Реализовать таблицу `UiNodeId -> NativeHandle`.
3. Реализовать операции `Create/Insert/Move/Replace/Remove/SetProp/SetText`.
4. Реализовать minimal event bridge (`Tap`, `TextInput`, `Appear`, `Disappear`).
5. Обеспечить выполнение UI-мутаций на main thread.
- Затрагиваемые crates/files:
1. `crates/backend_native/src/ios.rs`
2. `crates/backend_native/src/lib.rs`
3. `crates/backend_native/src/fallback.rs` (совместимость поведения на не-iOS)
- Зависимости: `P0-05`.
- Риски:
1. Ошибки владения/жизненного цикла Objective-C объектов.
2. Рассинхрон реестра хендлов и runtime tree.
- DoD:
1. `update` не очищает весь subtree.
2. Применяются только входящие mutation/layout batches.
3. Смоук-сценарии `counter` и `album_list` проходят на iOS.

#### P0-07 — Android parity path
- Приоритет: P0
- Оценка: 10-12 инженерных дней
- Описание:
1. Добавить Android backend (новый crate или модуль) с JNI scaffolding.
2. Повторить семантику iOS mutation executor.
3. Реализовать event bridge с тем же контрактом payload.
4. Обеспечить выполнение UI-операций на UI thread.
- Затрагиваемые crates/files:
1. `crates/backend_android/*` (рекомендуется отдельный crate)
2. `Cargo.toml`
3. Точки интеграции runtime/backend_api
- Зависимости: `P0-05`.
- Риски:
1. JNI/Threading edge-cases.
2. Семантический drift относительно iOS.
- DoD:
1. `counter` и `album_list` проходят на Android через тот же протокол.
2. Паритет базовых mutation scenarios с iOS подтвержден тестами.

### P1 — надежность, паритет, CI-гейты

#### P1-01 — Golden snapshots (mutations/layout)
- Приоритет: P1
- Оценка: 4-5 инженерных дней
- Описание:
1. Ввести версионированные fixtures (`v1`) для canonical screens.
2. Проверять mutation sequence snapshots.
3. Проверять layout frame snapshots.
- Зависимости: `P0-03`, `P0-04`.
- DoD:
1. Snapshot diff обязателен в PR review.

#### P1-02 — Negative/failure-mode тесты и recovery
- Приоритет: P1
- Оценка: 3-4 инженерных дня
- Описание:
1. Покрыть invalid ID, missing parent, unknown prop, cycles.
2. Реализовать batch reject + runtime resync affected subtree.
- Зависимости: `P0-05`, `P0-06`, `P0-07`.
- DoD:
1. Hard violations приводят к recoverable resync, без падения процесса.

#### P1-03 — Perf-бюджеты и no-op фильтрация
- Приоритет: P1
- Оценка: 3-5 инженерных дней
- Описание:
1. Добавить synthetic benchmark: large list updates.
2. Добавить no-op mutation count checks.
3. Добавить churn test create/remove cycles.
- Зависимости: `P0-03`, `P0-06`, `P0-07`.
- DoD:
1. Метрики зафиксированы, превышение порогов блокирует merge (nightly/optional strict gate).

#### P1-04 — CI quality gates
- Приоритет: P1
- Оценка: 2-3 инженерных дня
- Описание:
1. Обязательные проверки: fmt/check/clippy/test.
2. Snapshot verification.
3. Пайплайн smoke-скриптов для iOS/Android.
- Зависимости: `P1-01`, `P1-02`.
- DoD:
1. Main branch защищен обязательными статус-чеками.

### P2 — hardening и подготовка к следующему roadmap

#### P2-01 — Диагностика runtime/backend
- Приоритет: P2
- Оценка: 3-4 инженерных дня
- Описание:
1. Mutation trace logging.
2. Debug dump node tree/registry.
3. Feature flags для диагностики.

#### P2-02 — API freeze prep и migration notes
- Приоритет: P2
- Оценка: 2-3 инженерных дня
- Описание:
1. Зафиксировать v1 public surface.
2. Подготовить migration notes от pre-v1 API.

#### P2-03 — RC checklist
- Приоритет: P2
- Оценка: 1-2 инженерных дня
- Описание:
1. Свести known issues list.
2. Подготовить release candidate checklist.

---

## 6. Интерфейсы и типы (целевые изменения)

### 6.1 Канонические идентификаторы и версии
1. `type UiNodeId = u32`
2. `enum ProtocolVersion { V1 }`

### 6.2 Мутации (Rust -> Native)
1. `CreateNode { id, kind }`
2. `CreateTextNode { id, text }`
3. `SetText { id, text }`
4. `SetProp { id, key, value }`
5. `InsertChild { parent, child, index }`
6. `MoveNode { id, new_parent, index }`
7. `ReplaceNode { old, new_id, kind }`
8. `RemoveNode { id }`
9. `AttachEventListener { id, event }`

### 6.3 Layout
1. `LayoutFrame { id, x, y, width, height }`
2. Правила: parent-first apply order, no duplicate ids, отрицательные размеры запрещены.

### 6.4 События (Native -> Rust)
1. `Tap { id }`
2. `TextInput { id, value }`
3. `Scroll { id, dx, dy }`
4. `Appear { id }`
5. `Disappear { id }`

### 6.5 Backend API (целевой контракт)
1. `apply_mutations(&[Mutation]) -> Result<(), BatchError>`
2. `apply_layout(&[LayoutFrame]) -> Result<(), BatchError>`
3. `flush() -> Result<(), BackendError>`
4. `drain_events() -> Vec<UiEvent>`

### 6.6 Политика ошибок
1. Soft error: unknown prop/optional event -> ignore + warn.
2. Hard error: invalid IDs / invariant violation -> reject batch.
3. Recovery: runtime выполняет resync affected subtree.

---

## 7. Тестовая стратегия и acceptance-сценарии

### Unit tests
1. Widget -> schema mapping.
2. Mutation translator invariants.
3. Layout normalization/defaults.
4. Event parsing/validation.

### Golden snapshots
1. Snapshot последовательности мутаций для `counter` и `album_list`.
2. Snapshot кадров layout для canonical screens.
3. Версионирование фикстур по `v1`.

### Integration tests
1. Full render loop с synthetic events.
2. End-to-end сценарий: native event -> runtime update -> expected mutation delta.
3. Mock backend contract tests.

### Platform smoke tests
1. iOS build + launch + basic interactions.
2. Android build + launch + basic interactions.
3. Сценарные скрипты для `counter` и `album_list`.

### Performance tests
1. Large list mutation throughput.
2. No-op update mutation count.
3. Create/remove memory churn.

### Acceptance scenarios (обязательные)
1. `counter`: increment изменяет только text node (без structural remount).
2. `album_list`: insert/remove/reorder отражаются минимальными structural mutations.
3. Error path: invalid node ID -> batch reject -> subtree resync.

---

## 8. Риски, контрмеры и recovery

1. Риск: модельные расхождения schema/runtime/backend.
   1. Контрмера: единый `native_schema` crate + contract tests.
2. Риск: lifecycle ошибки в iOS/Android bridge.
   1. Контрмера: ownership policy + stress tests create/remove cycles.
3. Риск: layout drift между платформами.
   1. Контрмера: Rust authoritative frames + parity snapshots.
4. Риск: mutation storm/perf degradation.
   1. Контрмера: no-op filtering + coalescing + perf thresholds.
5. Риск: hard protocol violations на runtime.
   1. Контрмера: reject batch + deterministic subtree resync.

---

## 9. Оценка трудоёмкости и команда

### До M1 (ориентиры)
1. 1 инженер (Rust + mobile): 12-16 недель.
2. 2 инженера (Rust + iOS/Android): 8-10 недель.
3. 3 инженера (Rust + iOS + Android): 6-8 недель.

### Распределение работ (рекомендуемое)
1. Rust Core Engineer: `P0-02`..`P0-05`, snapshots/perf framework.
2. iOS Engineer: `P0-06`, iOS smoke/test harness.
3. Android Engineer: `P0-07`, Android smoke/test harness.

---

## 10. Definition of Done по фазам

### Фаза A: Стабилизация
1. Workspace собирается (`check/test/clippy`).
2. Базовые smoke tests добавлены.

### Фаза B: Канонический runtime слой
1. `native_schema` и `vdom_runtime` введены.
2. Инкрементальные мутации работают на базовых кейсах.

### Фаза C: Layout и backend контракт
1. `LayoutFrame[]` стабильно генерируется и валидируется.
2. Backend API переведен на batch-модель.

### Фаза D: Платформенный execution
1. iOS incremental renderer стабилен.
2. Android parity реализован.

### Фаза E: Надежность и CI hardening
1. Snapshot/perf/negative tests в CI.
2. Recovery paths проверены.

---

## 11. Итоговые критерии готовности M1

1. Протокол `v1` реализован в коде и зафиксирован в docs.
2. Инкрементальные обновления — основной путь на iOS и Android.
3. Layout вычисляется в Rust и консистентно применяется backend-ами.
4. `counter` и `album_list` работают через mutation/layout pipeline без регулярного full remount.
5. CI enforce:
   1. `cargo fmt --check`
   2. `cargo check --workspace`
   3. `cargo clippy --workspace -- -D warnings`
   4. `cargo test --workspace`
   5. snapshot checks
   6. smoke checks

---

## 12. Допущения и выбранные defaults

1. Документ является execution-backlog и не заменяет архитектурные документы.
2. Источники архитектурной истины:
   1. `docs/architecture_vdom.md`
   2. `docs/mutation_protocol.md`
   3. `docs/layout_contract.md`
3. Если реализация расходится с этим backlog, приоритет у канонических контрактов и инвариантов, после чего обновляется backlog.
