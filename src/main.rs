use eframe::egui::{self, text::{CCursor, CCursorRange}};
use egui::Key;
use crate::parser::CatwareCalc;
use egui_plot::{Line, Plot, PlotPoints};

pub mod parser;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("CATware v0.1", native_options, Box::new(|cc| Ok(Box::new(CatwareApp::new(cc)))))
}

struct CatwareApp {
    input: String,
    history: Vec<(String, Result<f64, Box<dyn std::error::Error>>)>,
    history_index: usize,
    parser: CatwareCalc,
    plot_shown: bool
}

impl CatwareApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let mut fonts = FontDefinitions::default();
        // fonts.font_data.insert("Fira Code".to_owned(), 
        // std::sync::Arc::new(
        //     // .ttf and .otf supported
        //     FontData::from_static(include_bytes!("../../../epaint_default_fonts/fonts/Ubuntu-Light.ttf"))
        // ));

        // cc.egui_ctx.set_fonts(font_definitions);

        cc.egui_ctx.style_mut(|style| {
            style.visuals.extreme_bg_color = style.visuals.faint_bg_color;
        });

        Self {
            input: "".to_owned(),
            history: vec![],
            history_index: usize::MAX,
            parser: CatwareCalc::new(),
            plot_shown: false
        }
    }
}

impl eframe::App for CatwareApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.plot_shown {
                let plot = Plot::new("main_plot")
                    .allow_drag(false)
                    .allow_scroll(false)
                    .allow_boxed_zoom(false)
                    .auto_bounds(false.into())
                    .include_x(-10.0)
                    .include_x(10.0)
                    .include_y(-10.0)
                    .include_y(10.0)
                    .show_x(false)
                    .show_y(false);

                plot.show(ui, |plot_ui| {
                    plot_ui.line(Line::new(PlotPoints::new(self.parser.plot_points.borrow().to_vec())));

                    ctx.input(|i| {
                        if i.key_down(Key::ArrowLeft) {
                            let x_travel = (plot_ui.plot_bounds().width() * -0.03) as f32;
                            plot_ui.translate_bounds((x_travel, 0.).into());
                            let _ = self.parser.refresh_graph(plot_ui.plot_bounds());
                        }
                        if i.key_down(Key::ArrowRight) {
                            let x_travel = (plot_ui.plot_bounds().width() * 0.03) as f32;
                            plot_ui.translate_bounds((x_travel, 0.).into());
                            let _ = self.parser.refresh_graph(plot_ui.plot_bounds());
                        }
                        if i.key_down(Key::ArrowUp) {
                            let y_travel = (plot_ui.plot_bounds().width() * 0.03) as f32;
                            plot_ui.translate_bounds((0., y_travel).into());
                            let _ = self.parser.refresh_graph(plot_ui.plot_bounds());
                        }
                        if i.key_down(Key::ArrowDown) {
                            let y_travel = (plot_ui.plot_bounds().width() * -0.03) as f32;
                            plot_ui.translate_bounds((0., y_travel).into());
                            let _ = self.parser.refresh_graph(plot_ui.plot_bounds());
                        }
                    });
                });
            }

            if !self.plot_shown {
                ui.label("BATTERY % HERE");
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    ui.label(self.history.iter().map(|(cmd, result)| {
                        format!("> {0}\n{1}\n", cmd, match result {
                            Ok(v) => {
                                v.to_string()
                            }
                            Err(e) => {
                                e.to_string()
                            }
                        })
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
        
                                // i know this is bad. i'm going insane
                                if self.input.starts_with("plot") {
                                    self.plot_shown = true;
                                }
        
                                let result = self.parser.eval_string(&self.input);
    
                                self.history.push((self.input.clone(), result));
        
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

            ctx.input(|i| {
                if i.key_pressed(Key::F7) {
                    self.plot_shown = !self.plot_shown;
                }

                if i.key_pressed(Key::Escape) {
                    if self.plot_shown {
                        self.plot_shown = false;
                    }
                }

                if i.key_pressed(Key::F4) {
                    self.input = "".to_owned();
                }
            });
        });
    }
}