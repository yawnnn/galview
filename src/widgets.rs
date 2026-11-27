use egui::{
    Align2, Color32, RichText, TextEdit, TextStyle, Ui,
    text::{CCursor, CCursorRange},
};
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
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
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
    pub text: String,
    title: String,
    label: String,
    lowercase: bool,
    align: Align2,
    movable: bool,
    start_selected: bool,
    anchor_frames: u8,
    width: f32,
}

impl Default for TextBox {
    fn default() -> Self {
        TextBox {
            title: String::new(),
            label: String::new(),
            text: String::new(),
            lowercase: false,
            align: Align2::LEFT_BOTTOM,
            movable: false,
            start_selected: false,
            anchor_frames: 2,
            width: 300.0,
        }
    }
}

impl TextBox {
    pub fn new(text: impl Into<String>) -> Self {
        TextBox {
            text: text.into(),
            ..Self::default()
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn lowercase(mut self, lowercase: bool) -> Self {
        self.lowercase = lowercase;
        self
    }

    /// Set alignment. If `movable`, this is the initial position; otherwise it's the anchor.
    pub fn align(mut self, align: Align2) -> Self {
        self.align = align;
        self
    }

    pub fn movable(mut self, movable: bool) -> Self {
        self.movable = movable;
        self
    }

    pub fn start_selected(mut self, start_selected: bool) -> Self {
        self.start_selected = start_selected;
        self
    }

    pub fn draw(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        let has_title = !self.title.is_empty();

        let mut window = egui::Window::new(&self.title)
            .title_bar(has_title)
            .resizable(false)
            .collapsible(false);

        if self.movable && self.anchor_frames > 0 {
            // frames 0..2: anchor to force position (need 2 frames for size to settle)
            window = window.anchor(self.align, [0.0, 0.0]).movable(false);
            self.anchor_frames -= 1;
        } else if self.movable {
            // frames > 2: movable, no anchor
            window = window.movable(true);
        } else {
            // not movable: always anchored
            window = window.anchor(self.align, [0.0, 0.0]).movable(false);
        }

        window.show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if !self.label.is_empty() {
                    ui.label(
                        RichText::new(&self.label)
                            .text_style(TextStyle::Heading),
                    );
                }

                let response = ui.add(
                    TextEdit::singleline(&mut self.text)
                        .desired_width(self.width),
                );

                if self.start_selected {
                    if let Some(mut state) =
                        TextEdit::load_state(ui.ctx(), response.id)
                    {
                        state.cursor.set_char_range(Some(CCursorRange::two(
                            CCursor::new(0),
                            CCursor::new(self.text.chars().count()),
                        )));
                        state.store(ui.ctx(), response.id);
                    }
                    self.start_selected = false;
                }

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
    pub fn new(
        title: impl Into<String>,
        text: impl Into<String>,
        duration_secs: f32,
    ) -> Self {
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
