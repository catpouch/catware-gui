use eframe::egui::{self, text::{CCursor, CCursorRange}};
use egui::Key;
use crate::parser::CatwareCalc;

pub mod parser;
// use egui_plot::{Line, Plot, PlotPoints};

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("CATware v0.1", native_options, Box::new(|cc| Ok(Box::new(CatwareApp::new(cc)))))
}

struct CatwareApp {
    input: String,
    history: Vec<(String, f64)>,
    history_index: usize,
    parser: CatwareCalc,
}

impl CatwareApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.visuals.extreme_bg_color = style.visuals.faint_bg_color;
        });

        Self {
            input: "glorp".to_owned(),
            history: vec![],
            history_index: usize::MAX,
            parser: CatwareCalc::new()
        }
    }
}

impl eframe::App for CatwareApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // egui::SidePanel::right("graph_panel").show(ctx, |ui| {
        //     Plot::new("main_plot").show(ui, |plot_ui| plot_ui.line(Line::new(PlotPoints::new(self.points.clone())))); // FIX THIS
        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.history.clone().into_iter().map(|(cmd, result)| {
                format!("> {0}\n{1}\n", cmd, result)
            }).fold(String::new(), |a,b| a + &b));

            ui.horizontal(|ui| {
                ui.label(">");
                let input_box_widget = egui::TextEdit::singleline(&mut self.input).frame(false);
                let mut input_box= input_box_widget.show(ui);
                if input_box.response.changed() && self.history_index != usize::MAX {
                    self.history[self.history_index].0 = self.input.clone();
                }

                ctx.input(|i| {
                    if i.key_pressed(Key::Enter) {
                        if self.history_index != usize::MAX {
                            self.history.truncate(self.history_index);
                        }

                        let result = self.parser.parse_string(&self.input);

                        if result.is_some() {
                            self.history.push((self.input.clone(), result.unwrap()));
                        }

                        self.history_index = usize::MAX;

                        self.input.clear();
                    }
    
                    if i.key_pressed(Key::ArrowUp) {
                        if self.history_index == usize::MAX {
                            self.history_index = self.history.len();
                        }
                        if self.history_index > 0 {
                            self.history_index -= 1;
                            self.input = self.history[self.history_index].0.clone();
                        }

                        input_box.state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(self.input.len()))));
                    }
    
                    if i.key_pressed(Key::ArrowDown) {
                        if self.history_index == self.history.len() - 1 {
                            self.history_index = usize::MAX;
                            self.input = "".to_owned();
                        } else if self.history_index != usize::MAX {
                            self.history_index += 1;
                            self.input = self.history[self.history_index].0.clone();
                        }

                        input_box.state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(self.input.len()))));
                    }
                });

                input_box.state.store(ui.ctx(), input_box.response.id);
                
                input_box.response.request_focus();
            });
        });
    }
}