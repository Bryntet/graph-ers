use std::collections::BTreeSet;
use eframe::egui;
use eframe::egui::{Key, Ui, Vec2, Vec2b};
use egui_autocomplete::AutoCompleteTextEdit;
use egui_plot::{Legend, Line, Plot, PlotBounds, PlotPoint, PlotPoints, Points};
use crate::helpers::random_data;
use crate::parse::Function;

#[derive(Default)]
pub struct GraphErBrain {
    graph_points: Vec<[f64; 2]>,
    input: AutoCompleteExample,
    zoom: Zoom,
    
}
#[derive(Default)]
enum Zoom {
    Increase,
    Decrease,
    #[default]
    Same
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
                
                ctx.input(|input| {
                    if input.key_pressed(Key::Plus) {
                        self.zoom = Zoom::Increase
                    } else if input.key_pressed(Key::Minus) {
                        self.zoom = Zoom::Decrease
                    }
                });
                let min_x = plot_ui.plot_bounds().min()[0];
                let max_x = plot_ui.plot_bounds().max()[0];
                /*let sin: PlotPoints = (0..1000).map(|i| {
                    let x = i as f64 * 0.01;
                    [x, x.sin()]
                }).collect();*/
                plot_ui.set_auto_bounds(Vec2b::new(false,false));
                let test = plot_ui.plot_bounds().max();
                let test1 = plot_ui.plot_bounds().min();
                
                let zoom_factor = match self.zoom {
                    Zoom::Increase => Vec2::new(2.,2.),
                    Zoom::Decrease => Vec2::new(0.5,0.5),
                    Zoom::Same => Vec2::new(1.,1.)
                };
                plot_ui.zoom_bounds(zoom_factor, PlotPoint::new((test1[0] + test[0]) / 2., (test1[1] + test[1]) / 2.));
                self.zoom = Zoom::Same;
                    
                
                plot_ui.line(Line::new(Function::new(1.,2.).into_plot_points(min_x+0.001,max_x-0.001)).name("Test"));
                //plot_ui.line(Line::new(sin).name("A"))
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