use crate::helpers::random_data;
use crate::parse::Function;
use eframe::egui;
use eframe::egui::{Color32, Key, RichText, Ui, Vec2, Vec2b, WidgetText};
use egui_autocomplete::AutoCompleteTextEdit;
use egui_plot::{Legend, Line, Plot, PlotBounds, PlotPoint, PlotPoints, Points};
use std::collections::{BTreeSet, HashMap};

#[derive(Default)]
pub struct GraphErBrain {
    input: AutoCompleteExample,
    zoom: Zoom,
    text_focused: bool,
    function_thing: String,
    function_error: Option<String>,
}

#[derive(Default)]
enum Zoom {
    Increase,
    Decrease,
    #[default]
    Same,
}

impl GraphErBrain {
    pub fn new() -> Self {
        Self {
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

        let mut space_to_the_left_of_graph = 0.;

        egui::SidePanel::left("math_input").show(ctx, |ui| {
            ui.label("Enter your text:");
            space_to_the_left_of_graph = ui.available_width();
            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::top_down(egui::Align::Max),
                |ui| {
                    self.input.update(ctx, ui, true);
                    ui.text_edit_singleline(&mut self.function_thing);
                    if let Some(error) = &self.function_error {
                        ui.label(RichText::new(error).color(Color32::RED));
                    }
                },
            );
        });
        let latest_pointer_x_pos = ctx.pointer_latest_pos().unwrap_or_default().x;

        let is_right_of_math_input = space_to_the_left_of_graph < latest_pointer_x_pos;
        egui::CentralPanel::default().show(ctx, |ui| {
            let my_plot = Plot::new("Grafen!").legend(Legend::default());

            let inner = my_plot.show(ui, |plot_ui| {
                if is_right_of_math_input {
                    ctx.input(|input| {
                        if input.key_pressed(Key::Plus) {
                            self.zoom = Zoom::Increase
                        } else if input.key_pressed(Key::Minus) {
                            self.zoom = Zoom::Decrease
                        }
                    })
                }
                let min_x = plot_ui.plot_bounds().min()[0];
                let max_x = plot_ui.plot_bounds().max()[0];
                /*let sin: PlotPoints = (0..1000).map(|i| {
                    let x = i as f64 * 0.01;
                    [x, x.sin()]
                }).collect();*/
                plot_ui.set_auto_bounds(Vec2b::new(false, true));
                let test = plot_ui.plot_bounds().max();
                let test1 = plot_ui.plot_bounds().min();

                let zoom_factor = match self.zoom {
                    Zoom::Increase => Vec2::new(2., 2.),
                    Zoom::Decrease => Vec2::new(0.5, 0.5),
                    Zoom::Same => Vec2::new(1., 1.),
                };
                plot_ui.zoom_bounds(
                    zoom_factor,
                    PlotPoint::new((test1[0] + test[0]) / 2., (test1[1] + test[1]) / 2.),
                );
                self.zoom = Zoom::Same;

                match Function::try_from(self.function_thing.trim().to_lowercase()) {
                    Ok(mut func) => {
                        println!("{}",func.internal_representation());
                        match func.plot_points(min_x + 0.001, max_x - 0.001) {
                            Ok(points) => {
                                plot_ui.line(Line::new(points).name(func.name));
                                self.function_error = None;
                            }
                            Err(e) => self.function_error = Some(e.to_string())
                        }
                    },
                    Err(e) => self.function_error = Some(e.to_string()),
                }
            });

            // Remember the position of the plot
            plot_rect = Some(inner.response.rect);
        });
    }
}

struct AutoCompleteExample {
    multi_input: String,
    search_field: String,
    max_suggestions: usize,
    result: f64,
    error: Option<String>,
}

impl Default for AutoCompleteExample {
    fn default() -> Self {
        Self {
            multi_input: STARTER_LIST.to_string(),
            search_field: "".to_string(),
            max_suggestions: 10,
            result: 0.,
            error: None,
        }
    }
}

impl AutoCompleteExample {
    fn update(&mut self, _ctx: &egui::Context, ui: &mut Ui, highlight_matches: bool) {
        let inputs = self.multi_input.lines().collect::<BTreeSet<_>>();
        self.search_field = self
            .search_field
            .chars()
            .filter(|c| {
                c.is_ascii_digit()
                    || c.is_whitespace()
                    || ['-', '+', '/', '*', '(', ')'].contains(c)
            })
            .collect();
        ui.add(
            AutoCompleteTextEdit::new(&mut self.search_field, inputs)
                .max_suggestions(self.max_suggestions)
                .highlight_matches(highlight_matches),
        );
        match crate::parse::TokenQueue::new(&self.search_field, &vec![]) {
            Ok(operation_queue) => {
                if let Ok(result) = operation_queue.calculate(&HashMap::<String, f64>::new()) {
                    self.result = result;
                    self.error = None;
                }
            }
            Err(e) => self.error = Some(e.to_string()),
        }

        ui.separator();
        // Display the result next to the input field
        ui.label(format!("Result: {}", self.result));
    }
}

const STARTER_LIST: &str = r#"test
"#;
