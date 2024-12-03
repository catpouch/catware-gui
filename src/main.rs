use eframe::egui::{self, text::CCursorRange, text::CCursor};
use egui::Key;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("CATware v0.1", native_options, Box::new(|_cc| Ok(Box::new(CatwareApp::default()))))
}

struct CatwareApp {
    input: String,
    // input_box: egui::Response,
    history: Vec<String>,
    history_index: usize,
}

impl Default for CatwareApp {
    fn default() -> Self {
        Self {
            input: "glorp".to_owned(),
            // input_box: egui::Response::from(()),
            history: vec![],
            history_index: usize::MAX,
        }
    }
}

impl eframe::App for CatwareApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.history.join("\n"));
            ui.label(self.history_index.to_string());
            ui.horizontal(|ui| {
                ui.label("> ");
                let input_box_widget = egui::TextEdit::singleline(&mut self.input);
                let mut input_box= input_box_widget.show(ui);
                if input_box.response.changed() && self.history_index != usize::MAX {
                    self.history[self.history_index] = self.input.clone();
                }

                let mut cursor_end = false;

                ctx.input(|i| {
                    if i.key_pressed(Key::Enter) {
                        if self.history_index != usize::MAX {
                            self.history.truncate(self.history_index + 1);
                        } else {
                            self.history.push(self.input.clone());
                        }
                        self.history_index = usize::MAX;
                        self.input.clear();
                    }
    
                    if i.key_pressed(Key::ArrowUp) && self.history_index > 0 {
                        if self.history_index == usize::MAX {
                            self.history_index = self.history.len();
                        }
                        self.history_index -= 1;
                        self.input = self.history[self.history_index].clone();

                        cursor_end = true;
                    }
    
                    if i.key_pressed(Key::ArrowDown) {
                        if self.history_index == self.history.len() - 1 {
                            self.history_index = usize::MAX;
                            self.input = "".to_owned();
                        } else {
                            self.history_index += 1;
                            self.input = self.history[self.history_index].clone();
                        }

                        cursor_end = true;
                    }
                });

                // this is super hacky because ctx.input locks the context so i can't do it there
                // TODO: try storing the state outside of the ctx.input and set the cursor inside it so there's no conditional weirdness
                if cursor_end {
                    input_box.state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(self.input.len()))));
                    input_box.state.store(ui.ctx(), input_box.response.id);
                }
            });
        });
    }
}