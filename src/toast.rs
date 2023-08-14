use crate::{ERROR_COLOR, INFO_COLOR, SUCCESS_COLOR, TOAST_HEIGHT, TOAST_WIDTH, WARNING_COLOR};
use crossbeam_channel::{Receiver, Sender};
use egui::{vec2, Color32, Vec2};
use std::{
    fmt::{Debug, Display},
    time::{Duration, SystemTime},
};

const DEFAULT_TOAST_DURATION: f32 = 3.5;

/// Level of importance
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ToastLevel {
    #[default]
    Info,
    Warning,
    Error,
    Success,
    None,
}

impl ToastLevel {
    /// Color used for the level.
    pub fn color(&self) -> Color32 {
        match self {
            Self::Info => INFO_COLOR,
            Self::Warning => WARNING_COLOR,
            Self::Error => ERROR_COLOR,
            Self::Success => SUCCESS_COLOR,
            Self::None => Color32::GRAY,
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
#[derive(Debug, Clone)]
pub struct ToastOptions {
    // (initial, current)
    pub duration: Option<(f32, f32)>,
    pub level: ToastLevel,
    pub closable: bool,
    pub show_progress_bar: bool,
}

impl ToastOptions {
    pub fn set_duration(&mut self, duration: Duration) {
        let secs = duration_to_seconds_f32(duration);
        self.duration = Some((secs, secs));
    }
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            duration: Some((DEFAULT_TOAST_DURATION, DEFAULT_TOAST_DURATION)),
            level: ToastLevel::None,
            closable: true,
            show_progress_bar: true,
        }
    }
}

pub struct ToastUpdate {
    pub(crate) caption: Option<String>,
    pub(crate) level: Option<ToastLevel>,
    pub(crate) fallback_options: Option<ToastOptions>,
    pub(crate) use_original_options: bool,
}

impl ToastUpdate {
    pub fn caption(caption: impl Into<String>) -> Self {
        Self {
            use_original_options: false,
            caption: Some(caption.into()),
            fallback_options: None,
            level: None,
        }
    }
    pub fn success(caption: impl Into<String>) -> Self {
        Self::caption(caption).with_level(ToastLevel::Success)
    }
    pub fn error(caption: impl Into<String>) -> Self {
        Self::caption(caption).with_level(ToastLevel::Error)
    }
    pub fn warning(caption: impl Into<String>) -> Self {
        Self::caption(caption).with_level(ToastLevel::Warning)
    }
    pub fn info(caption: impl Into<String>) -> Self {
        Self::caption(caption).with_level(ToastLevel::Info)
    }
    pub fn with_level(mut self, level: ToastLevel) -> Self {
        self.level = Some(level);
        if let Some(fallback_options) = self.fallback_options.as_mut() {
            fallback_options.level = level;
        }
        self
    }
    pub fn with_original_options(mut self) -> Self {
        self.use_original_options = true;
        self
    }
    pub fn with_fallback_options(mut self, mut fallback_options: ToastOptions) -> Self {
        if let Some(level) = self.level {
            fallback_options.level = level;
        }
        self.fallback_options = Some(fallback_options);
        self
    }
}

/// Single notification or *toast*
#[derive(Debug)]
pub struct Toast {
    pub(crate) caption: String,
    pub(crate) options: ToastOptions,
    pub(crate) original_options: ToastOptions,
    pub(crate) fallback_options: Option<ToastOptions>,

    pub(crate) height: f32,
    pub(crate) width: f32,

    pub(crate) toast_hovered: bool,
    pub(crate) cross_hovered: bool,

    pub(crate) timestamp: u128,
    pub(crate) update_reciever: Option<Receiver<ToastUpdate>>,

    pub(crate) state: ToastState,
    pub(crate) value: f32,
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
            original_options: options.clone(),
            options,
            toast_hovered: false,
            cross_hovered: false,
            update_reciever: None,
            timestamp,
            value: 0.,
            fallback_options: None,
            state: ToastState::Appear,
        }
    }

    /// Enables the toast to listen to channel updates.
    pub fn create_channel(&mut self) -> Sender<ToastUpdate> {
        let (sender, reciever) = crossbeam_channel::unbounded();
        self.options.duration = None;
        self.options.closable = false;
        self.update_reciever = Some(reciever);
        sender
    }

    /// Creates new basic toast, can be closed by default.
    pub fn basic(caption: impl Into<String>) -> Self {
        Self::new(caption, ToastOptions::default())
    }

    /// Creates new success toast, can be closed by default.
    pub fn success(mut self) -> Self {
        self.options.level = ToastLevel::Success;
        self
    }

    /// Creates new info toast, can be closed by default.
    pub fn info(mut self) -> Self {
        self.options.level = ToastLevel::Info;
        self
    }

    /// Creates new warning toast, can be closed by default.
    pub fn warning(mut self) -> Self {
        self.options.level = ToastLevel::Warning;
        self
    }

    /// Creates new error toast, can not be closed by default.
    pub fn error(mut self) -> Self {
        self.options.level = ToastLevel::Error;
        self
    }

    /// Set the options with a ToastOptions
    pub fn with_options(mut self, options: &ToastOptions) -> Self {
        self.options = options.clone();
        self
    }

    /// Change the level of the toast
    pub fn set_level(&mut self, level: ToastLevel) -> &mut Self {
        self.options.level = level;
        self
    }

    /// Can use close the toast?
    pub fn set_closable(&mut self, closable: bool) -> &mut Self {
        self.options.closable = closable;
        self
    }

    /// Should a progress bar be shown?
    pub fn set_show_progress_bar(&mut self, show_progress_bar: bool) -> &mut Self {
        self.options.show_progress_bar = show_progress_bar;
        self
    }

    /// In what time should the toast expire? Set to `None` for no expiry.
    pub fn set_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        if let Some(duration) = duration {
            let max_dur = duration_to_seconds_f32(duration);
            self.options.duration = Some((max_dur, max_dur));
        } else {
            self.options.duration = None;
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
