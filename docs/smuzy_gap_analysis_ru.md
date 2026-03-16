# Gap Analysis: что нужно добавить в `rust_native`, чтобы переписать `smuzy_swift`

## 1. Цель документа

Этот документ фиксирует, чего **не хватает именно во фреймворке `rust_native`**, чтобы переписать приложение `smuzy_swift` без критической зависимости от ad-hoc Swift glue.

Целевой сценарий:

1. iOS-first.
2. Близкий UX-паритет с текущим `smuzy_swift`.
3. Основные возможности должны жить внутри `rust_native` как reusable API.
4. App-specific код Smuzy должен в основном описывать продуктовую логику, а не компенсировать пробелы фреймворка.

---

## 2. Текущее состояние

### Что уже есть в `rust_native`

На момент анализа workspace находится в рабочем состоянии:

1. `cargo check --workspace` проходит.
2. `cargo test --workspace` проходит.

Фреймворк уже покрывает базовую платформу для переноса:

1. Rust-owned VDOM runtime, diff и layout pipeline.
2. Mutation protocol и native executor.
3. Signals/state model.
4. Базовый iOS host bridge.
5. Базовые виджеты:
   - `Text`
   - `Button`
   - `Input`
   - `Image`
   - `VStack`
   - `HStack`
   - `List`
   - `SafeArea`
6. Базовый event roundtrip:
   - tap для `Button`
   - text input
   - focus change
7. Asset image loading по имени.

### Чего этого уже хватает

Этого достаточно для:

1. счетчиков;
2. простых форм;
3. статических списков;
4. базовой iOS-демонстрации без сложной навигации и системных сервисов.

### Почему этого недостаточно для Smuzy

`smuzy_swift` опирается не только на базовые widgets, а на набор возможностей более высокого уровня:

1. layered layout;
2. wrap/grid layout;
3. gestures;
4. modal presentation;
5. tabs/navigation shell;
6. animation;
7. haptics;
8. file import/export/share;
9. persistence и reactive queries;
10. системные визуальные и environment-сигналы.

Именно этот слой сейчас в `rust_native` либо отсутствует, либо покрыт слишком узко.

---

## 3. Что именно использует `smuzy_swift`

### Экраны и сценарии

`smuzy_swift` состоит из следующих ключевых частей:

1. Splash screen с анимацией текста и изображения.
2. Tab shell:
   - day screen
   - settings screen
3. Day screen:
   - заголовок с датой;
   - modal calendar picker;
   - day grid;
   - переключение дня свайпом;
   - current block overlay;
   - interactive routine selection.
4. Routine list:
   - wrap layout для чипов;
   - add/edit/delete routine;
   - context menu;
   - modal form.
5. Routine form:
   - text input;
   - focus management;
   - palette picker;
   - disabled/save states.
6. Settings:
   - backup/share;
   - file import;
   - restore flow;
   - toast feedback;
   - app version info;
   - декоративный background image.

### Архитектурные зависимости `smuzy_swift`

Приложение зависит от нескольких классов capability:

1. UI composition и visual styling.
2. Gesture/event handling.
3. Presentation/navigation.
4. Persistence/data queries.
5. Platform services.
6. Animation/feedback.

---

## 4. Матрица покрытия: что уже есть, а чего нет

| Область | Нужно для Smuzy | Есть сейчас | Статус |
|---|---|---:|---|
| Базовый текст/кнопки/input/image | Да | Да | Достаточно для базы |
| Stack layout | Да | Да | Частично достаточно |
| Safe area | Да | Да | Достаточно |
| Overlay/layered layout | Да | Нет | Блокер |
| Grid layout | Да | Нет | Блокер |
| Wrap/flow layout | Да | Нет | Блокер |
| Scroll container | Да | Нет отдельного API | Блокер |
| Tabs | Да | Нет | Блокер |
| Navigation shell/title/toolbar | Да | Нет | Блокер |
| Modal sheet | Да | Нет | Блокер |
| Date picker | Да | Нет | Блокер |
| Context menu | Да | Нет | Блокер |
| Toast/HUD | Да | Нет | Блокер |
| Drag gesture | Да | Нет | Блокер |
| Generic tap/press handlers на любом view | Да | Нет | Блокер |
| Haptics | Да | Нет | Блокер |
| Animation/transitions | Да | Нет | Блокер |
| Persistence layer | Да | Нет | Блокер |
| File import/export/share | Да | Нет | Блокер |
| App environment values | Да | Частично | Недостаточно |
| Keyed reconciliation | Желательно | Нет | Высокий риск |
| Accurate native text measurement | Желательно | Нет | Высокий риск |

---

## 5. Ключевые пробелы фреймворка

## 5.1. Базовый визуальный примитив: нужен `Container`

Отдельные `Rectangle`, `Circle`, `RoundedRectangle` как разные widgets здесь не нужны. Для задач Smuzy лучше единый базовый визуальный контейнер.

### Что должен уметь `Container`

1. Быть пустым или содержать child.
2. Иметь explicit layout:
   - `width`
   - `height`
   - `min_width`
   - `min_height`
   - `max_width`
   - `max_height`
3. Иметь визуальные свойства:
   - `background`
   - `opacity`
   - `shadow`
   - `border`
   - `stroke`
   - `clip`
4. Иметь corner model:
   - `corner_radius`
   - `corner_radius_per_corner`
   - `full_round`
5. Иметь positioning/modifier API:
   - `padding`
   - `offset`
   - `overlay`
   - `background`

### Почему это блокер

Без такого `Container` не собрать:

1. ячейки day grid;
2. current block frame;
3. color circles;
4. routine chips;
5. outlined add button;
6. splash/image layers;
7. bordered panels и background decorations.

### Что нужно изменить в архитектуре

1. Добавить `ElementKind::Container`.
2. Добавить props для:
   - border/stroke;
   - shadow;
   - per-corner radius;
   - full-round mode;
   - offset;
   - overlay/background layers или отдельные контейнеры поверх `ZStack`.
3. Реализовать UIKit mapping для `UIView`/`CALayer`.
4. Уточнить layout semantics для пустого и content-bearing `Container`.

---

## 5.2. Layout API: текущих `VStack/HStack/List` недостаточно

Для Smuzy нужны дополнительные layout primitives.

### Обязательные additions

1. `ZStack`
   - для overlay current block;
   - для layered splash;
   - для tap-layer поверх day cells.
2. `Grid`
   - для day grid 8x9;
   - для отдельного time column + content grid.
3. `Wrap` или `Flow`
   - для routine chips;
   - для color palette в форме.
4. `ScrollView`
   - для settings;
   - для routines list;
   - для modal calendar screen.
5. `Form` и `Section`
   - как минимум iOS-oriented baseline.

### Нужные layout modifiers

1. per-edge `padding`;
2. frame constraints;
3. fill/stretch behavior как публичный API, а не скрытая schema capability;
4. alignment внутри overlay и grid;
5. clipping.

### Почему это блокер

Сейчас `smuzy_swift` активно использует композицию, которую нельзя выразить только через `VStack/HStack/List`.

---

## 5.3. Интерактивность: event surface слишком узкая

Сейчас интерактивность практически ограничена `Button` и `Input`.

### Для Smuzy нужны

1. `on_tap` на любом view.
2. `on_press` / `on_press_change`.
3. `on_long_press`.
4. `on_drag` с payload:
   - translation;
   - velocity или predicted end translation.
5. `on_appear` / `on_disappear` как публичный API.
6. scroll event API.

### Почему это блокер

Без этого нельзя корректно реализовать:

1. tap по ячейкам day grid;
2. press animation для color picker;
3. свайп между днями;
4. реакции на presentation lifecycle;
5. будущую жестовую навигацию внутри календаря.

### Изменения в протоколе

Нужно расширить:

1. `EventKind`
2. `UiEvent`
3. event binding в `backend_native`
4. runtime dispatch

---

## 5.4. Нужен `Pressable` или `Button` с произвольным content

Текущий `Button` принимает только строковый label. Для Smuzy этого недостаточно.

### Где это ломается

1. `RoutineButtonView` содержит circle + text.
2. `AddRoutineButton` содержит icon + text + outline.
3. Заголовок даты содержит text + chevron.
4. Color picker button визуально вообще не похож на обычный button.

### Что нужно

Один из двух путей:

1. `Button` с child content builder.
2. Отдельный `Pressable`/`Interactive` wrapper, который можно обернуть вокруг любого subtree.

Рекомендуемый путь для `rust_native`: добавить `Pressable`, а `Button` оставить как удобный sugar поверх него.

---

## 5.5. Presentation и navigation слой отсутствует

`smuzy_swift` использует не просто layout, а системную структуру экранов.

### Нужные capabilities

1. `TabView`
2. `NavigationStack` или iOS-first `NavigationView`
3. `Sheet`
4. navigation title / display mode / toolbar actions
5. context menu

### Где это нужно в Smuzy

1. Главный tab shell.
2. Открытие calendar screen.
3. Открытие routine form.
4. Навигационные заголовки в calendar/settings/form.
5. Context menu на routine chips.

### Почему app-specific Swift glue недостаточен

Если эти механизмы не встроены во фреймворк:

1. исчезает единый Rust-owned UI model;
2. presentation state живет в Swift;
3. теряется portability и переиспользуемость;
4. Smuzy перестает быть честной проверкой возможностей `rust_native`.

---

## 5.6. Animation/runtime эффектов пока нет

Smuzy заметно опирается на легкий, но системный motion layer.

### Что нужно

1. implicit animations на prop changes;
2. transitions для mount/unmount;
3. spring animations;
4. keyframe/timeline API;
5. delayed actions, которые не выглядят как ad-hoc `DispatchQueue` в приложении;
6. animated state helpers;
7. optional haptic trigger hooks.

### Где это используется

1. splash screen;
2. pulse/shake animation в `DayBlockAnimation`;
3. раскрытие/закрытие состояний;
4. плавное обновление selected date/current block.

### Почему это важно

Без animation layer перенос будет формально рабочим, но UX будет заметно хуже, чем в SwiftUI-версии.

---

## 5.7. Platform services: share/import/export/haptics отсутствуют

Для Smuzy это обязательная часть, а не nice-to-have.

### Нужные сервисы

1. share/export JSON file;
2. file import/document picker;
3. callbacks с результатом операции;
4. haptics;
5. app version/build info;
6. system symbols или icon service.

### Почему это должно быть частью framework surface

Эти сервисы нужны не только Smuzy. Это типичный слой для mobile apps:

1. экспорт данных;
2. импорт из файла;
3. share flows;
4. tactile feedback;
5. runtime environment.

### Рекомендуемая архитектура

Не смешивать это с mutation/layout pipeline. Лучше отдельный service/effect channel:

1. Rust UI инициирует effect.
2. platform adapter исполняет его.
3. результат возвращается обратно в runtime как typed event/result.

---

## 5.8. Persistence/data layer отсутствует

`smuzy_swift` использует `SwiftData` не как детали платформы, а как основу приложения:

1. хранение `Routine`;
2. хранение `Block`;
3. выборки по дню;
4. live query updates;
5. save/delete/restore flows.

### Что не обязательно повторять

Не нужно клонировать `SwiftData` API.

### Что обязательно нужно

Нужен reusable Rust-side storage/service layer:

1. локальное хранение сущностей;
2. reactive queries или подписки;
3. CRUD API;
4. сериализация/десериализация backup;
5. простой iOS storage backend.

### Минимальный scope

Для переноса Smuzy достаточно:

1. `RoutineRepository`
2. `BlockRepository`
3. query by date range
4. export/import backup service

Но это должно быть оформлено как reusable framework module, а не захардкожено в приложении.

---

## 5.9. Environment model пока слишком бедная

`smuzy_swift` читает системное состояние и UI context.

### Нужно добавить

1. `colorScheme`
2. host metrics как environment, не только как hidden layout input
3. current date/time service
4. bundle/app metadata
5. presentation context

### Где это нужно

1. dark/light mode styling;
2. current block/highlight behavior;
3. version label в settings;
4. общая адаптация UI под систему.

---

## 5.10. VDOM identity model слишком грубая для реального app UI

Сейчас diff в `rust_native` позиционный. Также исчезновение props может приводить к `ReplaceNode`.

### Риски для Smuzy

1. потеря focus state;
2. лишние remount при изменении routine lists;
3. лишние побочные эффекты на animated элементах;
4. хрупкость modal/form flows.

### Что нужно

1. keyed identity для списков и повторяющихся children;
2. более точный diff без избыточного `ReplaceNode`;
3. возможность убрать prop без полного remount узла.

Это не абсолютный стартовый блокер для первого скелета Smuzy, но это высокий риск для стабильного переноса.

---

## 5.11. Layout measurement пока эвристический

Smuzy много работает с:

1. плотной сеткой;
2. текстовыми чипами;
3. wrap layout;
4. overlay geometry.

Текущий layout движок измеряет текст эвристически.

### Риск

На реальном UIKit layout:

1. чипы могут переполняться;
2. тексты будут занимать не тот размер;
3. wrap/grid геометрия может расходиться с ожиданием.

### Что нужно

Добавить native measurement path для текста и, по возможности, для input/button content.

---

## 6. Что нужно добавить в публичный API

## 6.1. Widgets

Минимально необходимый набор:

1. `Container`
2. `ZStack`
3. `Grid`
4. `Wrap`
5. `ScrollView`
6. `Form`
7. `Section`
8. `Pressable`
9. `TabView`
10. `NavigationStack`
11. `Sheet`
12. `DatePicker`
13. `ToastHost`
14. `Icon`

## 6.2. Modifiers и props

Нужны публичные builders/modifiers для:

1. `padding(EdgeInsets)`
2. `frame(...)`
3. `offset(...)`
4. `background(...)`
5. `overlay(...)`
6. `opacity(...)`
7. `shadow(...)`
8. `border(...)`
9. `corner_radius(...)`
10. `corner_radius_per_corner(...)`
11. `full_round(...)`
12. `clip(...)`
13. `disabled(...)`

## 6.3. Events

Нужны публичные event APIs:

1. `on_tap`
2. `on_press`
3. `on_press_change`
4. `on_long_press`
5. `on_drag`
6. `on_scroll`
7. `on_appear`
8. `on_disappear`

## 6.4. Services/environment

Нужны reusable entry points для:

1. storage
2. share/export
3. file import
4. haptics
5. app metadata
6. color scheme
7. current time/date

---

## 7. Приоритизированный roadmap

## P0: без этого перенос Smuzy не стартует

1. `Container` как базовый визуальный примитив.
2. `Pressable` или content-based `Button`.
3. `ZStack`.
4. `Grid`.
5. `Wrap`.
6. `ScrollView`.
7. `TabView`.
8. `NavigationStack`/`NavigationView`.
9. `Sheet`.
10. Storage/service layer для `Routine` и `Block`.
11. Share/export/import service.
12. Generic modifiers: frame, offset, overlay, border, corner model.

## P1: без этого не будет близкого UX-паритета

1. Drag/press gestures.
2. Context menu.
3. Date picker.
4. Toast/HUD.
5. Haptics.
6. Color scheme environment.
7. App metadata environment.
8. Animation/transitions.
9. Icon/SF Symbol support.

## P2: стабилизация и production-grade перенос

1. Keyed diff.
2. Prop removal без `ReplaceNode`.
3. Native text measurement.
4. Layout parity hardening.
5. Perf tuning для dense grid/wrap screens.
6. Regression suite на presentation + gestures + storage services.

---

## 8. Acceptance criteria для готовности фреймворка под Smuzy

Считать `rust_native` достаточно готовым для переписывания Smuzy, когда можно реализовать следующие сценарии без ad-hoc Swift UI логики:

1. Splash screen с анимированным text/image.
2. Tab shell с day/settings вкладками.
3. Открытие calendar modal sheet.
4. Day grid:
   - layered rendering;
   - tap по ячейке;
   - swipe между днями;
   - current block overlay.
5. Routine chips в wrap layout.
6. Modal routine form с autofocus.
7. Color picker с press feedback.
8. Edit/delete через context actions.
9. Backup export через share sheet.
10. Restore из JSON через file picker.
11. Toast после restore.
12. Темизация под light/dark mode.

---

## 9. Итоговый вывод

Основная проблема не в том, что `rust_native` не умеет рендерить UI вообще. Базовый runtime, diff, layout и iOS executor уже существуют и находятся в хорошем состоянии.

Пробел между текущим состоянием фреймворка и `smuzy_swift` лежит в другом слое:

1. composable visual primitives;
2. rich layout;
3. generic interaction model;
4. presentation/navigation;
5. animation;
6. platform services;
7. persistence.

Если смотреть прагматично, то для реального переноса Smuzy `rust_native` нужно развивать не столько вглубь VDOM core, сколько вширь, до уровня полноценного app framework.

Ключевое архитектурное решение для этого этапа:

1. принять `Container` как универсальный базовый визуальный primitive вместо отдельного набора shape widgets;
2. строить поверх него interaction, overlay и presentation API;
3. добавить reusable service layer для storage и platform integrations.

Именно после этого `smuzy_swift` станет реалистичным target-приложением для переноса на `rust_native`.
