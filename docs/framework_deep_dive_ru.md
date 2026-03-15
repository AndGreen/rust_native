# Подробное устройство фреймворка

## Общая модель

Это уже не просто набор Rust-виджетов, а полноценный UI pipeline с четким разделением слоев:

1. декларативный UI описывается в Rust;
2. из него строится внутреннее дерево `View`;
3. это дерево переводится в каноническое VDOM-представление;
4. VDOM вычисляет изменения как `Mutation[]`;
5. layout считается на стороне Rust как `LayoutFrame[]`;
6. native backend только исполняет готовые команды;
7. события от платформы возвращаются обратно в Rust.

Ключевая идея проекта: `Rust owns truth`. Источник истины для дерева, состояния, layout и event routing находится в Rust, а iOS/Android выступают как исполнители mutation/layout протокола.

Основные слои проекта:

1. `mf_widgets`: декларативные примитивы UI и builders.
2. `mf_macros`: `ui!` DSL.
3. `mf_core`: базовый `View`, DSL traits, signals.
4. `vdom_runtime`: реальный VDOM runtime, diff и layout.
5. `native_schema`: канонический словарь элементов, props, events и mutations.
6. `backend_native`: generic executor и platform adapters для iOS/Android.
7. `mf_runtime`: scheduler, repaint loop, signal subscriptions.
8. `dev_protocol` / `dev_support` / `dev_cli`: dev server, worker и hot reload путь.

## Базовое внутреннее представление

Самый низкий общий слой UI здесь очень маленький. В `mf_core` есть `View`, который хранит:

1. erased widget element;
2. список дочерних `View`.

Это намеренно простая структура. Она не знает ни про UIKit, ни про Android, ни про mutation protocol. Это просто универсальное дерево, на котором потом строятся остальные этапы.

Еще один важный элемент на этом уровне это `Fragment`. Он используется как синтаксическая обертка, когда нужно вернуть несколько дочерних узлов без собственного визуального контейнера.

Отдельно стоит отметить, что в `mf_core` есть старый `DiffEngine`, но он уже не является сердцем архитектуры. Настоящий reconciliation сейчас живет в `vdom_runtime`.

## DSL и пользовательский API

Пользовательский слой у фреймворка двухуровневый:

1. можно писать интерфейс обычными Rust builders из `mf_widgets`;
2. можно писать через `ui!`, который дает SwiftUI-подобный синтаксис.

Важно, что `ui!` не вводит отдельный runtime-язык. Это compile-time sugar. Макрос парсит синтаксис и разворачивает его в обычные Rust-вызовы builders, `IntoView` и `WithChildren`.

То есть фреймворк не интерпретирует DSL в рантайме. После компиляции остаются обычные Rust-конструкции.

Практический смысл такого решения:

1. нет отдельного интерпретатора DSL;
2. типизация остается на стороне Rust;
3. `ui!` и ручные builders в архитектуре эквивалентны;
4. runtime вообще не знает, каким синтаксисом был написан UI.

## Виджеты и декларативный слой

`mf_widgets` задает публичную поверхность фреймворка.

Контейнеры вроде `VStack`, `HStack` и `SafeArea` задают структуру и layout-намерение:

1. axis;
2. spacing;
3. padding;
4. alignment;
5. justify-content;
6. background.

Leaf widgets вроде `Text`, `Button`, `Input`, `Image` хранят свои данные и визуальные свойства. При этом `Button` и `Input` уже на этом уровне содержат Rust-обработчики событий:

1. `Button` хранит `on_click`;
2. `Input` хранит `on_input`;
3. `Input` также хранит `on_focus_change`.

Это важно для всей архитектуры: event handlers не живут где-то отдельно в bridge-слое, а являются частью декларативного Rust UI и позже переносятся в таблицы обработчиков внутри runtime.

`List` на текущем этапе является довольно тонкой оберткой вокруг коллекции дочерних `View`. Это еще не виртуализированный список и не keyed list runtime в полноценном смысле.

## Реактивность и состояние

Реактивность сделана в стиле Solid:

1. есть `Signal<T>` для чтения;
2. есть `Setter<T>` для записи;
3. есть `batch_updates(...)` для coalescing уведомлений;
4. есть механизм отслеживания того, какие сигналы были прочитаны во время рендера.

Когда render closure вызывает `count.get()` или другой `signal.get()`, рантайм запоминает, что этот signal был использован. После завершения рендера `App` подписывается именно на те сигналы, которые реально читались в этом кадре.

Это дает довольно чистую модель:

1. builder closure описывает UI;
2. чтения сигналов автоматически определяют зависимости;
3. при обновлении signal приложение просто помечается как `dirty`;
4. следующий `tick()` строит новый кадр.

Важно, что это не тяжелый глобальный store и не React-подобный hook runtime. Механика достаточно прямолинейная и локальная.

## Роль `App` и жизненный цикл кадра

`mf_runtime::App` это главный orchestration-объект.

Он связывает:

1. backend;
2. builder closure;
3. `VdomRuntime`;
4. signal subscriptions;
5. repaint loop.

Один кадр устроен так:

1. `App` вызывает builder внутри механизма отслеживания signal reads;
2. получает `View`;
3. передает его в `VdomRuntime`;
4. получает `RenderBatch`, то есть `mutations + layout`;
5. отдает batch в backend;
6. вызывает `flush`;
7. после этого забирает события из backend;
8. отдает их обратно в `VdomRuntime`, который вызывает Rust handlers.

Если backend по какой-то причине отвергает batch, `App` не падает сразу. Он просит `VdomRuntime` сделать полный resync и повторяет кадр. Это важный recovery path на случай рассинхронизации между Rust runtime и native registry.

С точки зрения scheduling сейчас модель довольно простая: `tick()` вызывается в цикле с шагом примерно 16 мс. Это polling-based loop, а не сложный платформенный event scheduler.

## Настоящий VDOM runtime

Настоящий reconciliation живет в `crates/vdom_runtime`.

Именно этот слой:

1. превращает `View` в каноническое дерево;
2. раздает `UiNodeId`;
3. хранит текущую версию дерева;
4. сравнивает прошлое и новое состояние;
5. выпускает mutation batch;
6. считает layout;
7. хранит таблицы обработчиков событий.

Это уже полноценный VDOM-first слой, а не просто сериализация дерева виджетов.

### Канонический узел

После `View` дерево переводится в `CanonicalNode`.

В этом представлении у каждого узла уже есть:

1. стабильный на время жизни узла `UiNodeId`;
2. канонический `ElementKind`;
3. набор schema-level props;
4. текстовое значение, если оно есть;
5. ссылки на обработчики событий;
6. список канонических детей.

На этой границе runtime избавляется от widget-specific формы и переходит к единому словарю, понятному любому backend.

### Identity model

Идентичность узлов сейчас позиционная и довольно простая.

Если на той же позиции в дереве сохранился тот же тип узла, runtime старается переиспользовать `UiNodeId`. Если тип поменялся, выдается новый id.

Это значит:

1. keyed reconciliation пока нет;
2. reorder детей как отдельный класс обновлений пока не выделяется;
3. список без keys ведет себя как positional diff.

Хотя в протоколе уже есть `MoveNode`, текущая реализация его еще не генерирует.

## Diff и поток мутаций

После каноникализации `vdom_runtime` вычисляет разницу между старым и новым деревом.

На текущем этапе стратегия сравнительно простая:

1. если узел принципиально изменился, runtime делает `ReplaceNode`;
2. если поменялся только текст, выпускается `SetText`;
3. если изменились props, выпускаются `SetProp`;
4. общая часть children сравнивается рекурсивно;
5. новые дети домонтируются;
6. лишние старые дети удаляются.

Важная деталь: удаление свойства сейчас не кодируется как отдельный `UnsetProp`. Если у узла исчез prop, runtime предпочитает заменить весь узел через `ReplaceNode`. Это упрощает executor и backend logic, но делает diff грубее.

Архитектурно видно, что schema уже рассчитана на более богатый diff, чем тот, что реально включен сейчас.

## Канонический schema boundary

`native_schema` это одна из самых важных crate в проекте.

Она фиксирует общий словарь между Rust VDOM runtime и native backends:

1. `ElementKind`
2. `PropKey`
3. `PropValue`
4. `Mutation`
5. `UiEvent`
6. `LayoutFrame`

Это дает сильную архитектурную границу:

1. runtime не тянет за собой UIKit/JNI-типы;
2. backends не зависят от `mf_widgets`;
3. dev tools и remote mode работают через те же канонические данные;
4. mutation/layout поток становится общим ABI-подобным контрактом.

С точки зрения дизайна это один из самых зрелых участков проекта.

## Layout: Rust как единственный источник геометрии

Одна из центральных идей фреймворка: layout считается на стороне Rust.

Для этого используется `taffy`.

Pipeline такой:

1. из canonical tree извлекаются layout props;
2. строится дерево `taffy`;
3. `taffy` вычисляет размеры и позиции;
4. результат превращается в `LayoutFrame[]`;
5. backend применяет готовые frame уже после структурных mutations.

Это означает, что native side не должен самостоятельно решать layout subtree. UIKit и Android получают уже рассчитанную геометрию.

### Что это дает

1. единая модель layout для всех backends;
2. отсутствие platform-specific divergence в базовой геометрии;
3. более детерминированный render path;
4. контроль safe area и sizing внутри Rust runtime.

### Что пока упрощено

При этом layout пока не опирается на реальные native text measurement APIs.

Сейчас:

1. текст меряется эвристически;
2. button size выводится из текста и paddings;
3. input по умолчанию растягивается по ширине родителя;
4. image использует explicit size или fallback.

То есть layout уже централизован, но пока не обладает точностью зрелого production text engine.

## SafeArea

`SafeArea` здесь не является платформенным контейнером в духе "пусть UIKit сам решит". Это часть Rust-side layout model.

Host сообщает runtime свои `safe_area` insets, а `SafeArea` переводит их в padding на соответствующих edges.

Следствие:

1. safe area интегрирована прямо в layout pipeline;
2. поведение согласовано между backends;
3. subtree остается под контролем Rust, а не Auto Layout или Android measurement passes.

## Backend API и generic executor

`backend_api::Backend` сам по себе очень маленький:

1. применить mutations;
2. применить layout;
3. flush;
4. вернуть очередь событий.

Но настоящая backend-логика живет в generic executor внутри `backend_native`.

Этот executor:

1. ведет registry всех узлов;
2. знает parent/child связи;
3. знает какой `UiNodeId` соответствует какому native handle;
4. валидирует structural invariants;
5. отдельно накапливает layout frames до `flush()`.

Именно здесь enforced такие правила, как:

1. у узла не может быть двух родителей;
2. нельзя создать цикл;
3. root нельзя просто удалить;
4. layout не может ссылаться на неизвестный id.

То есть backend executor это не "тупой транзитный слой", а структурный guardian протокола.

## iOS backend

iOS backend уже не просто логирует команды. Он создает реальные UIKit views и применяет к ним batch.

Текущий mapping примерно такой:

1. контейнеры становятся `UIView`;
2. `Text` становится `UILabel`;
3. `Button` становится `UIButton`;
4. `Image` становится `UIImageView`;
5. `Input` становится `UITextField`.

Важный архитектурный момент: visual props backend применяет сам, а layout props сознательно игнорирует. Не потому что они не нужны, а потому что они уже были использованы в Rust layout engine. UIKit здесь не решает структуру, axis, spacing и геометрию subtree, а лишь materializes результат.

`LayoutFrame` применяется через `setFrame`, то есть управляемое поддерево живет в режиме manual frames.

## Android backend

Android слой архитектурно симметричен iOS, но чуть более абстрагирован.

Вместо прямой UIKit-подобной реализации там введен `AndroidBridge`, который скрывает конкретный JNI/FFI boundary. Снаружи все выглядит так же:

1. executor проверяет batch;
2. adapter переводит schema-level команды в platform bridge calls;
3. layout приходит уже готовыми frame;
4. события превращаются в `UiEvent`.

Это хороший признак архитектурной зрелости: общая логика executor не дублируется по платформам.

## Event model

События движутся обратно в Rust по той же канонической схеме.

Native layer не вызывает Rust widget closures напрямую. Вместо этого:

1. platform callback превращается в `UiEvent`;
2. событие кладется в очередь backend;
3. `App` забирает события через `drain_events()`;
4. `VdomRuntime` dispatch-ит их по таблицам обработчиков.

На практике сейчас реально используются:

1. `Tap`;
2. `TextInput`;
3. `FocusChanged`.

`Scroll`, `Appear`, `Disappear` уже есть в протоколе, но пока не стали полноценной частью основного runtime-пути.

## Dev protocol и remote mode

У проекта есть отдельная dev-архитектура, но она не ломает основную модель. Вместо этого используется тот же render pipeline с другим backend.

### Как это устроено

`dev_cli`:

1. собирает нужный example binary;
2. запускает его как worker;
3. следит за изменениями файлов;
4. пересобирает приложение;
5. ретранслирует render batches клиенту.

`dev_support::RemoteBackend`:

1. принимает mutations/layout как обычный backend;
2. вместо native rendering сериализует их в JSON lines;
3. пишет в stdout worker-процесса.

С другой стороны:

1. dev server шлет worker-команды вроде host resize, repaint, full resync и shutdown;
2. UI events также пробрасываются назад в worker;
3. worker продолжает крутить обычный `App`, только поверх remote backend.

Это очень важный момент: dev mode здесь не отдельный renderer и не фальшивая симуляция, а тот же runtime, просто с транспортным backend вместо нативного.

## Hot reload и dev server

`dev_cli` реализует довольно прямолинейный цикл hot reload:

1. следит за изменениями файлов в workspace;
2. при изменениях пересобирает target app;
3. убивает старый worker;
4. запускает новый;
5. уведомляет клиент через `Reloading` и `ResetUi`.

С технической стороны это не тонкий HMR на уровне компонентов, а process-level reload с повторным запуском worker и повторной синхронизацией UI. Но для текущей стадии проекта это вполне согласуется с общей архитектурой.

## Один полный путь кадра

Если посмотреть на пример вроде `counter`, полный путь выглядит так:

1. код на `ui!` или builders создает декларативное описание интерфейса;
2. builder closure читает signals;
3. `App` собирает зависимости через `collect_reads`;
4. строится `View`-дерево;
5. `VdomRuntime` переводит его в canonical tree;
6. runtime вычисляет mutations;
7. runtime вычисляет layout;
8. backend materializes изменения в native subtree;
9. пользовательский input возвращается как `UiEvent`;
10. runtime вызывает Rust handler;
11. handler меняет signal;
12. следующий `tick()` строит новый кадр.

Это и есть главный цикл фреймворка.

## Что в архитектуре уже выглядит сильным

Если смотреть на проект как на framework, у него уже есть несколько сильных архитектурных решений:

1. очень четкая schema boundary;
2. отдельный VDOM runtime;
3. layout полностью принадлежит Rust;
4. generic executor с проверкой инвариантов;
5. единая event model;
6. одинаковая концепция для native и remote rendering;
7. typed declarative API без reflection и JS runtime.

Видно, что проект строится не как тонкий wrapper над native UI, а как собственный Rust-native UI runtime.

## Текущие ограничения

При этом по коду видно, что проект еще на этапе активного упрощения и стабилизации.

Главные ограничения сейчас:

1. нет keyed diff;
2. нет реального `MoveNode` в reconciliation;
3. удаление props приводит к `ReplaceNode`;
4. text measurement пока эвристический;
5. нет полноценной list virtualization;
6. top-level `Fragment` фактически схлопывается к первому ребенку;
7. часть протокола уже заложена, но пока не используется в полном объеме.

То есть архитектурный каркас уже выстроен, а многие алгоритмически сложные части пока еще intentionally упрощены.

## Итог

Фреймворк уже устроен как полноценная VDOM-first Rust-native система:

1. DSL и builders создают обычный Rust UI;
2. этот UI переводится в внутреннее каноническое дерево;
3. каноническое дерево диффится в mutation batch;
4. layout считается в Rust;
5. native side применяет уже готовый результат;
6. события возвращаются обратно в Rust;
7. `App` координирует весь lifecycle кадра;
8. dev mode использует тот же pipeline, только с транспортным backend.

Если сформулировать совсем коротко, ядро проекта сегодня это не "виджеты на Rust", а именно:

1. Rust-side declarative UI;
2. Rust-side reactive runtime;
3. Rust-side VDOM reconciliation;
4. Rust-side layout engine;
5. mutation/layout protocol для платформ;
6. thin native executors.

Это и есть фактическое устройство текущего фреймворка.
