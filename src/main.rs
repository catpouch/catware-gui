use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("CATware v0.1", native_options, Box::new(|_cc| Ok(Box::new(CatwareApp::default()))))
    // let _ = eframe::run_simple_native("CATware v0.1", native_options, update);
}

struct CatwareApp {
    input: String,
    history: Vec<String>,
}

impl Default for CatwareApp {
    fn default() -> Self {
        Self {
            input: "".to_owned(),
            history: vec![],
        }
    }
}

impl eframe::App for CatwareApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.history.join("\n"));
            ui.horizontal(|ui| {
                let prompt = ui.label("> ");
                let _input_box = ui.text_edit_multiline(&mut self.input).labelled_by(prompt.id);
            });
        });
    }
}