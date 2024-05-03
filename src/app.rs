use crate::parse::{Function, ParseError};
use eframe::{egui, Theme};
use eframe::egui::{Color32, Key, RichText, Vec2, Vec2b};
use egui_plot::{Legend, Line, Plot, PlotPoint, PlotPoints};

#[derive(Default)]
pub struct GraphErBrain {
    zoom: Zoom,
    function_thing: Vec<FunctionInput>,
}
#[derive(Default)]
struct FunctionInput(String);


impl FunctionInput {
    fn func(&self) -> Result<Function, ParseError> {
        Function::try_from(self.0.clone())
    }

    fn points(&self, minimum_x:f64,maximum_x:f64) -> Result<PlotPoints, ParseError> {
        self.func()?.plot_points(minimum_x,maximum_x)
    }

    fn err(&self) -> Option<ParseError> {
        // Check on calculations instead of function to catch any additional errors that may occur during later parsing.
        match self.func() {
            Err(e) => Some(e),
            Ok(f) => {
                if let Err(e) = f.y_pos(&f.generate_naive_map()) {
                    Some(e)
                } else {
                    None
                }

            }
        }

    }

    fn name(&self) -> Result<String, ParseError> {
        Ok(self.func()?.name)
    }

}
#[derive(Default)]
enum Zoom {
    Increase,
    Decrease,
    Same,
    #[default]
    InitialState
}

impl GraphErBrain {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start() -> eframe::Result<()> {
        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280., 720.])
                .with_title("Graph-ers - The oxidized geogebra replacement")
                .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../icon.png")).expect("Is valid png.")),
            default_theme: Theme::Dark,
            ..Default::default()
        };

        eframe::run_native(
            "Graph-ers",
            options,
            Box::new(|_cc| Box::new(GraphErBrain::new())),
        )
    }

    // Optional branch that gets followed if built for wasm target
    #[cfg(target_arch = "wasm32")]
    pub fn start() {
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();
        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "the_canvas_id", // hardcode it
                    web_options,
                    Box::new(|_cc| Box::new(GraphErBrain::new())),
                )
                .await
                .expect("failed to start eframe");
        });
    }
}

impl eframe::App for GraphErBrain {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Persistence for the plot
        let mut plot_rect = None;

        let mut space_to_the_left_of_graph = 0.;
        
        
        egui::SidePanel::left("math_input").show(ctx, |ui| {
            ctx.set_zoom_factor(1.3);
            
            ui.label("Enter your text:");
            space_to_the_left_of_graph = ui.available_width();
            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::top_down(egui::Align::Max),
                |ui| {
                    for (i, func_input) in self.function_thing.iter_mut().enumerate() {
                        // If not first iteration, add separator above current.
                        if i != 0 {
                            ui.separator();
                        }
                        ui.text_edit_singleline(&mut func_input.0);
                        if let Some(error) = &func_input.err() {
                            match error {
                                ParseError::NoFunctionDefined => (),
                                _ => {
                                    ui.label(RichText::new(error.to_string()).color(Color32::RED));
                                }
                            }
                        }
                    }

                },
            );
        });


        let latest_pointer_x_pos = ctx.pointer_latest_pos().unwrap_or_default().x;
        let is_right_of_math_input = space_to_the_left_of_graph < latest_pointer_x_pos;

        egui::CentralPanel::default().show(ctx, |ui| {
            let my_plot = Plot::new("Main graph area").legend(Legend::default());

            let plot_response = my_plot.show(ui, |plot_ui| {
                // Logic for if zoom keys should zoom the graph
                if is_right_of_math_input {
                    ctx.input(|input| {
                        if input.key_pressed(Key::Plus) {
                            self.zoom = Zoom::Increase
                        } else if input.key_pressed(Key::Minus) {
                            self.zoom = Zoom::Decrease
                        }
                    })
                }

                // Don't automatically zoom based on the graphs because
                // otherwise it will go crazy because of the bounds
                plot_ui.set_auto_bounds(Vec2b::new(false, true));

                // Minimum and maximum bounds used for knowing which range is actually needed to calculate
                let minimum_bound = plot_ui.plot_bounds().min();
                let maximum_bound = plot_ui.plot_bounds().max();

                let zoom_factor = match self.zoom {
                    Zoom::Increase => Vec2::new(2., 2.),
                    Zoom::Decrease => Vec2::new(0.5, 0.5),
                    Zoom::Same => Vec2::new(1., 1.),
                    Zoom::InitialState => Vec2::new(0.25, 0.25),
                };

                plot_ui.zoom_bounds(
                    zoom_factor,
                    PlotPoint::new((minimum_bound[0] + maximum_bound[0]) / 2., (minimum_bound[1] + maximum_bound[1]) / 2.),
                );
                self.zoom = Zoom::Same;

                for func in &mut self.function_thing {
                    // Ignore errors since that's handled elsewhere
                    if let Ok(points) = func.points(minimum_bound[0], maximum_bound[1]) {
                        plot_ui.line(Line::new(points).name(func.name().expect("Func already valid since points was ok")));
                    }
                }
                if self.function_thing.iter().all(|f|!f.0.is_empty() && f.err().is_none()) {
                    // All have text and none have errors (because it indicates usage),
                    // so add an empty text box
                    self.function_thing.push(FunctionInput::default());
                }
            });

            // Remember the position of the plot
            plot_rect = Some(plot_response.response.rect);
        });
    }
}