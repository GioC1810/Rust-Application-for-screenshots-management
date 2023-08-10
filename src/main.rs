use std::fs;
use druid::widget::{Button, FillStrat, Flex, Image, Label};
use druid::{AppLauncher, BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, LocalizedString, MouseButton, PaintCtx, Point, Size, UpdateCtx, Widget, WindowDesc};
use druid::{Color, PlatformError, ImageBuf};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;
use image::io::Reader as ImageReader;
use screenshots::{Compression, Screen};
use druid::piet::ImageFormat;
use druid::Data;


fn make_screen() -> screenshots::Image{
    println!("inside the func");
    //window.hide();
    //window.set_window_state(Minimized);
    let screen = Screen::from_point(100, 100).unwrap();

    let image = screen.capture().unwrap();
    println!("after screen");
    let buffer = image.to_png(Compression::Default).unwrap();
    //let compressed_buffer = image.to_png(Compression::Best).unwrap();

    fs::write("capture_display_with_point.png", buffer).unwrap();
    //fs::write("capture_display_with_point.png", compressed_buffer).unwrap();
    return image;
}
fn ui_builder() -> impl Widget<AppState> {

    let do_screen=Button::new("Screen").on_click(|ctx,_data,_env|  {
        ctx.window().hide();
        let image=make_screen();
        //ctx.new_window(WindowDesc::new(build_ui(image)));
        let screens=Screen::all().unwrap();
        for screen in screens{
            let monitor_height= screen.display_info.height as f64;
            let monitor_width= screen.display_info.width as f64;
            ctx.new_window(WindowDesc::new(MyApp).transparent(true).window_size(Size::new(monitor_width, monitor_height)).resizable(false));
             ctx.window().close();}
    });

    Flex::column().with_child(do_screen)

}
struct MyApp;
#[derive(Clone, Data)]
struct AppState {
    mouse_position: Point,
    initial_point:Option<Point>,
    final_point:Option<Point>

    // Other state variables...
}
impl Widget<AppState> for MyApp {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseMove(mouse_event) => {
                // Update the mouse position in the app state
                data.mouse_position = mouse_event.pos;
                ctx.request_paint(); // Request a redraw
            }
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    data.initial_point=Some(data.mouse_position);
                    ctx.request_paint(); // Request a redraw
                    println!("during click {}", mouse_event.pos.x)
                }
            }
            Event::MouseUp(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    data.final_point=Some(data.mouse_position);
                    ctx.request_paint(); // Request a redraw
                    println!("after click {}", mouse_event.pos.x)
                }
            }
            _ => (),
        }
        if data.initial_point.is_some() && data.final_point.is_some(){
            let screenshot_width=data.final_point.unwrap().x-data.initial_point.unwrap().x ;
            let screenshot_height=data.final_point.unwrap().y-data.initial_point.unwrap().y ;
            let image=Screen::from_point(0,0).unwrap().capture_area(data.initial_point.unwrap().x as i32, data.initial_point.unwrap().y as i32,screenshot_width as u32, screenshot_height as u32).unwrap();
            let buffer = image.to_png(Compression::Default).unwrap();
            //let compressed_buffer = image.to_png(Compression::Best).unwrap();

            fs::write("test_screen.png", buffer).unwrap();
            ctx.new_window(WindowDesc::new(build_ui(image)));
            ctx.window().close();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) { }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {   }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppState, env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {

    }
}
fn build_ui(image:screenshots::Image) -> impl Widget<AppState> {
    //let screen = Screen::from_point(0, 0).unwrap();

    //let image = screen.capture().unwrap();

    println!("after screen");
    let image_buf=ImageBuf::from_raw(image.rgba().clone(),ImageFormat::RgbaSeparate,image.width() as usize,image.height() as usize);

    let save_png = Button::new("Save as png")
        .on_click(|_ctx, _data, _env| {});

    let image:Image=Image::new(image_buf).fill_mode(FillStrat::Contain);
    Flex::row().with_child(save_png).with_child(image)
    //Flex::row().with_child(Image::new(image_buf))
}


fn main() -> Result<(), PlatformError> {
    // Create the window
    let main_window = WindowDesc::new(ui_builder())
        .window_size((1000.0, 1000.0))
        .title(LocalizedString::new("Image Example")).show_titlebar(false).transparent(true);

    // Launch the application
    AppLauncher::with_window(main_window).launch(AppState {
        mouse_position: Point::new(0.0, 0.0),
        initial_point: None,
        final_point: None,
    })
}
