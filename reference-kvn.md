# Reference: kvn как пример применения принципов

Конкретная реализация принципов из [principles.md](principles.md) в TUI `kvn`
(Rust + ratatui + crossterm). Читать вместе с принципами: здесь показано, как
«общее» воплощено в конкретном коде. Исходники: `~/kvn/src/ui/*`.

## Раскладка (принцип 2)

Верхнеуровневый вертикальный сплит (`draw_impl`, `layout.rs`):

```
[Length(3)]  панель трафика (рисуется только когда VPN подключён)
[Min(0)]     основной контент — горизонтальный сплит 50/50: Profiles | Logs
[Length(1)]  статус-бар
```

- Порог двух панелей: `TWO_PANE_MIN_WIDTH = 90` — уже Logs скрывается, Profiles
  на всю ширину.
- Минимальный размер `70×15` → иначе «terminal too small».
- Рамки панелей `Borders::ALL` + `.title(" Profiles ")`; фокус панели — `accent()`,
  неактивная — `border()`.

## Цвет/темы (принцип 3)

- `Palette { accent, cursor, foreground, background, selection_fg, selection_bg, ansi[16] }`
  (`palette.rs`). 22 темы (`themes/*.toml`, Omarchy-схема) компилируются в статику
  через `build.rs`; фолбэк `legacy` (cyan/gray/black, named ANSI).
- `Theme` (`styles.rs`) — семантические методы: `accent()`, `normal()`, `muted()`,
  `error()`, `success()`, `warning()`, `border()`, `selected()`,
  `selected_connected()`, `connected_badge()`, `status_bar()`, `popup_bg()`,
  `background()`.
- Производные цвета: `mix(a, b, t)` — линейная интерполяция в sRGB. Примеры:
  `selected_connected().bg = mix(green, background, 0.75)`,
  `status_bar().bg = mix(foreground, background, 0.92)`.
- Фон кадра заливается первым (`Block` со стилем `background()`), виджеты ставят
  только `fg`.
- Контраст — тестом: luminosа fg/bg ≥ 3.0 у всех тем.

## Виджеты (принцип 4)

- **StatusBar** (`widgets.rs`): слева бейдж ` CONNECTED `/` CONNECTING `/
  ` DISCONNECTED ` (жирный + цветной фон) + имя активного профиля (обрезанное);
  справа бейджи `KS`, `Auto`, вид DNS, режим роутинга, свежесть rule-set —
  с **приоритетным обрезанием** (KS держится первым).
- **Панель трафика**: колонки фикс. ширины (скорость 8, суммарно 6, соединений 6),
  1024-базовые единицы, переполнение `>9 EB/s`.
- **Строка профиля** (`profile_line`): префикс дерева `├ `/`└ `; колонки
  `имя` (гибкая) · `протокол` (6) · `address:port` (11–21, глобально выравнено) ·
  `latency` (7, `…` при тесте). Состояния: `selected` / `success` (подключён) /
  `selected_connected` (выбран+подключён) / `normal`.
- **Toast** (`widgets.rs`): `Clear` + рамка, info=акцент / error=красный, wrap,
  справа сверху, не рисуется при <12×5.

## Взаимодействие (принцип 5)

- Фокус панелей `MainPaneFocus::Sources|Logs`, переключение `Ctrl+h`/`Ctrl+l`
  (или `←`/`→`); выбор `j`/`k`, `gg`/`G`.
- Оверлеи (`Overlay`): Help/ConfirmDelete/RoutingMode/GeoRegions/DnsSettings/
  ThemeSettings/ServiceRouting/Support/Migration — центрированные, `Enter`/`h`/`l`/
  `q`/`Esc`.
- Выход: `q`/`Esc` отсоединяет TUI (демон и VPN живут), `Ctrl+C` — полный выход.
- Справка `?` — таблица клавиш (`help.rs`).

## Как это соотносится с workflow

| Шаг workflow | В kvn |
|---|---|
| сущность | профиль (и подписка) — список Profiles |
| действия | `Enter` connect, `t`/`T` test, `u` update, `d` delete, `e` edit, `p` paste |
| глобальное состояние | бейдж CONNECTED/… + бейджи настроек в статус-баре |
| вторичная информация | панель Logs (поток sing-box/app-логов) |
| настройки/подтверждения | модалки (роутинг, DNS, тема, geo, confirm-delete) |
| фоновая работа | `…` в колонке latency при тесте + тост по завершении |
| палитра/роли | `Theme` + 22 палитры + `mix()` |
| крайние случаи | min-size 70×15, обрезка по видимой ширине, фолбэк-тема |
