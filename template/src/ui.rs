//! Отрисовка: список + панель деталей + статус-бар + оверлей справки.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::model::{App, Focus, ServiceState, ToastKind};
use crate::theme::Theme;

pub fn draw(frame: &mut Frame, app: &mut App, theme: &Theme) {
    let area = frame.area();

    // Фон заливается один раз на весь кадр; виджеты ниже ставят только fg.
    frame.render_widget(Block::default().style(theme.background()), area);

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let (main_area, status_area) = (split[0], split[1]);

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_area);
    draw_list(frame, app, theme, panes[0]);
    draw_detail(frame, app, theme, panes[1]);
    draw_status_bar(frame, app, theme, status_area);

    if app.show_help {
        draw_help(frame, theme, area);
    }
    if let Some((text, kind)) = &app.toast {
        draw_toast(frame, theme, area, text, *kind);
    }
}

fn draw_list(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let focused = app.focus == Focus::List;
    let block = Block::default()
        .title(" Services ")
        .borders(Borders::ALL)
        .border_style(if focused { theme.accent() } else { theme.border() });

    let items: Vec<ListItem> = app
        .services
        .iter()
        .enumerate()
        .map(|(i, svc)| {
            let state_color = match svc.state {
                ServiceState::Running => Color::Green,
                ServiceState::Stopped => Color::DarkGray,
                ServiceState::Restarting => Color::Yellow,
            };
            let prefix = if i + 1 == app.services.len() { "└ " } else { "├ " };
            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(&svc.name, theme.normal()),
                Span::raw("  "),
                Span::styled(svc.state.label(), theme.badge(state_color)),
                Span::raw("  "),
                Span::styled(
                    format!("cpu {:>4.1}%  mem {:>6.0} MB  up {}", svc.cpu, svc.mem_mb, fmt_uptime(svc.uptime_s)),
                    theme.muted(),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selected())
        .highlight_symbol("");

    let mut state = ListState::default();
    state.select(Some(app.selected));

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let focused = app.focus == Focus::Detail;
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(if focused { theme.accent() } else { theme.border() });

    let mut lines: Vec<Line> = Vec::new();
    if let Some(svc) = app.services.get(app.selected) {
        let state_style = match svc.state {
            ServiceState::Running => theme.success(),
            ServiceState::Stopped => theme.muted(),
            ServiceState::Restarting => theme.warning(),
        };
        lines.push(Line::from(vec![
            Span::styled("  ", theme.normal()),
            Span::styled(&svc.name, theme.accent()),
            Span::raw("  "),
            Span::styled(svc.state.label(), state_style),
        ]));
        lines.push(Line::from(""));
        lines.push(kv("state", svc.state.label(), theme));
        lines.push(kv("cpu", &format!("{:.1}%", svc.cpu), theme));
        lines.push(kv("mem", &format!("{:.0} MB", svc.mem_mb), theme));
        lines.push(kv("uptime", &fmt_uptime(svc.uptime_s), theme));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Enter toggle · r restart · s stop · S start",
            theme.muted(),
        )));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(para, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let running = app.running_count();
    let total = app.services.len();

    let left = Line::from(vec![
        Span::styled(" ", theme.normal()),
        Span::styled(
            format!(" {running}/{total} up "),
            theme.badge(Color::Green),
        ),
        Span::raw("  "),
        Span::styled(
            if app.focus == Focus::List { "list" } else { "detail" },
            theme.accent(),
        ),
    ]);

    let right = Line::from(vec![
        Span::styled("j/k select · enter toggle · ? help · q quit  ", theme.muted()),
    ]);

    let left_p = Paragraph::new(left).style(theme.status_bar());
    let right_p = Paragraph::new(right)
        .style(theme.status_bar())
        .alignment(Alignment::Right);

    frame.render_widget(left_p, area);
    frame.render_widget(right_p, area);
}

fn draw_help(frame: &mut Frame, theme: &Theme, area: Rect) {
    let help = r#"
keys
────────────────────────
j / k        select
gg / G       first / last
h / l        focus pane
enter        toggle service
r            restart
s            stop
S            start
?            help
q / esc      quit

legend
────────────────────────
green        running
yellow       restarting
gray         stopped
"#;

    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area)[1];
    let width = area.width.min(44);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let popup = Rect::new(x, popup.y, width, popup.height);

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(help)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(theme.accent())
                    .padding(Padding::horizontal(1))
                    .style(theme.background()),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn draw_toast(frame: &mut Frame, theme: &Theme, area: Rect, text: &str, kind: ToastKind) {
    let border_style = match kind {
        ToastKind::Info => theme.accent(),
        ToastKind::Error => theme.error(),
    };
    let max_width = area.width.saturating_sub(2).min(48) as u16;
    let width = (text.len() as u16 + 4).min(max_width);
    let toast = Rect::new(
        area.right().saturating_sub(width + 1),
        area.y + 1,
        width,
        3,
    );

    frame.render_widget(Clear, toast);
    frame.render_widget(
        Paragraph::new(Span::styled(text, theme.normal()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(border_style)
                    .style(theme.background()),
            )
            .wrap(Wrap { trim: true }),
        toast,
    );
}

fn kv(key: &str, value: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<8}", key), theme.muted()),
        Span::styled(value.to_string(), theme.normal()),
    ])
}

fn fmt_uptime(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, (secs / 60) % 60, secs % 60);
    format!("{h:02}h {m:02}m {s:02}s")
}
