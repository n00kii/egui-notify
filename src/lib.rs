//! egui-notify
//! Simple notifications library for EGUI

#![warn(missing_docs)]

mod toast;
use crossbeam_channel::TryRecvError;
pub use toast::*;

#[doc(hidden)]
pub use egui::__run_test_ctx;
use egui::{
    epaint::Shadow, pos2, vec2, Align2, Color32, Context, FontId, Id, LayerId, Order, Pos2, Rect,
    Rounding, Stroke, Vec2,
};

pub(crate) const TOAST_WIDTH: f32 = 180.;
pub(crate) const TOAST_HEIGHT: f32 = 34.;

const ERROR_COLOR: Color32 = Color32::from_rgb(200, 90, 90);
const INFO_COLOR: Color32 = Color32::from_rgb(150, 200, 210);
const WARNING_COLOR: Color32 = Color32::from_rgb(230, 220, 140);
const SUCCESS_COLOR: Color32 = Color32::from_rgb(140, 230, 140);

/// Load icon font
pub fn load_icon_font(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts);

    let mut phosphor_data = fonts.font_data.get_mut("phosphor").unwrap();
    phosphor_data.tweak = egui::FontTweak {
        y_offset: 1.25,
        ..Default::default()
    };
    ctx.set_fonts(fonts);
}

/// Main notifications collector.
/// # Usage
/// You need to create [`Toasts`] once and call `.show(ctx)` in every frame.
/// ```
/// # use std::time::Duration;
/// use egui_notify::Toasts;
///
/// # egui_notify::__run_test_ctx(|ctx| {
/// let mut t = Toasts::default();
/// t.info("Hello, World!").set_duration(Some(Duration::from_secs(5))).set_closable(true);
/// // More app code
/// t.show(ctx);
/// # });
/// ```
pub struct Toasts {
    /// The attachment point for toasts
    pub anchor: Align2,
    /// Default toast options.
    pub default_options: ToastOptions,
    toasts: Vec<Toast>,
    margin: Vec2,
    spacing: f32,
    padding: Vec2,
    reverse: bool,
    speed: f32,

    held: bool,
}

impl Toasts {
    /// Creates new [`Toasts`] instance.
    pub fn new() -> Self {
        Self {
            default_options: ToastOptions::default(),
            anchor: Align2::RIGHT_BOTTOM,
            margin: vec2(8., 8.),
            toasts: vec![],
            spacing: 8.,
            padding: vec2(10., 10.),
            held: false,
            speed: 4.,
            reverse: false,
        }
    }

    /// Adds new toast to the collection.
    /// By default adds toast at the end of the list, can be changed with `self.reverse`.
    pub fn add(&mut self, toast: Toast) -> &mut Toast {
        if self.reverse {
            self.toasts.insert(0, toast);
            return self.toasts.get_mut(0).unwrap();
        } else {
            self.toasts.push(toast);
            let l = self.toasts.len() - 1;
            return self.toasts.get_mut(l).unwrap();
        }
    }

    /// Dismisses the oldest toast
    pub fn dismiss_oldest_toast(&mut self) {
        if let Some(toast) = self.toasts.get_mut(0) {
            toast.dismiss();
        }
    }

    /// Dismisses the most recent toast
    pub fn dismiss_latest_toast(&mut self) {
        if let Some(toast) = self.toasts.last_mut() {
            toast.dismiss();
        }
    }

    /// Dismisses all toasts
    pub fn dismiss_all_toasts(&mut self) {
        for toast in self.toasts.iter_mut() {
            toast.dismiss();
        }
    }

    fn base_toast(&self, caption: impl Into<String>) -> Toast {
        Toast::basic(caption).with_options(&self.default_options)
    }

    /// Shortcut for adding a toast with info `success`.
    pub fn success(&mut self, caption: impl Into<String>) -> &mut Toast {
        self.add(self.base_toast(caption).success())
    }

    /// Shortcut for adding a toast with info `level`.
    pub fn info(&mut self, caption: impl Into<String>) -> &mut Toast {
        self.add(self.base_toast(caption).info())
    }

    /// Shortcut for adding a toast with warning `level`.
    pub fn warning(&mut self, caption: impl Into<String>) -> &mut Toast {
        self.add(self.base_toast(caption).warning())
    }

    /// Shortcut for adding a toast with error `level`.
    pub fn error(&mut self, caption: impl Into<String>) -> &mut Toast {
        self.add(self.base_toast(caption).error())
    }

    /// Shortcut for adding a toast with no level.
    pub fn basic(&mut self, caption: impl Into<String>) -> &mut Toast {
        self.add(self.base_toast(caption))
    }

    /// Should toasts be added in reverse order?
    pub const fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    /// Where toasts should appear.
    pub const fn with_anchor(mut self, anchor: Align2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets spacing between adjacent toasts.
    pub const fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Margin or distance from screen to toasts' bounding boxes
    pub const fn with_margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    /// Padding or distance from toasts' bounding boxes to inner contents.
    pub const fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = padding;
        self
    }
}

impl Toasts {
    /// Displays toast queue
    pub fn show(&mut self, ctx: &Context) {
        let screen_rect = ctx.screen_rect();
        let mut toast_anchor = self
            .anchor
            .pos_in_rect_with_margin(&screen_rect, self.margin);
        let toasts_layer_id = Id::new("toasts");
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, toasts_layer_id));
        let mut dismiss: Option<usize> = None;

        // Remove disappeared toasts
        self.toasts.retain(|t| !t.state.disappeared());

        // Start disappearing expired toasts
        self.toasts.iter_mut().for_each(|t| {
            if let Some((_initial_d, current_d)) = t.options.duration {
                if current_d <= 0. {
                    t.state = ToastState::Disapper
                }
            }
        });

        // `held` used to prevent sticky removal
        if ctx.input(|i| i.pointer.primary_released()) {
            self.held = false;
        }

        let visuals = ctx.style().visuals.widgets.noninteractive;
        let mut repaint = false;

        for (i, toast) in self.toasts.iter_mut().enumerate() {
            let toast_id = toasts_layer_id.with(toast.timestamp);
            let mut disconnect = false;
            if let Some(update_res) = toast.update_reciever.as_ref() {
                match update_res.try_recv() {
                    Ok(update) => {
                        if let Some(caption) = update.caption {
                            toast.caption = caption
                        }
                        if let Some(fallback_options) = update.fallback_options {
                            toast.fallback_options = Some(fallback_options);
                        }
                        if let Some(level) = update.level {
                            toast.options.level = level
                        }
                    }
                    Err(TryRecvError::Disconnected) => {
                        disconnect = true;
                        if let Some(fallback_options) = toast.fallback_options.take() {
                            toast.options = fallback_options;
                        } else {
                            dismiss = Some(i);
                        }
                    }
                    _ => {}
                };
            }

            if disconnect {
                toast.update_reciever = None;
            }

            // Decrease duration if idling
            if let Some((_, d)) = toast.options.duration.as_mut() {
                if toast.state.idling() && !toast.toast_hovered {
                    *d -= ctx.input(|i| i.stable_dt);
                    repaint = true;
                }
            }

            // Create toast label
            let caption_galley = ctx.fonts(|f| {
                f.layout(
                    toast.caption.clone(),
                    FontId::proportional(16.),
                    visuals.fg_stroke.color,
                    f32::INFINITY,
                )
            });

            let (caption_width, caption_height) =
                (caption_galley.rect.width(), caption_galley.rect.height());

            let line_count = toast.caption.chars().filter(|c| *c == '\n').count() + 1;
            let icon_width = caption_height / line_count as f32;

            // Create toast icon
            let icon_font = FontId::proportional(icon_width);
            let icon_galley = if !matches!(toast.options.level, ToastLevel::None) {
                Some(ctx.fonts(|f| {
                    f.layout(
                        toast.options.level.to_string(),
                        icon_font,
                        toast.options.level.color(),
                        f32::INFINITY,
                    )
                }))
            } else {
                None
            };

            let (action_width, action_height) = if let Some(icon_galley) = icon_galley.as_ref() {
                (icon_galley.rect.width(), icon_galley.rect.height())
            } else {
                (0., 0.)
            };

            // Create closing cross
            let cross_galley = if toast.options.closable {
                let cross_fid = FontId::proportional(icon_width);
                let cross_galley = ctx.fonts(|f| {
                    f.layout(
                        "❌".into(),
                        cross_fid,
                        if toast.cross_hovered {
                            lighter(visuals.fg_stroke.color)
                        } else {
                            visuals.fg_stroke.color
                        },
                        f32::INFINITY,
                    )
                });
                Some(cross_galley)
            } else {
                None
            };

            let (cross_width, cross_height) = if let Some(cross_galley) = cross_galley.as_ref() {
                (cross_galley.rect.width(), cross_galley.rect.height())
            } else {
                (0., 0.)
            };

            let icon_x_padding = (0., 7.);
            let cross_x_padding = (7., 0.);

            let icon_width_padded = if icon_width == 0. {
                0.
            } else {
                icon_width + icon_x_padding.0 + icon_x_padding.1
            };
            let cross_width_padded = if cross_width == 0. {
                0.
            } else {
                cross_width + cross_x_padding.0 + cross_x_padding.1
            };

            toast.width =
                icon_width_padded + caption_width + cross_width_padded + (self.padding.x * 2.);
            toast.height =
                action_height.max(caption_height).max(cross_height) + self.padding.y * 2.;

            let anim_offset = toast.width * (1. - ease_in_cubic(toast.value));
            let toast_pos_x = toast_anchor.x + anim_offset * self.anchor.side();

            let toast_pos_y = ctx.animate_value_with_time(toast_id, toast_anchor.y, 0.1);
            let toast_rect = self
                .anchor
                .align_size_to_pos(pos2(toast_pos_x, toast_pos_y), toast.size());
            
            let toast_rect_rounding = Rounding::same(4.);
            let mut toast_shadow = Shadow::small_dark();

            toast_shadow.color = toast_shadow.color.linear_multiply(0.5);
            painter.add(toast_shadow.tessellate(toast_rect, toast_rect_rounding));

            // Draw background
            painter.rect(
                toast_rect,
                Rounding::same(4.),
                visuals.bg_fill,
                Stroke::new(
                    if toast.state.disappearing() { 0. } else { 1. },
                    toast.options.level.color(),
                ),
            );

            if toast.options.show_progress_bar {
                if let Some((initial, current)) = toast.options.duration {
                    if !toast.state.disappearing() {
                        let mut duration_rect = toast_rect;
                        duration_rect.set_left(
                            toast_rect.right() - (1. - (current / initial)) * toast_rect.width(),
                        );
                        painter.rect_stroke(
                            duration_rect,
                            Rounding::same(4.),
                            Stroke::new(2., visuals.bg_fill),
                        );
                    }
                }
            }

            // Paint icon
            if let Some((icon_galley, true)) =
                icon_galley.zip(Some(toast.options.level != ToastLevel::None))
            {
                let oy = toast.height / 2. - action_height / 2.;
                let ox = self.padding.x + icon_x_padding.0;
                painter.galley(toast_rect.min + vec2(ox, oy), icon_galley);
            }

            // Paint caption
            let oy = toast.height / 2. - caption_height / 2.;
            let o_from_icon = if action_width == 0. {
                0.
            } else {
                action_width + icon_x_padding.1
            };
            let o_from_cross = if cross_width == 0. {
                0.
            } else {
                cross_width + cross_x_padding.0
            };
            let ox = (toast.width / 2. - caption_width / 2.) + o_from_icon / 2. - o_from_cross / 2.;
            painter.galley(toast_rect.min + vec2(ox, oy), caption_galley);

            // Paint cross
            if let Some(cross_galley) = cross_galley {
                let cross_rect = cross_galley.rect;
                let oy = toast.height / 2. - cross_height / 2.;
                let ox = toast.width - cross_width - cross_x_padding.1 - self.padding.x;
                let cross_pos = toast_rect.min + vec2(ox, oy);
                painter.galley(cross_pos, cross_galley);

                let cross_screen_rect = Rect {
                    max: cross_pos + cross_rect.max.to_vec2(),
                    min: cross_pos,
                };

                if let Some(hover_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    toast.toast_hovered = toast_rect.contains(hover_pos);
                    toast.cross_hovered = cross_screen_rect.contains(hover_pos);
                }

                if let Some(click_pos) = ctx.input(|i| i.pointer.press_origin()) {
                    if cross_screen_rect.contains(click_pos) && !self.held {
                        dismiss = Some(i);
                        self.held = true;
                    }
                }
            }

            self.anchor
                .offset_height(&mut toast_anchor, self.spacing + toast.height);

            // Animations
            if toast.state.appearing() {
                repaint = true;
                toast.value += ctx.input(|i| i.stable_dt) * (self.speed);

                if toast.value >= 1. {
                    toast.value = 1.;
                    toast.state = ToastState::Idle;
                }
            } else if toast.state.disappearing() {
                repaint = true;
                toast.value -= ctx.input(|i| i.stable_dt) * (self.speed);

                if toast.value <= 0. {
                    toast.state = ToastState::Disappeared;
                }
            }
        }

        if repaint {
            ctx.request_repaint();
        }

        if let Some(i) = dismiss {
            self.toasts[i].dismiss();
        }
    }
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

fn mul_vec2(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
}

trait AnchorPoint {
    fn pos_in_rect_with_margin(&self, frame: &Rect, margin: Vec2) -> Pos2;
    fn align_size_to_pos(&self, anchor_pos: Pos2, size: Vec2) -> Rect;
    fn offset_height(&self, pos: &mut Pos2, offset: f32);
    fn side(&self) -> f32;
}

impl AnchorPoint for Align2 {
    fn pos_in_rect_with_margin(&self, frame: &Rect, margin: Vec2) -> Pos2 {
        let signed_margin = mul_vec2(margin, -self.to_sign());
        self.pos_in_rect(frame) + signed_margin
    }
    fn align_size_to_pos(&self, anchor_pos: Pos2, size: Vec2) -> Rect {
        let center_pos = anchor_pos + mul_vec2(size * 0.5, -self.to_sign());
        Rect::from_center_size(center_pos, size)
    }
    fn offset_height(&self, pos: &mut Pos2, offset: f32) {
        pos.y += -self.to_sign().y * offset
    }
    fn side(&self) -> f32 {
        self.to_sign().x
    }
}

const COLOR_LIGHTEN_FACTOR: f32 = 1.5;

fn lighter(base_color: Color32) -> Color32 {
    scale_color(base_color, COLOR_LIGHTEN_FACTOR)
}

fn scale_color(base_color: Color32, color_factor: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (base_color.r() as f32 * color_factor) as u8,
        (base_color.g() as f32 * color_factor) as u8,
        (base_color.b() as f32 * color_factor) as u8,
        (base_color.a()) as u8,
    )
}

fn ease_in_cubic(x: f32) -> f32 {
    1. - (1. - x).powi(3)
}
