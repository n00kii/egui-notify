use crate::{ERROR_COLOR, INFO_COLOR, SUCCESS_COLOR, TOAST_HEIGHT, TOAST_WIDTH, WARNING_COLOR};
use egui::{vec2, Color32, Vec2};
use std::{
    fmt::{Debug, Display},
    time::{Duration, SystemTime},
};

/// Level of importance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ToastLevel {
    Info,
    Warning,
    Error,
    Success,
    None,
}

impl ToastLevel {
    pub(crate) fn color(&self) -> Color32 {
        match self {
            Self::Info => INFO_COLOR,
            Self::Warning => WARNING_COLOR,
            Self::Error => ERROR_COLOR,
            Self::Success => SUCCESS_COLOR,
            Self::None => Color32::WHITE,
        }
    }
}

impl Display for ToastLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self {
            Self::Info => egui_phosphor::INFO,
            Self::Warning => egui_phosphor::QUESTION,
            Self::Error => egui_phosphor::WARNING_DIAMOND,
            Self::Success => egui_phosphor::CHECK_CIRCLE,
            Self::None => "",
        };
        write!(f, "{icon}")
    }
}

impl Default for ToastLevel {
    fn default() -> Self {
        ToastLevel::Info
    }
}

#[derive(Debug)]
pub(crate) enum ToastState {
    Appear,
    Disapper,
    Disappeared,
    Idle,
}

impl ToastState {
    pub fn appearing(&self) -> bool {
        matches!(self, Self::Appear)
    }
    pub fn disappearing(&self) -> bool {
        matches!(self, Self::Disapper)
    }
    pub fn disappeared(&self) -> bool {
        matches!(self, Self::Disappeared)
    }
    pub fn idling(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

/// Container for options for initlizing toasts
pub struct ToastOptions {
    duration: Option<Duration>,
    level: ToastLevel,
    closable: bool,
    show_progress_bar: bool,
}

/// Single notification or *toast*
#[derive(Debug)]
pub struct Toast {
    pub(crate) level: ToastLevel,
    pub(crate) caption: String,
    // (initial, current)
    pub(crate) duration: Option<(f32, f32)>,
    pub(crate) height: f32,
    pub(crate) width: f32,
    pub(crate) closable: bool,
    pub(crate) show_progress_bar: bool,

    pub(crate) toast_hovered: bool,
    pub(crate) cross_hovered: bool,

    pub(crate) timestamp: u128,

    pub(crate) state: ToastState,
    pub(crate) value: f32,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            duration: Some(Duration::from_millis(3500)),
            level: ToastLevel::None,
            closable: true,
            show_progress_bar: true,
        }
    }
}

fn duration_to_seconds_f32(duration: Duration) -> f32 {
    duration.as_nanos() as f32 * 1e-9
}

impl Toast {
    fn new(caption: impl Into<String>, options: ToastOptions) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self {
            caption: caption.into(),
            height: TOAST_HEIGHT,
            width: TOAST_WIDTH,
            duration: if let Some(dur) = options.duration {
                let max_dur = duration_to_seconds_f32(dur);
                Some((max_dur, max_dur))
            } else {
                None
            },
            closable: options.closable,
            show_progress_bar: options.show_progress_bar,
            level: options.level,

            toast_hovered: false,
            cross_hovered: false,

            timestamp,

            value: 0.,
            state: ToastState::Appear,
        }
    }

    /// Creates new basic toast, can be closed by default.
    pub fn basic(caption: impl Into<String>) -> Self {
        Self::new(caption, ToastOptions::default())
    }

    /// Creates new success toast, can be closed by default.
    pub fn success(caption: impl Into<String>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Success,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new info toast, can be closed by default.
    pub fn info(caption: impl Into<String>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Info,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new warning toast, can be closed by default.
    pub fn warning(caption: impl Into<String>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Warning,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new error toast, can not be closed by default.
    pub fn error(caption: impl Into<String>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                closable: false,
                level: ToastLevel::Error,
                ..ToastOptions::default()
            },
        )
    }

    /// Set the options with a ToastOptions
    pub fn set_options(&mut self, options: ToastOptions) -> &mut Self {
        self.set_closable(options.closable);
        self.set_duration(options.duration);
        self.set_level(options.level);
        self
    }

    /// Change the level of the toast
    pub fn set_level(&mut self, level: ToastLevel) -> &mut Self {
        self.level = level;
        self
    }

    /// Can use close the toast?
    pub fn set_closable(&mut self, closable: bool) -> &mut Self {
        self.closable = closable;
        self
    }

    /// Should a progress bar be shown?
    pub fn set_show_progress_bar(&mut self, show_progress_bar: bool) -> &mut Self {
        self.show_progress_bar = show_progress_bar;
        self
    }

    /// In what time should the toast expire? Set to `None` for no expiry.
    pub fn set_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        if let Some(duration) = duration {
            let max_dur = duration_to_seconds_f32(duration);
            self.duration = Some((max_dur, max_dur));
        } else {
            self.duration = None;
        }
        self
    }

    /// Toast's box height
    pub fn set_height(&mut self, height: f32) -> &mut Self {
        self.height = height;
        self
    }

    /// Toast's box width
    pub fn set_width(&mut self, width: f32) -> &mut Self {
        self.width = width;
        self
    }

    /// Dismiss this toast
    pub fn dismiss(&mut self) {
        self.state = ToastState::Disapper;
    }

    pub(crate) fn size(&self) -> Vec2 {
        vec2(self.width, self.height)
    }
}
