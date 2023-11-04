use eframe::{
    egui::{Context, Slider, Window},
    App, Frame, NativeOptions,
};
use egui::{Align2, RichText};
use egui_extras::{Column, TableBuilder};
use egui_notify::{ToastLevel, Toasts};
use std::time::Duration;

const DEFAULT_DURATION: u64 = 3500;
const ROW_HEIGHT: f32 = 20.;
struct ExampleApp {
    toasts: Toasts,
    caption: String,
    // closable: bool,
    // show_progress_bar: bool,
    // expires: bool,
    // duration: f32,
}
impl App for ExampleApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        Window::new("controls").title_bar(false).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.group(|ui| {
                    ui.heading("caption");
                    ui.text_edit_multiline(&mut self.caption);
                });
                ui.group(|ui| {
                    ui.heading("options");
                    let mut expires = self.toasts.default_options.duration.is_some();
                    TableBuilder::new(ui)
                        .striped(true)
                        .columns(Column::remainder(), 2)
                        .body(|mut body| {
                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.label("expires?");
                                });
                                row.col(|ui| {
                                    if ui.checkbox(&mut expires, "").clicked() {
                                        if expires {
                                            self.toasts.default_options.set_duration(
                                                Duration::from_millis(DEFAULT_DURATION),
                                            )
                                        } else {
                                            self.toasts.default_options.duration = None;
                                        }
                                    };
                                });
                            });
                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.label("closable?");
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut self.toasts.default_options.closable, "");
                                });
                            });
                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.label("progressbar?");
                                });
                                row.col(|ui| {
                                    ui.checkbox(
                                        &mut self.toasts.default_options.show_progress_bar,
                                        "",
                                    );
                                });
                            });
                            body.row(ROW_HEIGHT, |mut row| {
                                if !(expires || self.toasts.default_options.closable) {
                                    row.col(|ui| {
                                        ui.label("toasts will have to be closed programatically");
                                    });
                                } else {
                                    row.col(|ui| {
                                        ui.label("duration (ms)");
                                    });
                                    row.col(|ui| {
                                        ui.add_enabled_ui(expires, |ui| {
                                            if let Some(duration) =
                                                self.toasts.default_options.duration.as_mut()
                                            {
                                                ui.add(Slider::new(&mut duration.0, 1.0..=10.0));
                                                duration.1 = duration.0;
                                            };
                                        });
                                    });
                                }
                            });
                        });
                });

                ui.group(|ui| {
                    ui.heading("anchor");
                    ui.selectable_value(&mut self.toasts.anchor, Align2::LEFT_TOP, "left top");
                    ui.selectable_value(&mut self.toasts.anchor, Align2::CENTER_TOP, "center top");
                    ui.selectable_value(&mut self.toasts.anchor, Align2::RIGHT_TOP, "right top");

                    ui.selectable_value(
                        &mut self.toasts.anchor,
                        Align2::LEFT_CENTER,
                        "left center",
                    );
                    ui.selectable_value(
                        &mut self.toasts.anchor,
                        Align2::CENTER_CENTER,
                        "center center",
                    );
                    ui.selectable_value(
                        &mut self.toasts.anchor,
                        Align2::RIGHT_CENTER,
                        "right center",
                    );

                    ui.selectable_value(
                        &mut self.toasts.anchor,
                        Align2::LEFT_BOTTOM,
                        "left bottom",
                    );
                    ui.selectable_value(
                        &mut self.toasts.anchor,
                        Align2::CENTER_BOTTOM,
                        "center bottom",
                    );
                    ui.selectable_value(
                        &mut self.toasts.anchor,
                        Align2::RIGHT_BOTTOM,
                        "right bottom",
                    );
                });

                fn color(text: &str, level: ToastLevel) -> RichText {
                    RichText::new(text).color(level.color())
                }

                ui.group(|ui| {
                    ui.heading("toasts");

                    if ui.button(color("success", ToastLevel::Success)).clicked() {
                        self.toasts.success(self.caption.clone());
                    }
                    if ui.button(color("info", ToastLevel::Info)).clicked() {
                        self.toasts.info(self.caption.clone());
                    }
                    if ui.button(color("warning", ToastLevel::Warning)).clicked() {
                        self.toasts.warning(self.caption.clone());
                    }
                    if ui.button(color("error", ToastLevel::Error)).clicked() {
                        self.toasts.error(self.caption.clone());
                    }
                    if ui.button("basic").clicked() {
                        self.toasts.basic(self.caption.clone());
                    }
                    if ui.button("double basic").clicked() {
                        self.toasts.basic(self.caption.clone());
                        self.toasts.basic(self.caption.clone());
                    }
                });

                ui.group(|ui| {
                    ui.heading("actions");
                    ui.horizontal(|ui| {
                        if ui.button("dismiss all toasts").clicked() {
                            self.toasts.dismiss_all_toasts();
                        }
                        if ui.button("dismiss latest toast").clicked() {
                            self.toasts.dismiss_latest_toast();
                        }
                        if ui.button("dismiss oldest toast").clicked() {
                            self.toasts.dismiss_oldest_toast();
                        }
                    });
                });
            });
        });

        self.toasts.show(ctx);
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "example",
        NativeOptions::default(),
        Box::new(|cc| {
            egui_notify::load_icon_font(&cc.egui_ctx);
            Box::new(ExampleApp {
                caption: r#"Hello! It's a multiline caption
Next line
Another one
And another one"#
                    .into(),
                toasts: Toasts::default(),
            })
        }),
    )
}
