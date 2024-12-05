use eframe::egui::{self, text::{CCursor, CCursorRange}};
use egui::Key;
use evalexpr::*;
use std::collections::HashMap;
// use egui_plot::{Line, Plot, PlotPoints};

// const PLOT_RESOLUTION: usize = 100;

// please ignore all of the commented code, it was extremely rough and i know i have to find a different solution
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("CATware v0.1", native_options, Box::new(|_cc| Ok(Box::new(CatwareApp::default()))))
}

struct CatwareApp {
    input: String,
    history: Vec<(String, Result<Value, EvalexprError>)>,
    history_index: usize,
    // points: Vec<[f64; 2]>,
    context: HashMapContext,
}

impl Default for CatwareApp {
    fn default() -> Self {
        Self {
            input: "glorp".to_owned(),
            history: vec![],
            history_index: usize::MAX,
            // points: vec![],
            context: {
                let hardcoded_values: HashMap<&str, f64> = HashMap::from([
                    ("e", 2.7182818284590452353602874713527),
                    ("pi", 3.1415926535897932384626433832795)
                ]);
                let function_aliases: [&str; 24] = [
                    "is_nan",
                    "is_finite",
                    "is_infinite",
                    "is_normal",
                    "ln",
                    "log2",
                    "log10",
                    "exp",
                    "exp2",
                    "cos",
                    "acos",
                    "cosh",
                    "acosh",
                    "sin",
                    "asin",
                    "sinh",
                    "asinh",
                    "tan",
                    "atan",
                    "tanh",
                    "atanh",
                    "sqrt",
                    "cbrt",
                    "abs",
                ];
                let mut temp_context = HashMapContext::<DefaultNumericTypes>::new();
                for pair in hardcoded_values {
                    temp_context.set_value(pair.0.into(), Value::from_float(pair.1)).unwrap();
                }
                for alias in function_aliases {
                    temp_context.set_function(alias.into(), Function::new(|a| {
                        eval(("math::".to_string() + alias + "(" + a.str_from().as_str() + ")").as_str())
                    })).unwrap();
                }
                // temp_context.set_function("plot".to_owned(), Function::new(|a| {
                //     if !a.is_string() {
                //         Err(EvalexprError::expected_string(a.clone()))
                //     } else {
                //         // create new context where x exists, repeatedly evaluate for different values of x, create plot from that
                //         let mut plot_context = HashMapContext::<DefaultNumericTypes>::new();

                //         let f = match build_operator_tree::<DefaultNumericTypes>(a.as_string().unwrap().as_str()) {
                //             Ok(val) => val,
                //             Err(e) => return Err(e),
                //         };
                        
                //         let mut points: Vec<Value> = vec![Value::Tuple(vec![Value::from_float(f64::NAN), Value::from_float(f64::NAN)])];

                //         let range = (-10.0, 10.0);

                //         for i in 0..PLOT_RESOLUTION {
                //             let x_val = range.0 + (range.1 - range.0) * (i / PLOT_RESOLUTION).to_f64();
                //             plot_context.set_value("x".to_owned(), Value::from_float(x_val)).unwrap();
                //             points.push(Value::Tuple(vec![Value::from_float(x_val), f.eval_with_context_mut(&mut plot_context).unwrap()]));
                //         }
                //         Ok(Value::Tuple(points))
                //     }
                // })).unwrap();
                temp_context
            }
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
                format!("> {0}\n{1}\n", cmd, {
                    match result {
                        Ok(value) => format!("{value}"),
                        Err(e) => format!("{e}"),
                    }
                })
            }).fold(String::new(), |a,b| a + &b));
            // ui.label(self.history_index.to_string());
            ui.horizontal(|ui| {
                ui.label("> ");
                let input_box_widget = egui::TextEdit::singleline(&mut self.input);
                let mut input_box= input_box_widget.show(ui);
                if input_box.response.changed() && self.history_index != usize::MAX {
                    self.history[self.history_index].0 = self.input.clone();
                }

                ctx.input(|i| {
                    if i.key_pressed(Key::Enter) {
                        if self.history_index != usize::MAX {
                            self.history.truncate(self.history_index);
                        }
                        let result = eval_with_context_mut(&self.input, &mut self.context);

                        // this is the worst code i have ever written
                        // if result.unwrap().is_tuple() 
                        // && result.unwrap().as_tuple().unwrap()[0] == Value::Tuple(vec![Value::from_float(f64::NAN), Value::from_float(f64::NAN)])
                        // && result.unwrap().as_tuple().unwrap().len() == PLOT_RESOLUTION + 1 {
                        //     for point in result.unwrap().as_tuple().unwrap() {
                        //         let point_tuple = point.as_tuple().unwrap();
                        //         self.points.push([point_tuple[0].as_float().unwrap(), point_tuple[1].as_float().unwrap()]);
                        //     }
                        // } else {
                        self.history.push((self.input.clone(), result.clone()));
                        // }

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