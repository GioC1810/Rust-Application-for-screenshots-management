use std::cell::RefCell;
use std::fs;
use std::env;
use std::rc::Rc;
use druid::widget::{Button, Flex, Image, SizedBox, ZStack};
use druid::{Point, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget, WidgetExt, WindowDesc, ImageBuf, LensExt, KbKey};
use druid::WindowState::Maximized;
use druid::piet::ImageFormat as FormatImage;
use image::{DynamicImage, ImageBuffer};
use screenshots::{Compression, Screen};
use arboard::{Clipboard,ImageData};

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
    pub cropping_mode: bool,
    pub cropped_area: Option<Rect>,
    pub image:Option<ImageBuf>,
    #[data(same_fn = "hotkeys_equal")]
    pub hotkeys: Vec<HotKey>,
    pub hotkey_to_register: HotKey,
    pub actual_hotkey: HotKey,
    pub image_width:u32,
    pub image_height:u32
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
                    let expandable_rect = ExpandableRect::new(mouse_event.pos);
                    data.current_rectangle = Some(expandable_rect);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    data.final_point=Some(data.mouse_position);
                    //ctx.request_paint(); // Request a redraw
                    println!("after click {}", mouse_event.pos.x);
                    if let Some(rect) = data.current_rectangle.take() {
                        data.rectangles.push(rect);
                        ctx.request_paint();
                    }
                    if data.cropping_mode{
                        data.cropping_mode = false;
                        ctx.request_paint();
                    }
                }
            }


            _ => (),
        }
        if data.initial_point.is_some() && data.final_point.is_some(){



            let scale_factor=1.0;

            let screenshot_width=data.final_point.unwrap().x-data.initial_point.unwrap().x ;
            let mut screenshot_height=data.final_point.unwrap().y-data.initial_point.unwrap().y ;

            let mut initial_height = data.initial_point.unwrap().y as i32;

            if env::consts::OS.eq("macos") {
                initial_height += 55;
            }
            let image=Screen::from_point(0,0).unwrap().capture_area(data.initial_point.unwrap().x as i32, initial_height as i32,screenshot_width as u32, screenshot_height as u32).unwrap();

            let image_buf=ImageBuf::from_raw(image.rgba().clone(),FormatImage::RgbaPremul,image.width() as usize,image.height() as usize);
            data.image=Some(image_buf.clone());
            data.image_width=image.width();
            data.image_height=image.height();
            println!("{}",data.cropped_area.is_some());
            ctx.window().close();
            ctx.new_window(WindowDesc::new(build_ui(Image::new(image_buf), image, data)).set_window_state(Maximized));

        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) { }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {   }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppState, env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {

        let rect=Rect::from_origin_size( Point{x:0.0, y:0.0}, Size{width:data.image_width as f64,height:data.image_height as f64});
        ctx.stroke(rect, &Color::WHITE,1.0);

        for expandable_rect in &data.rectangles {

            ctx.fill(expandable_rect.rect, &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Transparent background
            ctx.stroke(expandable_rect.rect, &Color::WHITE, 1.0); // White border
        }

        if let Some(expandable_rect) = &data.current_rectangle {
            ctx.fill(expandable_rect.rect, &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Transparent background
            ctx.stroke(expandable_rect.rect, &Color::WHITE, 1.0); // White border
        }
    }
}

pub struct Croptest;
impl Widget<AppState> for Croptest {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseMove(mouse_event) => {
                // Update the mouse position in the app state
                data.mouse_position = mouse_event.pos;
                ctx.request_paint(); // Request a redraw
                if let Some(rect) = &mut data.current_rectangle {
                    rect.update(mouse_event.pos);
                    ctx.request_paint();
                }
            }
            Event::MouseDown(mouse_event) => {
                // Check if cropping mode is active and update cropping area
                if data.cropping_mode && mouse_event.button == MouseButton::Left {
                    data.initial_point = Some(data.mouse_position);
                    let expandable_rect = ExpandableRect::new(mouse_event.pos);
                    data.current_rectangle = Some(expandable_rect);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                ctx.set_focus(ctx.widget_id());
                // Update cropping area and disable cropping mode
                if data.cropping_mode && mouse_event.button == MouseButton::Left {
                    data.final_point = Some(data.mouse_position);

                    if let Some(rect) = data.current_rectangle.take() {
                        data.rectangles.push(rect);
                        ctx.request_paint();
                    }

                    data.cropping_mode = false;
                    ctx.request_paint();
                }
            }
            _ => (),
        }
        if data.initial_point.is_some() && data.final_point.is_some() {


            let cropped_width = data.final_point.unwrap().x - data.initial_point.unwrap().x;
            let mut cropped_height = data.final_point.unwrap().y - data.initial_point.unwrap().y;
            if env::consts::OS.eq("macos") {
                cropped_height += 100.0;
            }
            let rgba_data = data.image.as_ref().unwrap().raw_pixels();

            let mut dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                data.image.as_ref().unwrap().width() as u32,
                data.image.as_ref().unwrap().height() as u32,
                rgba_data.to_vec(),
            )
                .expect("Failed to create ImageBuffer"));

            let mut initial_height = data.initial_point.unwrap().y as u32;

            if env::consts::OS.eq("macos") {
                initial_height += 55;
            }
            let cropped_dyn_image = dynamic_image.crop_imm(data.initial_point.unwrap().x as u32, initial_height, cropped_width as u32, cropped_height as u32);
            let rgba_data_cropped = cropped_dyn_image.clone().into_rgba8().to_vec();
            let cropped_image_buf=Some(ImageBuf::from_raw(rgba_data_cropped.clone(),FormatImage::RgbaPremul,cropped_dyn_image.width() as usize,cropped_dyn_image.height() as usize));
            let cropped_image=screenshots::Image::new(cropped_dyn_image.width() as u32,cropped_dyn_image.height() as u32,rgba_data_cropped);

            // Get image dimensions
            data.cropping_mode = !data.cropping_mode;
            //data.cropping_mode=true;
            data.initial_point = None;
            data.final_point = None;
            data.image=cropped_image_buf.clone();
            data.image_width=cropped_image.width();
            data.image_height=cropped_image.height();
            data.cropped_area = None;
            ctx.window().close();
            ctx.new_window(WindowDesc::new(build_ui(Image::new(cropped_image_buf.unwrap()), cropped_image,  data)).set_window_state(Maximized));
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) { }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {   }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppState, env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        let size= ctx.size();
        ctx.stroke(size.to_rect(),&Color::YELLOW,1.0);

        if let Some(expandable_rect) = &data.current_rectangle {
            ctx.fill(expandable_rect.rect, &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Transparent background
            ctx.stroke(expandable_rect.rect, &Color::WHITE, 1.0); // White border
        }
    }
}

//ui generation functions

pub fn ui_builder() -> impl Widget<AppState> {

    let screen_button = Button::new("Screen")
        .on_click(|ctx, _data, _env| {
            let mut is_macos = false;
            if env::consts::OS.eq("macos") {
                is_macos = true;
            }
            ctx.new_window(WindowDesc::new(MyApp)
                .set_window_state(Maximized)
                .set_position(Point::new(0 as f64, 0 as f64))
                .show_titlebar(is_macos)
                .transparent(true)
            );
            ctx.window().close();
        });

    let memorize_hotkey = Button::new("Add hotkey")
        .on_click(|ctx, _data, _env| {
            let mut is_macos = false;
            if env::consts::OS.eq("macos") {
                is_macos = true;
            }
            ctx.new_window(WindowDesc::new(HoyKeyRecord)
                .title("digit hotkey")
                .set_window_state(Maximized)
                .set_position(Point::new(0 as f64, 0 as f64))
                .show_titlebar(true)
                .transparent(false)
            );
            ctx.window().close();
        });

    let buttons_row = Flex::row()
        .with_child(screen_button)
        .with_spacer(16.0) // Add spacing between buttons
        .with_child(memorize_hotkey);

    Flex::column()
        .with_child(buttons_row) // Add the buttons row
        .with_spacer(16.0) // Add spacing between buttons and KeyDetectionApp
        .with_child(KeyDetectionApp)
}

fn build_ui(image:Image, mut img: screenshots::Image, my_data:&mut AppState) -> impl Widget<AppState> {

    my_data.mouse_position=Point::new(0.0, 0.0);
    my_data.initial_point=None;
    my_data.final_point=None;
    my_data.current_rectangle= None;
    my_data.rectangles= Vec::new();
    my_data.cropped_area=None;
    my_data.cropping_mode= false;

    let toggle_crop_button = Button::new("Toggle Crop")
        .on_click(|ctx, data:&mut AppState, _: &Env| {

            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.cropped_area=None;
            data.cropping_mode= !data.cropping_mode;

        });

    let img_data = Rc::new(RefCell::new(img.to_png(Compression::Default).unwrap().clone()));

    fn save_image(img_type: i32, img: Vec<u8>) {
        let path_name = match img_type {
            0 => "test_crop_values.png",
            1 => "test_crop_values.jpg",
            2 => "test_crop_values.gif",
            _ => "",
        };
        fs::write(path_name, img).unwrap();
    }

    let save_as_png_data = Rc::clone(&img_data);
    let save_as_jpg_data = Rc::clone(&img_data);
    let save_as_gif_data = Rc::clone(&img_data);
    let copy_to_clipboard_data = Rc::clone(&img_data);


    let save_as_png = Button::new("Save as png")
        .on_click(move |ctx, data: &mut AppState, _: &Env| {
            let mut is_macos = false;
            if env::consts::OS.eq("macos") {
                is_macos = true;
            }
            let img_data = Rc::clone(&save_as_png_data);
            let img_cloned = img_data.borrow().to_owned();
            save_image(0, img_cloned);
            ctx.new_window(WindowDesc::new(ui_builder())
                .set_window_state(Maximized)
                .set_position(Point::new(0 as f64, 0 as f64))
                .show_titlebar(is_macos)
                .transparent(true)
            );
            ctx.window().close();
        });

    let save_as_jpg = Button::new("Save as jpg")
        .on_click(move |ctx, data: &mut AppState, _: &Env| {
            let mut is_macos = false;
            if env::consts::OS.eq("macos") {
                is_macos = true;
            }
            let img_data = Rc::clone(&save_as_jpg_data);
            let img_cloned = img_data.borrow().to_owned();
            save_image(1, img_cloned);
            ctx.new_window(WindowDesc::new(ui_builder())
                .set_window_state(Maximized)
                .set_position(Point::new(0 as f64, 0 as f64))
                .show_titlebar(is_macos)
                .transparent(true)
            );
            ctx.window().close();
        });

    let save_as_gif = Button::new("Save as gif")
        .on_click(move |ctx, data: &mut AppState, _: &Env| {
            let mut is_macos = false;
            if env::consts::OS.eq("macos") {
                is_macos = true;
            }
            let img_data = Rc::clone(&save_as_gif_data);
            let img_cloned = img_data.borrow().to_owned();
            save_image(2, img_cloned);
            ctx.new_window(WindowDesc::new(ui_builder())
                .set_window_state(Maximized)
                .set_position(Point::new(0 as f64, 0 as f64))
                .show_titlebar(is_macos)
                .transparent(true)
            );
            ctx.window().close();
        });

    let copy_to_clipboard = Button::new("Copy to clipboard")
        .on_click(move |ctx, data: &mut AppState, _: &Env| {
            let img_data = Rc::clone(&copy_to_clipboard_data);
            let img_cloned = img_data.borrow().to_owned();
            Clipboard::new().unwrap().set_image(ImageData { width: img.width() as usize, height: img.height() as usize, bytes: img.rgba().into() });
            ctx.new_window(WindowDesc::new(ui_builder())
                .set_window_state(Maximized)
                .set_position(Point::new(0 as f64, 0 as f64))
                .show_titlebar(true)
                .transparent(true)
            );
            ctx.window().close();
        });

    Flex::column()
        .with_child(toggle_crop_button)
        .with_child(save_as_png)
        .with_child(save_as_jpg)
        .with_child(save_as_gif)
        .with_child(copy_to_clipboard)
        .with_child(SizedBox::new(ZStack::new(image)
            .with_centered_child(Croptest))
            .width(my_data.image_width as f64)
            .height(my_data.image_height as f64))
        .with_child(KeyDetectionApp)

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

#[derive(Clone)]
pub struct HotKey{
    pub keys: Vec<KbKey>
}

impl HotKey{
    pub fn new() -> HotKey{
        HotKey{keys: Vec::new()}
    }
}


impl Data for HotKey{
    fn same(&self, other: &Self) -> bool {
        self.keys.iter().zip(&other.keys).all(|(a, b)| a.eq(b))
    }
}

fn hotkeys_equal(r1: &Vec<HotKey>, r2: &Vec<HotKey>) -> bool{
    if r1.len() != r2.len() {
        return false;
    }

    r1.iter().zip(r2).all(|(hotkey1, hotkey2)| {
        hotkey1.keys.len() == hotkey2.keys.len()
            && hotkey1
            .keys
            .iter()
            .zip(&hotkey2.keys)
            .all(|(key1, key2)| key1 == key2)
    })
}

fn find_hotkey_match(r1: &HotKey, r2: &Vec<HotKey>) -> bool{
    r2.iter().filter(|r| {
        if r.keys.len()  != r1.keys.len() {
            return false;
        }
        r.keys.iter()
            .zip(&r1.keys)
            .all(|(hotkey1, hotkey2)| {
                hotkey1 == hotkey2 }
            )}
    ).count() > 0
}

pub struct HoyKeyRecord;
impl Widget<AppState> for HoyKeyRecord {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        ctx.set_focus(ctx.widget_id());
        match event {
            Event::KeyDown(key_event) => {
                if data.hotkey_to_register.keys.len() < 4 && (data.hotkey_to_register.keys.len() == 0 || key_event.key.ne(data.hotkey_to_register.keys.get(data.hotkey_to_register.keys.len()-1).unwrap())) {
                    data.hotkey_to_register.keys.push(key_event.key.clone());
                    println!("insert new hotkey: {:?}", data.hotkey_to_register.keys.get(data.hotkey_to_register.keys.len() - 1));
                }
            }
            Event::KeyUp(_) => {
                data.hotkeys.push(data.hotkey_to_register.clone());
                for hotkey in &data.hotkeys{
                    print_hotkeys(&hotkey.keys);
                }
                println!("hoykeys registered after escape: ");
                data.hotkey_to_register.keys.clear();
                println!("hoykeys memorized: ");
                ctx.new_window(WindowDesc::new(ui_builder())
                    .set_window_state(Maximized)
                    .set_position(Point::new(0 as f64, 0 as f64))
                    .show_titlebar(true)
                    .transparent(true)
                );
                ctx.window().close();
            }
            _ => {}
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

//utility functions

fn convert_coordinates(coord: (f64, f64), source_scale: f64, target_scale: f64) -> (f64, f64) {
    let (x, y) = coord;
    let converted_x = x * (target_scale / source_scale);
    let converted_y = y * (target_scale / source_scale);
    (converted_x, converted_y)
}

fn print_hotkeys(r: &Vec<KbKey>) {
    println!("hotkeys printed______");
    for (i,  key) in r.iter().enumerate() {
        println!("character number: {:?}, character value: {:?}", i, key);
    }
}

pub struct KeyDetectionApp;
impl Widget<AppState> for KeyDetectionApp {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        ctx.set_focus(ctx.widget_id());
        match event {
            Event::KeyDown(key_event) => {
                if data.actual_hotkey.keys.len() < 4 && (data.actual_hotkey.keys.len() == 0 || key_event.key.ne(data.actual_hotkey.keys.get(data.actual_hotkey.keys.len() - 1).unwrap())) {
                    println!("button pressed to trigger combination: {:?}", key_event.key);
                    data.actual_hotkey.keys.push(key_event.key.clone());
                    if find_hotkey_match(&data.actual_hotkey, &data.hotkeys) {
                        println!("combination triggered!!");
                        ctx.new_window(WindowDesc::new(MyApp)
                            .set_window_state(Maximized)
                            .set_position(Point::new(0 as f64, 0 as f64))
                            .show_titlebar(true)
                            .transparent(true)
                        );
                        ctx.window().close();
                        data.actual_hotkey.keys.clear();
                    }
                } else if data.actual_hotkey.keys.len() == 4 {
                    println!("overreach the max number of button for the hotkey, start again!");
                    data.actual_hotkey.keys.clear();
                    ctx.new_window(WindowDesc::new(ui_builder())
                        .set_window_state(Maximized)
                        .set_position(Point::new(0 as f64, 0 as f64))
                        .show_titlebar(true)
                        .transparent(true)
                    );
                    ctx.window().close();
                }
            }
            Event::KeyUp(key_event) => {
                println!("Hotkey pressed: {:?}", key_event.key);
                data.actual_hotkey.keys.clear();
                if key_event.key == KbKey::Escape{
                    ctx.new_window(WindowDesc::new(ui_builder())
                        .set_window_state(Maximized)
                        .set_position(Point::new(0 as f64, 0 as f64))
                        .show_titlebar(true)
                        .transparent(true)
                    );
                    ctx.window().close();
                }
            }
            _ => {}
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





