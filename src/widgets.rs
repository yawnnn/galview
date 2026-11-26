use egui::{Color32, RichText, Ui};
use std::time::Instant;

#[derive(Clone)]
pub struct Confirm {
    title: String,
    text: String,
    pub curr: bool,
}

impl Confirm {
    pub fn new(
        title: impl Into<String>,
        text: impl Into<String>,
        start: bool,
    ) -> Self {
        Confirm {
            title: title.into(),
            text: text.into(),
            curr: start,
        }
    }

    pub fn toggle(&mut self) {
        self.curr = !self.curr;
    }

    pub fn draw(&self, ui: &mut Ui) {
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(&self.text);
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        let yes = ui.button("Yes");
                        let no = ui.button("No");
                        if self.curr {
                            yes.request_focus();
                        } else {
                            no.request_focus();
                        }
                    });
                });
            });
    }
}

pub struct TextBox {
    label: String,
    pub text: String,
    lowercase: bool,
}

impl TextBox {
    pub fn new(
        label: impl Into<String>,
        initial: impl Into<String>,
        lowercase: bool,
    ) -> Self {
        TextBox {
            label: label.into(),
            text: initial.into(),
            lowercase,
        }
    }

    pub fn draw(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        egui::Window::new("")
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::LEFT_BOTTOM, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.label)
                            .text_style(egui::TextStyle::Heading),
                    );
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.text)
                            .desired_width(300.0),
                    );

                    response.request_focus();
                    changed = response.changed();
                    if changed && self.lowercase {
                        self.text = self.text.to_lowercase();
                    }
                });
            });

        changed
    }
}

pub struct Toast {
    title: String,
    text: String,
    ctime: Instant,
    duration_secs: f32,
}

impl Toast {
    pub fn new(title: impl Into<String>, text: impl Into<String>, duration_secs: f32) -> Self {
        Toast {
            title: title.into(),
            text: text.into(),
            ctime: Instant::now(),
            duration_secs,
        }
    }

    fn opacity(&self) -> Option<f32> {
        let elapsed = self.ctime.elapsed().as_secs_f32();
        if elapsed >= self.duration_secs {
            None
        } else {
            let fade_start = self.duration_secs - 0.3;
            if elapsed > fade_start {
                Some(1.0 - (elapsed - fade_start) / 0.3)
            } else {
                Some(1.0)
            }
        }
    }

    pub fn draw(&self, ui: &mut Ui) -> bool {
        let Some(opacity) = self.opacity() else {
            return false;
        };

        ui.request_repaint();
        let alpha = (opacity * 255.0) as u8;

        egui::Window::new(&self.title)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(40, 40, 40, alpha)),
            )
            .show(ui, |ui| {
                ui.label(RichText::new(&self.text).color(
                    Color32::from_rgba_unmultiplied(200, 200, 200, alpha),
                ));
            });

        true
    }
}
