use eframe::{
    egui::{Context, Slider, Window},
    App, Frame, NativeOptions,
};
use egui::{Align2, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use egui_notify::{Toast, ToastLevel, Toasts};
use std::time::Duration;

struct ExampleApp {
    toasts: Toasts,
    caption: String,
    closable: bool,
    show_progress_bar: bool,
    expires: bool,
    duration: f32,
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
                    TableBuilder::new(ui)
                        .striped(true)
                        .columns(Column::remainder(), 2)
                        .body(|mut body| {
                            body.row(20., |mut row| {
                                row.col(|ui| {
                                    ui.label("expires?");
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut self.expires, "");
                                });
                            });
                            body.row(20., |mut row| {
                                row.col(|ui| {
                                    ui.label("closable?");
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut self.closable, "");
                                });
                            });
                            body.row(20., |mut row| {
                                row.col(|ui| {
                                    ui.label("progressbar?");
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut self.show_progress_bar, "");
                                });
                            });
                            body.row(20., |mut row| {
                                if !(self.expires || self.closable) {
                                    row.col(|ui| {
                                        ui.label("toasts will have to be closed programatically");
                                    });
                                } else {
                                    row.col(|ui| {
                                        ui.label("duration (in s)");
                                    });
                                    row.col(|ui| {
                                        ui.add_enabled_ui(self.expires, |ui| {
                                            ui.add(Slider::new(&mut self.duration, 1.0..=10.0));
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

                let customize_toast = |t: &mut Toast| {
                    let duration = if self.expires {
                        Some(Duration::from_millis((1000. * self.duration) as u64))
                    } else {
                        None
                    };
                    t.set_closable(self.closable)
                        .set_duration(duration)
                        .set_show_progress_bar(self.show_progress_bar);
                };
                let colored_text =
                    |text: &str, color: Color32| -> RichText { RichText::new(text).color(color) };

                ui.group(|ui| {
                    ui.heading("toasts");

                    if ui
                        .button(colored_text("success", ToastLevel::Success.color()))
                        .clicked()
                    {
                        customize_toast(self.toasts.success(self.caption.clone()));
                    }

                    if ui
                        .button(colored_text("info", ToastLevel::Info.color()))
                        .clicked()
                    {
                        customize_toast(self.toasts.info(self.caption.clone()));
                    }

                    if ui
                        .button(colored_text("warning", ToastLevel::Warning.color()))
                        .clicked()
                    {
                        customize_toast(self.toasts.warning(self.caption.clone()));
                    }

                    if ui
                        .button(colored_text("error", ToastLevel::Error.color()))
                        .clicked()
                    {
                        customize_toast(self.toasts.error(self.caption.clone()));
                    }

                    if ui.button("basic").clicked() {
                        customize_toast(self.toasts.basic(self.caption.clone()));
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
                closable: true,
                expires: true,
                show_progress_bar: true,
                duration: 3.5,
            })
        }),
    )
}
