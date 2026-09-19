//! Чистое состояние приложения (без привязки к ratatui/crossterm).

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
}

impl Service {
    fn new(name: &str, state: ServiceState) -> Self {
        Service {
            name: name.to_string(),
            state,
            cpu: 0.0,
            mem_mb: 0.0,
            uptime_s: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
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
    pub show_help: bool,
    pub toast: Option<(String, ToastKind)>,
    toast_ticks: u32,
    rng: u64,
}

impl App {
    pub fn new() -> Self {
        App {
            services: seed(),
            selected: 0,
            focus: Focus::List,
            show_help: false,
            toast: None,
            toast_ticks: 0,
            rng: 0x9e37_79b9_7f4a_7c15,
        }
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
        self.set_toast(format!("toggled {}", self.selected_name()), ToastKind::Info);
    }

    pub fn stop(&mut self) {
        if let Some(svc) = self.services.get_mut(self.selected) {
            svc.state = ServiceState::Stopped;
        }
        self.set_toast(format!("stopped {}", self.selected_name()), ToastKind::Info);
    }

    pub fn start(&mut self) {
        if let Some(svc) = self.services.get_mut(self.selected) {
            svc.state = ServiceState::Running;
        }
        self.set_toast(format!("started {}", self.selected_name()), ToastKind::Info);
    }

    pub fn restart(&mut self) {
        if let Some(svc) = self.services.get_mut(self.selected) {
            svc.state = ServiceState::Restarting;
        }
        self.set_toast(format!("restarting {}", self.selected_name()), ToastKind::Info);
    }

    pub fn running_count(&self) -> usize {
        self.services
            .iter()
            .filter(|s| s.state == ServiceState::Running)
            .count()
    }

    pub fn set_toast(&mut self, text: String, kind: ToastKind) {
        self.toast = Some((text, kind));
        self.toast_ticks = 0;
    }

    /// Обновление «метрик» по тику (симуляция живых данных).
    pub fn tick(&mut self) {
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
                svc.cpu = clamp(svc.cpu + (a - 0.5) * 6.0, 0.0, 100.0);
                svc.mem_mb = clamp(svc.mem_mb + (b - 0.5) * 4.0, 8.0, 2048.0);
            } else {
                svc.cpu = 0.0;
            }
        }
        self.rng = rng;
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

fn seed() -> Vec<Service> {
    vec![
        Service::new("api", ServiceState::Running),
        Service::new("db", ServiceState::Running),
        Service::new("cache", ServiceState::Stopped),
        Service::new("worker", ServiceState::Running),
        Service::new("scheduler", ServiceState::Stopped),
        Service::new("proxy", ServiceState::Running),
    ]
}
