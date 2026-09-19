//! Отрисовка: заголовок, вкладки, три колонки (список / детали / бар-чарт),
//! линейный график, логи, статус-бар, оверлей справки, тост.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Bar, BarChart, Block, Borders, Chart, Clear, Dataset, Gauge, GraphType, List, ListItem,
    ListState, Padding, Paragraph, Sparkline, Tabs, Wrap,
};
use ratatui::Frame;

use crate::model::{App, Focus, ServiceState, Tab, ToastKind};
use crate::theme::Theme;

pub fn draw(frame: &mut Frame, app: &mut App, theme: &Theme) {
    let area = frame.area();

    frame.render_widget(Block::default().style(theme.background()), area);

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);
    let (header_area, tabs_area, main_area, status_area) =
        (split[0], split[1], split[2], split[3]);

    draw_header(frame, app, theme, header_area);
    draw_tabs(frame, app, theme, tabs_area);

    match app.tab {
        Tab::Dashboard => draw_dashboard(frame, app, theme, main_area),
        Tab::Metrics => draw_metrics(frame, app, theme, main_area),
        Tab::Logs => draw_logs(frame, app, theme, main_area),
    }

    draw_status_bar(frame, app, theme, status_area);

    if app.show_help {
        draw_help(frame, theme, area);
    }
    if let Some((text, kind)) = &app.toast {
        draw_toast(frame, theme, area, text, *kind);
    }
}

fn draw_header(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let title = Line::from(Span::styled(" mock-monitor ", theme.accent()));
    let clock = Line::from(Span::styled(app.clock_str(), theme.muted()));

    frame.render_widget(Paragraph::new(title).style(theme.status_bar()), area);
    frame.render_widget(
        Paragraph::new(clock)
            .style(theme.status_bar())
            .alignment(Alignment::Right),
        area,
    );
}

fn draw_tabs(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let titles = vec![
        Tab::Dashboard.label(),
        Tab::Metrics.label(),
        Tab::Logs.label(),
    ];
    let tabs = Tabs::new(titles)
        .select(match app.tab {
            Tab::Dashboard => 0,
            Tab::Metrics => 1,
            Tab::Logs => 2,
        })
        .highlight_style(theme.accent())
        .divider("  ");
    frame.render_widget(tabs, area);
}

fn draw_dashboard(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(area);
    let (cols_area, logs_area) = (split[0], split[1]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(36),
            Constraint::Percentage(36),
            Constraint::Percentage(28),
        ])
        .split(cols_area);

    draw_list(frame, app, theme, cols[0]);
    draw_overview(frame, app, theme, cols[1]);
    draw_barchart(frame, app, theme, cols[2]);

    draw_logs(frame, app, theme, logs_area);
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
                    format!("cpu {:>4.1}%  mem {:>6.0} MB", svc.cpu, svc.mem_mb),
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

fn draw_overview(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let focused = app.focus == Focus::Detail;
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(if focused { theme.accent() } else { theme.border() });
    let inner = block.inner(area);

    frame.render_widget(Paragraph::new("").block(block), area);

    let Some(svc) = app.services.get(app.selected) else {
        return;
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let state_style = match svc.state {
        ServiceState::Running => theme.success(),
        ServiceState::Stopped => theme.muted(),
        ServiceState::Restarting => theme.warning(),
    };
    let header = Line::from(vec![
        Span::raw("  "),
        Span::styled(&svc.name, theme.accent()),
        Span::raw("  "),
        Span::styled(svc.state.label(), state_style),
        Span::raw("   up "),
        Span::styled(fmt_uptime(svc.uptime_s), theme.muted()),
    ]);
    frame.render_widget(Paragraph::new(header), rows[0]);

    draw_gauge(
        frame,
        (svc.cpu / 100.0) as f64,
        &format!(" cpu {:.1}% ", svc.cpu),
        theme.success(),
        rows[1],
    );
    draw_gauge(
        frame,
        (svc.mem_mb / 2048.0) as f64,
        &format!(" mem {:.0} MB ", svc.mem_mb),
        theme.warning(),
        rows[2],
    );

    let spark_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(rows[3]);
    frame.render_widget(
        Paragraph::new(Span::styled(" cpu history", theme.muted())),
        spark_rows[0],
    );
    draw_sparkline(frame, &svc.cpu_history, theme.accent(), spark_rows[1]);
}

fn draw_barchart(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let bars: Vec<Bar> = app
        .services
        .iter()
        .map(|s| Bar::with_label(s.name.as_str(), s.cpu as u64))
        .collect();

    let chart = BarChart::new(bars)
        .block(
            Block::default()
                .title(" CPU % ")
                .borders(Borders::ALL)
                .border_style(theme.border()),
        )
        .bar_width(2)
        .bar_gap(1)
        .bar_style(theme.accent())
        .max(100)
        .direction(Direction::Vertical);

    frame.render_widget(chart, area);
}

fn draw_metrics(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let Some(svc) = app.services.get(app.selected) else {
        return;
    };

    let data: Vec<(f64, f64)> = svc
        .cpu_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let dataset = Dataset::default()
        .name("cpu %")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(theme.accent())
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title(format!(" {} · cpu history ", svc.name))
                .borders(Borders::ALL)
                .border_style(theme.border()),
        )
        .x_axis(Axis::default().bounds([0.0, 64.0]))
        .y_axis(Axis::default().bounds([0.0, 100.0]).labels(["0", "50", "100"]));

    frame.render_widget(chart, area);
}

fn draw_logs(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .border_style(theme.border());

    let tail: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .rev()
        .map(|line| ListItem::new(Line::from(Span::styled(line, theme.normal()))))
        .collect();

    frame.render_widget(List::new(tail).block(block), area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let running = app.running_count();
    let total = app.services.len();

    let left = Line::from(vec![
        Span::raw(" "),
        Span::styled(format!(" {running}/{total} up "), theme.badge(Color::Green)),
        Span::raw("  "),
        Span::styled(
            match app.focus {
                Focus::List => "list".to_string(),
                Focus::Detail => app.tab.label().to_lowercase(),
            },
            theme.accent(),
        ),
    ]);

    let right = Line::from(vec![Span::styled(
        "tab: switch · j/k select · enter toggle · ? help · q quit  ",
        theme.muted(),
    )]);

    frame.render_widget(Paragraph::new(left).style(theme.status_bar()), area);
    frame.render_widget(
        Paragraph::new(right)
            .style(theme.status_bar())
            .alignment(Alignment::Right),
        area,
    );
}

fn draw_gauge(frame: &mut Frame, ratio: f64, label: &str, style: ratatui::style::Style, area: Rect) {
    let gauge = Gauge::default()
        .ratio(ratio.clamp(0.0, 1.0))
        .label(label)
        .gauge_style(style)
        .use_unicode(true);
    frame.render_widget(gauge, area);
}

fn draw_sparkline(frame: &mut Frame, data: &[u64], style: ratatui::style::Style, area: Rect) {
    let spark = Sparkline::default().data(data).max(100).style(style);
    frame.render_widget(spark, area);
}

fn draw_help(frame: &mut Frame, theme: &Theme, area: Rect) {
    let help = r#"
keys
────────────────────────
j / k        select
gg / G       first / last
h / l        focus pane
tab          switch tab
enter        toggle service
r / s / S    restart / stop / start
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

fn fmt_uptime(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, (secs / 60) % 60, secs % 60);
    format!("{h:02}h {m:02}m {s:02}s")
}
