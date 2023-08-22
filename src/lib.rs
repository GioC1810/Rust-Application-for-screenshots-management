use std::fs;
use druid::widget::{Button, FillStrat, Flex, Image, Label};
use druid::{AppLauncher, Point, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, LocalizedString, MouseButton, MouseEvent, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget, WidgetExt, WindowDesc, ImageBuf, WindowState};
use druid::piet::ImageFormat;
use druid::WindowState::Maximized;
use screenshots::{Compression, Screen};

//principal structs

pub struct MyApp;
#[derive(Clone, Data)]
pub struct AppState{
    pub mouse_position: Point,
    pub initial_point:Option<Point>,
    pub final_point:Option<Point>,
    #[data(same_fn = "rectangles_equal")]
    pub rectangles: Vec<ExpandableRect>,
    pub current_rectangle: Option<ExpandableRect>,
}

impl Widget<AppState> for MyApp {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseMove(mouse_event) => {
                data.mouse_position = mouse_event.pos;
                ctx.request_paint(); // Request a redraw
                if let Some(rect) = &mut data.current_rectangle {
                    rect.update(mouse_event.pos);
                    ctx.request_paint();
                }
            }
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    data.initial_point=Some(data.mouse_position);
                    ctx.request_paint(); // Request a redraw
                    println!("during click {}", mouse_event.pos.x)
                }
                let expandable_rect = ExpandableRect::new(mouse_event.pos);
                data.current_rectangle = Some(expandable_rect);
                ctx.request_paint();
            }
            Event::MouseUp(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    data.final_point=Some(data.mouse_position);
                    ctx.request_paint(); // Request a redraw
                    println!("after click {}", mouse_event.pos.x);
                    if let Some(rect) = data.current_rectangle.take() {
                        data.rectangles.push(rect);
                        ctx.request_paint();
                    }
                }
            }
            _ => (),
        }
        if data.initial_point.is_some() && data.final_point.is_some(){
            let event_loop = winit::event_loop::EventLoop::new();
            // Get the list of available monitors
            let mut monitors = event_loop.available_monitors();
            // Find the primary monitor (the one with the taskbar/menu bar)
            let primary_monitor = monitors
                .next()
                .unwrap(); // You might want to add error handling here
            // Get the scale factor of the primary monitor
            let scale_factor = primary_monitor.scale_factor();
            println!("Fattore di scala: {}", scale_factor);


            let screenshot_width=data.final_point.unwrap().x-data.initial_point.unwrap().x ;
            let screenshot_height=data.final_point.unwrap().y-data.initial_point.unwrap().y ;

            //-----TEST ------

            let source_scale = 1.0;
            let target_scale = 1.0;

            // Coordinates in source scale
            let source_coordinates = (data.initial_point.unwrap().x, data.initial_point.unwrap().y);
            let source_dim = (screenshot_width, screenshot_height);

            // Convert coordinates to target scale
            let converted_coordinates = convert_coordinates(source_coordinates, source_scale, target_scale);
            let converted_dim = convert_coordinates(source_dim, source_scale, target_scale);
            let image=Screen::from_point(0,0).unwrap().capture_area(converted_coordinates.0 as i32, converted_coordinates.1 as i32,converted_dim.0 as u32, converted_dim.1 as u32).unwrap();

            //let image=Screen::from_point(0,0).unwrap().capture_area(data.initial_point.unwrap().x as i32, data.initial_point.unwrap().y as i32,screenshot_width as u32, screenshot_height as u32).unwrap();
            let buffer = image.to_png(Compression::Default).unwrap();
            //let compressed_buffer = image.to_png(Compression::Best).unwrap();

            fs::write("test_screen.png", buffer).unwrap();
            ctx.new_window(WindowDesc::new(build_ui(image)).window_size((screenshot_width,screenshot_height)));
            ctx.window().close();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) { }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {   }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppState, env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        for expandable_rect in &data.rectangles {
            ctx.fill(expandable_rect.rect, &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Transparent background
            ctx.stroke(expandable_rect.rect, &Color::WHITE, 2.0); // White border
        }

        if let Some(expandable_rect) = &data.current_rectangle {
            ctx.fill(expandable_rect.rect, &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Transparent background
            ctx.stroke(expandable_rect.rect, &Color::WHITE, 2.0); // White border
        }
    }
}

//ui generation functions

pub fn ui_builder() -> impl Widget<AppState> {
    let screen_button = Button::new("Screen")
        .on_click(|ctx, _data, _env| {
            ctx.new_window(WindowDesc::new(MyApp)
                .set_window_state(Maximized)
                .set_position(Point::new(0 as f64, 0 as f64))
                .show_titlebar(true)
                .transparent(true)
            );
            ctx.window().close();
        });
    Flex::column()
        .with_child(screen_button)
}

fn build_ui(image:screenshots::Image) -> impl Widget<AppState> {
    //let screen = Screen::from_point(0, 0).unwrap();

    //let image = screen.capture().unwrap();
    let image_buf=ImageBuf::from_raw(image.rgba().clone(),ImageFormat::RgbaPremul,image.width() as usize,image.height() as usize);
    /*let save_png = Button::new("Save as png")
        .on_click(|_ctx, _data, _env| {});*/
    let image:Image=Image::new(image_buf).fill_mode(FillStrat::Fill); // Adjust aspect ratio as needed;
    //Flex::row().with_child(save_png).with_child(image)
    Flex::row().with_child(image)

}

//Rectangle drawer section
#[derive(Clone)]
pub struct ExpandableRect {
    rect: Rect,
}

impl ExpandableRect {
    fn new(origin: Point) -> Self {
        ExpandableRect {
            rect: Rect::from_origin_size(origin, Size::ZERO),
        }
    }
    fn update(&mut self, new_point: Point) {
        let width = new_point.x - self.rect.origin().x;
        let height = new_point.y - self.rect.origin().y;
        let size = Size::new(width, height);
        self.rect = Rect::from_origin_size(self.rect.origin(), size);
    }
}

impl Data for ExpandableRect {
    fn same(&self, other: &Self) -> bool {
        self.rect.same(&other.rect)
    }
}

fn rectangles_equal(r1: &Vec<ExpandableRect>, r2: &Vec<ExpandableRect>) -> bool {
    r1.len() == r2.len() && r1.iter().zip(r2).all(|(a, b)| a.same(b))
}
struct RectangleDrawer;

//utility functions

fn convert_coordinates(coord: (f64, f64), source_scale: f64, target_scale: f64) -> (f64, f64) {
    let (x, y) = coord;
    let converted_x = x * (target_scale / source_scale);
    let converted_y = y * (target_scale / source_scale);
    (converted_x, converted_y)
}