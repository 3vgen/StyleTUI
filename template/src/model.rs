//! Чистое состояние приложения (без привязки к ratatui/crossterm).

const CPU_HISTORY_LEN: usize = 64;
const LOG_MAX: usize = 200;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Stopped,
    Restarting,
}

impl ServiceState {
    pub fn label(self) -> &'static str {
        match self {
            ServiceState::Running => " RUNNING ",
            ServiceState::Stopped => " STOPPED ",
            ServiceState::Restarting => " RESTART ",
        }
    }
}

pub struct Service {
    pub name: String,
    pub state: ServiceState,
    pub cpu: f32,
    pub mem_mb: f32,
    pub uptime_s: u64,
    pub cpu_history: Vec<u64>,
    base_cpu: f32,
    base_mem: f32,
}

impl Service {
    fn new(name: &str, state: ServiceState, base_cpu: f32, base_mem: f32) -> Self {
        Service {
            name: name.to_string(),
            state,
            cpu: base_cpu,
            mem_mb: base_mem,
            uptime_s: 0,
            cpu_history: Vec::with_capacity(CPU_HISTORY_LEN),
            base_cpu,
            base_mem,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Metrics,
    Logs,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Metrics => "Metrics",
            Tab::Logs => "Logs",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    #[allow(dead_code)]
    Error,
}

pub struct App {
    pub services: Vec<Service>,
    pub selected: usize,
    pub focus: Focus,
    pub tab: Tab,
    pub show_help: bool,
    pub toast: Option<(String, ToastKind)>,
    pub logs: Vec<String>,
    toast_ticks: u32,
    rng: u64,
    clock: u64,
    tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            services: seed(),
            selected: 0,
            focus: Focus::List,
            tab: Tab::Dashboard,
            show_help: false,
            toast: None,
            logs: Vec::new(),
            toast_ticks: 0,
            rng: 0x9e37_79b9_7f4a_7c15,
            clock: 0,
            tick_count: 0,
        };
        app.log("daemon started");
        app.log("6 services discovered");
        app
    }

    pub fn next(&mut self) {
        if !self.services.is_empty() {
            self.selected = (self.selected + 1).min(self.services.len() - 1);
        }
    }

    pub fn prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn first(&mut self) {
        self.selected = 0;
    }

    pub fn last(&mut self) {
        if !self.services.is_empty() {
            self.selected = self.services.len() - 1;
        }
    }

    fn selected_name(&self) -> String {
        self.services
            .get(self.selected)
            .map(|s| s.name.clone())
            .unwrap_or_default()
    }

    pub fn toggle(&mut self) {
        if let Some(svc) = self.services.get_mut(self.selected) {
            svc.state = match svc.state {
                ServiceState::Running => ServiceState::Stopped,
                ServiceState::Stopped => ServiceState::Running,
                ServiceState::Restarting => ServiceState::Running,
            };
        }
        let name = self.selected_name();
        self.log(&format!("toggle {name}"));
        self.set_toast(format!("toggled {name}"), ToastKind::Info);
    }

    pub fn stop(&mut self) {
        if let Some(svc) = self.services.get_mut(self.selected) {
            svc.state = ServiceState::Stopped;
        }
        let name = self.selected_name();
        self.log(&format!("stop {name}"));
        self.set_toast(format!("stopped {name}"), ToastKind::Info);
    }

    pub fn start(&mut self) {
        if let Some(svc) = self.services.get_mut(self.selected) {
            svc.state = ServiceState::Running;
        }
        let name = self.selected_name();
        self.log(&format!("start {name}"));
        self.set_toast(format!("started {name}"), ToastKind::Info);
    }

    pub fn restart(&mut self) {
        if let Some(svc) = self.services.get_mut(self.selected) {
            svc.state = ServiceState::Restarting;
        }
        let name = self.selected_name();
        self.log(&format!("restart {name}"));
        self.set_toast(format!("restarting {name}"), ToastKind::Info);
    }

    pub fn running_count(&self) -> usize {
        self.services
            .iter()
            .filter(|s| s.state == ServiceState::Running)
            .count()
    }

    pub fn clock_str(&self) -> String {
        fmt_clock(self.clock)
    }

    pub fn set_toast(&mut self, text: String, kind: ToastKind) {
        self.toast = Some((text, kind));
        self.toast_ticks = 0;
    }

    pub fn log(&mut self, text: &str) {
        let line = format!("{} {text}", fmt_clock(self.clock));
        self.logs.push(line);
        if self.logs.len() > LOG_MAX {
            self.logs.remove(0);
        }
    }

    /// Обновление «метрик» по тику (симуляция живых данных).
    pub fn tick(&mut self) {
        self.clock += 1;
        self.tick_count += 1;

        let mut rng = self.rng;
        for svc in &mut self.services {
            if svc.state == ServiceState::Restarting {
                svc.state = ServiceState::Running;
                svc.uptime_s = 0;
            }
            if svc.state == ServiceState::Running {
                svc.uptime_s += 1;
                let a = next_f32(&mut rng);
                let b = next_f32(&mut rng);
                svc.cpu = clamp(svc.base_cpu + (a - 0.5) * 24.0, 0.5, 100.0);
                svc.mem_mb = clamp(svc.base_mem + (b - 0.5) * 48.0, 16.0, 2048.0);
            } else {
                svc.cpu = 0.0;
            }
            svc.cpu_history.push(svc.cpu as u64);
            if svc.cpu_history.len() > CPU_HISTORY_LEN {
                svc.cpu_history.remove(0);
            }
        }
        self.rng = rng;

        if self.tick_count % 24 == 0 {
            self.log("metrics heartbeat");
        }

        self.toast_ticks += 1;
        if self.toast_ticks > 8 {
            self.toast = None;
        }
    }
}

fn next_f32(rng: &mut u64) -> f32 {
    *rng = rng
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*rng >> 33) as u32 as f32) / (u32::MAX as f32)
}

fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    v.max(lo).min(hi)
}

fn fmt_clock(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, (secs / 60) % 60, secs % 60);
    format!("{h:02}:{m:02}:{s:02}")
}

fn seed() -> Vec<Service> {
    vec![
        Service::new("api", ServiceState::Running, 42.0, 512.0),
        Service::new("db", ServiceState::Running, 18.0, 1024.0),
        Service::new("cache", ServiceState::Stopped, 0.0, 64.0),
        Service::new("worker", ServiceState::Running, 65.0, 384.0),
        Service::new("scheduler", ServiceState::Stopped, 0.0, 32.0),
        Service::new("proxy", ServiceState::Running, 8.0, 128.0),
    ]
}
