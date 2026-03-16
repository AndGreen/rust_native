# Пошаговый план обновления `rust_native` для переноса `smuzy_swift`

## 1. Назначение документа

Этот документ превращает [smuzy_gap_analysis_ru.md](/Users/andreyzelenin/projects/rust_native/docs/smuzy_gap_analysis_ru.md) в последовательный план реализации.

Цель:

1. обновить `rust_native` до состояния, в котором `smuzy_swift` можно переписать без критической зависимости от ad-hoc Swift UI кода;
2. сохранить архитектурный принцип `Rust owns truth`;
3. развивать reusable framework surface, а не app-specific обходные пути.

План рассчитан на iOS-first реализацию с близким UX-паритетом относительно `smuzy_swift`.

---

## 2. Принципы реализации

### Что считается успехом

Фреймворк считается достаточно обновленным, когда Smuzy можно собрать в Rust через:

1. framework widgets/layout;
2. framework navigation/presentation;
3. framework services;
4. framework persistence layer;
5. iOS backend без SwiftUI-hosted экранов.

### Что не делаем

1. Не клонируем SwiftUI API 1:1.
2. Не добавляем отдельные `Rectangle`, `Circle`, `RoundedRectangle`.
3. Не смешиваем platform services с `Mutation`/`LayoutFrame` протоколом.
4. Не строим сразу Android parity до стабилизации iOS-first surface.

### Базовые архитектурные решения

1. Вводим единый `Container` как базовый визуальный primitive.
2. Добавляем `Pressable` как общий interactive wrapper; `Button` остается удобным sugar.
3. Platform services оформляем как отдельный effect/service channel.
4. Storage слой делаем reusable, но минимальным по первой версии.

---

## 3. Целевое состояние фреймворка

После завершения плана фреймворк должен уметь:

1. строить layered UI через `Container` и `ZStack`;
2. рендерить dense layouts через `Grid` и `Wrap`;
3. обрабатывать tap/press/drag/scroll и lifecycle events;
4. управлять tabs, navigation и modal sheets из Rust;
5. предоставлять date picker, context menu, toast и icons;
6. вызывать share/import/haptics/app metadata services из Rust UI;
7. хранить и реактивно читать Smuzy data model;
8. удерживать identity/focus в списках и формах без лишних remount;
9. обеспечивать достаточно точный layout для chip/grid-based экранов.

---

## 4. Пошаговый roadmap

## Этап 0. Подготовка каркаса

### Шаг 0.1. Зафиксировать vocabulary и ownership

Что сделать:

1. Уточнить в коде и docs, что базовый shape primitive называется `Container`.
2. Зафиксировать, что overlay/background/presentation/services вводятся как framework-level capabilities.
3. Привязать новый план к `smuzy_gap_analysis_ru.md` как source of truth по требованиям.

Результат:

1. дальнейшие этапы не расходятся по терминам;
2. implementer не выбирает между shape-widget model и container model.

---

## Этап 1. Visual foundation

### Шаг 1.1. Ввести `Container` в public widget API

Что сделать:

1. Добавить `Container` в `mf_widgets`.
2. Разрешить два режима:
   - пустой visual box;
   - container с child subtree.
3. Добавить builder API для:
   - `width`
   - `height`
   - `min_width`
   - `min_height`
   - `max_width`
   - `max_height`
   - `background`
   - `opacity`
   - `shadow`
   - `border`
   - `stroke`
   - `corner_radius`
   - `corner_radius_per_corner`
   - `full_round`
   - `offset`

Решения:

1. Не вводить shape-specific widgets поверх этого этапа.
2. `full_round` трактовать как "максимально скруглить относительно финального frame".
3. `corner_radius_per_corner` должен иметь приоритет над общим `corner_radius`.

Результат:

1. можно выразить day cell, chip, outlined button, ring/highlight и decorative box.

### Шаг 1.2. Расширить schema и VDOM под `Container`

Что сделать:

1. Добавить `ElementKind::Container`.
2. Добавить `PropKey` и `PropValue` для visual model:
   - border width/color;
   - stroke width/color;
   - shadow color/radius/offset;
   - corner radius;
   - per-corner radius;
   - full round flag;
   - offset x/y.
3. Научить `vdom_runtime` каноникализации `Container`.
4. Научить diff корректно обновлять `Container` props.

Решения:

1. Visual props должны обновляться через `SetProp`, а не через `ReplaceNode`.
2. Container identity должна сохраняться при смене стиля.

### Шаг 1.3. Реализовать iOS backend mapping для `Container`

Что сделать:

1. На iOS маппить `Container` на `UIView`.
2. Применять visual props через `UIView`/`CALayer`.
3. Реализовать clipping и corner model.
4. Обеспечить корректное обновление при repeated `SetProp`.

Решения:

1. Если border/stroke окажутся разными по модели, использовать `CALayer`/sublayer path, не forcing full remount.
2. `full_round` пересчитывать после применения frame.

### Шаг 1.4. Добавить layout modifiers как first-class API

Что сделать:

1. Вынести explicit frame API в public widgets, а не держать только в schema.
2. Добавить per-edge padding API.
3. Добавить `offset`.
4. Добавить background/overlay composition path.

Решения:

1. Overlay/background реализовывать через декларативную композицию и canonical children model, а не специальный UIKit-only bypass.

Результат этапа 1:

1. framework может выразить базовую визуальную геометрию Smuzy;
2. shape model закрыта через единый `Container`.

---

## Этап 2. Interaction foundation

### Шаг 2.1. Добавить `Pressable`

Что сделать:

1. Ввести `Pressable` как generic wrapper вокруг child view.
2. Поддержать:
   - `on_tap`
   - `on_press_change`
   - `on_long_press`
3. Оставить `Button` как высокоуровневый sugar поверх `Pressable`.

Решения:

1. Все non-textual interactive elements Smuzy строить через `Pressable + Container`.
2. `Button` не расширять отдельно, если `Pressable` уже покрывает custom-content сценарии.

### Шаг 2.2. Расширить event schema и runtime

Что сделать:

1. Добавить новые `EventKind`.
2. Добавить новые `UiEvent` payloads.
3. Провести их через `backend_native` и `vdom_runtime`.
4. Добавить dispatch path в `mf_runtime`.

### Шаг 2.3. Добавить drag и scroll event baseline

Что сделать:

1. Добавить `on_drag` с payload:
   - translation x/y;
   - predicted end translation x/y.
2. Добавить scroll events для будущего `ScrollView`.
3. Реализовать iOS bindings для pan/scroll.

Результат этапа 2:

1. можно реализовать tap по ячейке, press feedback, swipe между днями и scroll-aware screens.

---

## Этап 3. Layout system для Smuzy

### Шаг 3.1. Добавить `ZStack`

Что сделать:

1. Реализовать container layout с наложением children.
2. Поддержать alignment внутри overlay.
3. Учесть overlay use cases Smuzy:
   - current block поверх grid;
   - invisible tap layer;
   - layered splash.

### Шаг 3.2. Добавить `Grid`

Что сделать:

1. Ввести grid container с фиксированным количеством колонок/строк или явным track description.
2. Поддержать dense static grid сценарий для day blocks.
3. Упростить первую версию под Smuzy: не нужна полноценная CSS-grid модель.

Решения:

1. Для v1 достаточно row/column positioning без span/reflow complexity.
2. Сначала покрыть fixed grid use case 8x9.

### Шаг 3.3. Добавить `Wrap`

Что сделать:

1. Реализовать flow/wrap layout для chips.
2. Поддержать horizontal и vertical spacing.
3. Поддержать alignment внутри строки.

### Шаг 3.4. Добавить `ScrollView`

Что сделать:

1. Ввести вертикальный scroll container как минимум для iOS.
2. Добавить content layout внутри scroll area.
3. Прокинуть scroll events обратно в runtime.

Результат этапа 3:

1. можно собрать `DayGrid`, `RoutinesList`, `Settings`, `CalendarScreen`.

---

## Этап 4. Higher-level controls

### Шаг 4.1. Добавить `Icon`

Что сделать:

1. Ввести widget для системных icon names.
2. На iOS маппить его на SF Symbols.
3. Поддержать tint/color/size.

### Шаг 4.2. Добавить form-oriented primitives

Что сделать:

1. Ввести `Form`.
2. Ввести `Section`.
3. Выровнять их с iOS-first baseline.

### Шаг 4.3. Добавить `DatePicker`

Что сделать:

1. Ввести виджет выбора даты.
2. Поддержать controlled value и change callback.
3. Для первой версии ограничить сценарий calendar/date-only.

### Шаг 4.4. Добавить `ContextMenu`

Что сделать:

1. Ввести API для context actions.
2. Поддержать destructive/default actions.
3. Реализовать iOS bridge для menu presentation.

### Шаг 4.5. Добавить `ToastHost`

Что сделать:

1. Ввести overlay-based toast/HUD component.
2. Поддержать success/error/info варианты.
3. Поддержать timed dismissal.

Результат этапа 4:

1. routine form и settings screen можно довести до функционального parity.

---

## Этап 5. Navigation и presentation

### Шаг 5.1. Добавить `TabView`

Что сделать:

1. Ввести tab shell с controlled selected state.
2. Поддержать label/icon.
3. Реализовать iOS tab host.

### Шаг 5.2. Добавить iOS-first `NavigationStack`

Что сделать:

1. Поддержать navigation title.
2. Поддержать display mode.
3. Поддержать toolbar/header actions в минимальной форме.

### Шаг 5.3. Добавить `Sheet`

Что сделать:

1. Поддержать controlled `is_presented`.
2. Поддержать child content.
3. Поддержать dismissal callback/state sync.

Результат этапа 5:

1. tabs, modal calendar и routine form переходят под полный Rust control.

---

## Этап 6. Environment layer

### Шаг 6.1. Добавить системные environment values

Что сделать:

1. `colorScheme`
2. host metrics
3. safe area как публично доступный environment
4. current date/time source

### Шаг 6.2. Добавить app metadata environment

Что сделать:

1. app version;
2. build number;
3. bundle info.

### Шаг 6.3. Провести environment через runtime и iOS bridge

Что сделать:

1. при platform changes обновлять environment и запрашивать repaint;
2. исключить дублирование таких данных в app-specific glue.

Результат этапа 6:

1. Smuzy сможет корректно стилизоваться под dark/light mode и показывать версию приложения.

---

## Этап 7. Platform services

### Шаг 7.1. Ввести service/effect channel

Что сделать:

1. Создать отдельный framework service API вне mutation protocol.
2. Поддержать request/result model.
3. Сделать типизированные responses обратно в runtime.

### Шаг 7.2. Реализовать file/share services

Что сделать:

1. export/share file;
2. import file/document picker;
3. typed result callbacks.

### Шаг 7.3. Реализовать haptics service

Что сделать:

1. light/medium/success/error baseline;
2. декларативный trigger path из Rust UI/state.

Результат этапа 7:

1. backup/restore и tactile feedback можно реализовать без отдельного SwiftUI слоя.

---

## Этап 8. Persistence layer

### Шаг 8.1. Ввести reusable local storage surface

Что сделать:

1. добавить storage crate или модуль;
2. оформить local persistence API как reusable service;
3. обеспечить reactive updates.

### Шаг 8.2. Реализовать Smuzy-oriented repositories

Что сделать:

1. `RoutineRepository`;
2. `BlockRepository`;
3. query by date range/day;
4. insert/update/delete APIs.

### Шаг 8.3. Реализовать backup encode/decode

Что сделать:

1. JSON export format;
2. JSON import/restore flow;
3. восстановление с полным refresh reactive data.

Решения:

1. Не копировать SwiftData annotations/model layer.
2. Public API хранить в Rust-терминах repositories/queries/signals.

Результат этапа 8:

1. данные Smuzy полностью живут в Rust.

---

## Этап 9. Animation и UX parity

### Шаг 9.1. Добавить animation primitives

Что сделать:

1. implicit animation на prop change;
2. transition на mount/unmount;
3. spring helper;
4. delayed action helper.

### Шаг 9.2. Добавить timeline/keyframe baseline

Что сделать:

1. поддержать keyframe-like animation для block pulse;
2. поддержать однократный trigger-based animation flow.

Результат этапа 9:

1. splash и day block animation переходят в framework-supported path.

---

## Этап 10. Runtime correctness и layout quality

### Шаг 10.1. Добавить keyed identity

Что сделать:

1. расширить public list/repeat API ключами;
2. научить VDOM сохранять identity по key, а не только по позиции.

### Шаг 10.2. Добавить снятие props без полного remount

Что сделать:

1. убрать зависимость от `ReplaceNode` при исчезновении visual/layout props;
2. поддержать unset path в diff/backend.

### Шаг 10.3. Добавить native text measurement

Что сделать:

1. подключить measurement bridge для текста;
2. по возможности для input/button text content;
3. откалибровать `Wrap`/`Grid`/chip widths под UIKit.

Результат этапа 10:

1. перенос становится устойчивым для форм, списков и плотных layout-сценариев.

---

## Этап 11. Acceptance target внутри workspace

### Шаг 11.1. Добавить Smuzy-focused demo/example

Что сделать:

1. создать demo внутри `examples/` или отдельный integration target;
2. покрыть:
   - splash;
   - tabs;
   - day grid;
   - routine chips;
   - routine form;
   - settings services.

### Шаг 11.2. Использовать demo как framework acceptance suite

Что сделать:

1. golden snapshots для VDOM/layout;
2. iOS smoke scenarios для interaction/presentation/services;
3. регрессии на focus retention и keyed updates.

Результат этапа 11:

1. развитие фреймворка проверяется не только synthetic examples, но и продуктовым сценарием, близким к реальному приложению.

---

## 5. Изменения по подсистемам

## `mf_widgets`

Добавить:

1. `Container`
2. `Pressable`
3. `ZStack`
4. `Grid`
5. `Wrap`
6. `ScrollView`
7. `Form`
8. `Section`
9. `Icon`
10. `TabView`
11. `NavigationStack`
12. `Sheet`
13. `DatePicker`
14. `ToastHost`

## `native_schema`

Расширить:

1. `ElementKind`
2. `PropKey`
3. `PropValue`
4. `EventKind`
5. `UiEvent`

## `vdom_runtime`

Добавить:

1. canonicalization новых widgets;
2. diff support для new props/events;
3. keyed identity;
4. unset-prop path;
5. overlay/layout semantics.

## `backend_native`

Добавить:

1. UIKit mapping для `Container` и новых widgets;
2. gestures;
3. navigation/sheet executors;
4. date picker/context menu/toast/icon support;
5. platform services bridge;
6. environment updates;
7. text measurement bridge.

## `mf_runtime`

Добавить:

1. dispatch новых UI events;
2. environment propagation;
3. service/effect request-result loop;
4. animation/timeline orchestration hooks.

---

## 6. План тестирования

## Базовые проверки на каждом этапе

1. `cargo check --workspace`
2. `cargo test --workspace`

## Unit tests

Добавить тесты на:

1. widget-to-schema mapping;
2. visual props `Container`;
3. event dispatch для `Pressable`;
4. layout props и frame semantics;
5. repositories/services behavior.

## Snapshot tests

Добавить golden snapshots для:

1. `Container` trees;
2. `ZStack` overlays;
3. `Grid`/`Wrap`;
4. tabs/navigation/sheet mount/update paths;
5. keyed list updates.

## Integration tests

Добавить сценарии:

1. tap/press/drag roundtrip;
2. modal open/close;
3. tab switching;
4. date picker update;
5. share/import result flow;
6. storage update -> reactive repaint.

## iOS smoke tests

Проверять:

1. splash render path;
2. current block overlay;
3. swipe between days;
4. routine form autofocus;
5. restore backup with toast;
6. dark/light mode repaint.

---

## 7. Критический путь

Последовательность без скрытых решений:

1. Этап 1 — `Container` и visual modifiers.
2. Этап 2 — `Pressable` и gesture/event foundation.
3. Этап 3 — `ZStack`/`Grid`/`Wrap`/`ScrollView`.
4. Этап 4 — higher-level controls.
5. Этап 5 — tabs/navigation/sheet.
6. Этап 6 — environment.
7. Этап 7 — platform services.
8. Этап 8 — persistence.
9. Этап 9 — animation.
10. Этап 10 — correctness and measurement.
11. Этап 11 — Smuzy-focused acceptance demo.

Почему именно так:

1. navigation и forms не должны строиться до visual/layout foundation;
2. services и persistence бессмысленно встраивать до stabilizing UI surface;
3. animation и correctness hardening идут после того, как базовый framework surface уже выразителен.

---

## 8. Критерии завершения

План считается реализованным, когда внутри `rust_native` можно собрать Smuzy-like приложение со следующими возможностями:

1. animated splash;
2. tab shell;
3. modal calendar picker;
4. interactive day grid;
5. routine chips в wrap layout;
6. routine form с focus management;
7. settings screen с share/import/restore;
8. toast и haptic feedback;
9. dark/light theming;
10. Rust-owned persistence и reactive queries.

И при этом:

1. нет обязательного SwiftUI UI слоя поверх Rust tree;
2. `cargo check --workspace` и `cargo test --workspace` остаются зелеными;
3. есть acceptance demo и regression coverage для основных сценариев.

