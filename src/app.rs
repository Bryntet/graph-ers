use std::collections::BTreeSet;
use eframe::egui;
use eframe::egui::{ Ui};
use egui_autocomplete::AutoCompleteTextEdit;
use egui_plot::{Legend, Plot, PlotPoints, Points};
use crate::helpers::random_data;

#[derive(Default)]
pub struct GraphErBrain {
    graph_points: Vec<[f64; 2]>,
    input: AutoCompleteExample
}

impl GraphErBrain {
    pub fn new() -> Self {
        Self {
            graph_points: random_data(),
            ..Default::default()
        }
    }
    pub fn start() -> eframe::Result<()> {
        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1280., 720.]),
            ..Default::default()
        };

        eframe::run_native(
            "Graph-ers",
            options,
            Box::new(|_cc| Box::new(GraphErBrain::new())),
        )
    }
}

impl eframe::App for GraphErBrain {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut plot_rect = None;

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Enter your text:");

            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::top_down(egui::Align::Max),
                |ui| {
                    self.input.update(ctx, ui, true);
                },
            );
        });
        
        

        egui::CentralPanel::default().show(ctx, |ui| {
            let my_plot = Plot::new("Grafen!").legend(Legend::default());

            let inner = my_plot.show(ui, |plot_ui| {
                plot_ui.points(Points::new(PlotPoints::from(self.graph_points.clone())).name("curve"));
            });

            // Remember the position of the plot
            plot_rect = Some(inner.response.rect);
        });
    }
}

struct AutoCompleteExample {
    multi_input: String,
    search_field: String,
    max_suggestions:usize,
    result: f64
}

impl Default for AutoCompleteExample {
    fn default() -> Self {
        Self {multi_input: STARTER_LIST.to_string(), search_field: "".to_string(), max_suggestions: 10,result:0.}
    }
}

impl AutoCompleteExample {
    fn update(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut Ui,
        highlight_matches: bool,
    ) {
        let inputs = self.multi_input.lines().collect::<BTreeSet<_>>();
        self.search_field=self.search_field.chars().filter(|c|c.is_ascii_digit() || c.is_whitespace() || ['-','+','/','*','(',')'].contains(c)).collect();
        ui.add(
            AutoCompleteTextEdit::new(&mut self.search_field, inputs)
                .max_suggestions(self.max_suggestions)
                .highlight_matches(highlight_matches),
        );
        let operation_queue = crate::parse::TokenQueue::new(&self.search_field);
        if let Some(result) = operation_queue.calculate() {
            self.result = result
        }
        ui.separator();
        // Display the result next to the input field
        ui.label(format!("Result: {}", self.result));
    }
}

const STARTER_LIST: &str = r#"test
"#;