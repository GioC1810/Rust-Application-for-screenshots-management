use std::cell::RefCell;
use std::{rc::Rc,fs};
use druid::widget::{Axis, Button, Flex, Image, KnobStyle, Label,  Painter,SizedBox, Slider, TextBox, ViewSwitcher, ZStack};
use druid::{Point, Color, Env, EventCtx, RenderContext, Size,  Widget, WindowDesc, ImageBuf, WidgetExt, KeyOrValue};
use druid::WindowState::Maximized;
use druid::piet::ImageFormat as FormatImage;
use image::{DynamicImage, ImageBuffer, Rgba};
use screenshots::{Compression, Screen};
use arboard::{Clipboard,ImageData};
use native_dialog::{FileDialog};
use rusttype::{Font,Scale};
use imageproc::drawing::draw_text;
use crate::data::*;
use crate::hotkey_screen::*;
use crate::graphical_elements::*;

pub fn initial_window(ctx:&mut EventCtx){
    //let screens=Screen::all().unwrap();
    ctx.new_window(WindowDesc::new(ui_builder())
        .window_size((700.0,200.0))
        .show_titlebar(true)
        .title("Screenshot App")
    );
    ctx.window().close();
}
pub fn screen_window(ctx:&mut EventCtx, data: &mut AppState){

    data.initial_point=None;
    data.final_point=None;

    ctx.new_window(WindowDesc::new(MyApp)
        .set_window_state(Maximized)
        .set_position(Point::new(data.screen.display_info.x as f64, data.screen.display_info.y as f64))
        .show_titlebar(data.is_macos)
        .transparent(true)

    );
    ctx.window().close();
}
pub fn editing_window(ctx:&mut EventCtx,img: screenshots::Image, my_data:&mut AppState) {
    ctx.new_window(WindowDesc::new(build_ui(img,  my_data)).title("Editing Window")
        .set_window_state(Maximized));
    ctx.window().close();

}
pub fn ui_builder() -> impl Widget<AppState> {

    fn take_screenshot(ctx: &mut EventCtx, data: &mut AppState) {

        data.current_rectangle = None;
        data.rectangles.clear();
        data.cropping_mode = false;
        data.initial_point = None;
        data.final_point = None;
        data.image_height=0;
        data.image_width=0;
        screen_window(ctx, data);
    }

    let screens = Screen::all().unwrap();
    let mut monitors_buttons = Flex::column();
    let mut counter = 0;
    for screen in screens{
        monitors_buttons = monitors_buttons.with_child(Button::new("Take Screen on Monitor ".to_owned() + &counter.to_string())
            .on_click(move |ctx, data: &mut AppState, _env| {
                data.screen = screen;

                take_screenshot(ctx,data);
            }));
        counter += 1;
    }

    let memorize_hotkey = Button::new("Add hotkey")
        .on_click(|ctx, data, _env| {

            ctx.new_window(WindowDesc::new(build_hotkey_ui(data))
                .title("digit hotkey")
                .window_size((500.0,200.0))
                .set_always_on_top(true)
                .show_titlebar(true)
                .transparent(false)
            );
            ctx.window().close();
        });

    #[tokio::main]
    pub async fn timer_handling(_ctx: &mut EventCtx,_monitor_index: usize, time: u64) {
        // Sleep for time seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(time)).await;
    }

    let timer_button = Button::new("Start timer")
        .on_click(|ctx, data:&mut AppState, _: &Env| {
            timer_handling(ctx,1,data.value as u64);
            take_screenshot(ctx,data);
        });

    let value = Flex::row()
        .with_child(Label::dynamic(|value: &AppState, _| {
            format!("Seconds: {:?}", value.value)
        }))
        .with_default_spacer()
        .with_child(ViewSwitcher::new(
            |_data: &AppState, _| (0.0,10.0),
            |range, _, _| {
                Slider::new()
                    .with_range(range.0, range.1)
                    .track_color(KeyOrValue::Concrete(Color::YELLOW))
                    .knob_style(KnobStyle::Circle)
                    .axis(Axis::Horizontal)
                    .with_step(0.5)
                    .annotated(1.0, 1.0)
                    .fix_width(150.0)
                    .lens(AppState::value)
                    .boxed()
            },
        ));

    let buttons_row = Flex::row()
        //.with_child(screen_button)
        .with_child(monitors_buttons)
        .with_spacer(16.0)
        // Add spacing between buttons
        .with_child(memorize_hotkey)
        .with_spacer(16.0) // Add spacing between buttons
        .with_child(timer_button)
        .with_child(value);


    Flex::column()
        .with_child(buttons_row) // Add the buttons row
        .with_spacer(16.0) // Add spacing between buttons and KeyDetectionApp
        .with_child(KeyDetectionApp)
}

pub fn build_ui(img: screenshots::Image, my_data:&mut AppState) -> impl Widget<AppState> {

    //let selected_color_label  = Label::new("Selected Color:");

    let color_circle = Painter::new(|ctx, data: &AppState, _env| {
        let circle_rect = Size{width:20.0, height:20.0}.to_rect();
        ctx.fill(circle_rect, &data.selected_color);
    }).fix_size(20.0, 20.0)
        .border(Color::BLACK, 2.0);

    let toggle_crop_button = Button::new("Toggle Crop")
        .on_click(|_ctx, data:&mut AppState, _: &Env| {
            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.cropping_mode= !data.cropping_mode;

        });



    let img_data = Rc::new(RefCell::new(img.to_png(Compression::Default).unwrap().clone()));


    let save_as_png_data = Rc::clone(&img_data);
    let save_as_jpg_data = Rc::clone(&img_data);
    let save_as_gif_data = Rc::clone(&img_data);
    let save_data = Rc::clone(&img_data);
    //let copy_to_clipboard_data = Rc::clone(&img_data);

    let save_button = Button::new("Save")
        .on_click(move |_ctx, data: &mut AppState, _: &Env| {
            let file_name = FileDialog::new()
                .add_filter("PNG", &["png"])
                .add_filter("JPEG", &["jpeg"])
                .add_filter("GIF", &["gif"])
                .set_filename(&*("IMG".to_string() + &data.screen_saved_counter.to_string()))
                .show_save_single_file()
                .unwrap();

            if file_name.is_some() {
                fs::write(file_name.unwrap(), save_data.borrow().to_owned()).expect("error in saving the file");
                data.screen_saved_counter +=1;
            }

        });


    let save_as_png = Button::new("Save as png")
        .on_click(move |ctx, _data: &mut AppState, _: &Env| {

            let img_data = Rc::clone(&save_as_png_data);
            let img_cloned = img_data.borrow().to_owned();
            ctx.submit_command(SAVE_IMAGE_COMMAND.with(SaveImageCommand{img_format: 1,  img: img_cloned}));
        });

    let save_as_jpg = Button::new("Save as jpg")
        .on_click(move |ctx, _data: &mut AppState, _: &Env| {
            let img_data = Rc::clone(&save_as_jpg_data);
            let img_cloned = img_data.borrow().to_owned();
            ctx.submit_command(SAVE_IMAGE_COMMAND.with(SaveImageCommand{img_format: 1,  img: img_cloned}));
        });

    let save_as_gif = Button::new("Save as gif")
        .on_click(move |ctx, _data: &mut AppState, _: &Env| {

            let img_data = Rc::clone(&save_as_gif_data);
            let img_cloned = img_data.borrow().to_owned();
            ctx.submit_command(SAVE_IMAGE_COMMAND.with(SaveImageCommand{img_format: 1,  img: img_cloned}));
        });

    let copy_to_clipboard = Button::new("Copy to clipboard")
        .on_click(move |_ctx, _data: &mut AppState, _: &Env| {
            //let img_data = Rc::clone(&copy_to_clipboard_data);
            Clipboard::new().unwrap().set_image(ImageData { width: img.width() as usize, height: img.height() as usize, bytes: img.rgba().into() }).expect("Error in copying");
            //initial_window(ctx);
        });

    let insert_input=Button::new("Insert Text")
        .on_click(move |_ctx, data: &mut AppState, _: &Env| {
            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.is_inserting_text=true;

        });
    let draw_rectangle= Button::new("⬜").on_click(move |_ctx, data: &mut AppState, _: &Env| {
        data.mouse_position=Point::new(0.0, 0.0);
        data.initial_point=None;
        data.final_point=None;
        data.current_rectangle= None;
        data.rectangles= Vec::new();
        data.draw_rect_mode= !data.draw_rect_mode;
    });
    let draw_circle= Button::new("⚪").on_click(move |_ctx, data: &mut AppState, _: &Env| {
        data.mouse_position=Point::new(0.0, 0.0);
        data.initial_point=None;
        data.final_point=None;
        data.current_rectangle= None;
        data.rectangles= Vec::new();
        data.draw_circle_mode= !data.draw_circle_mode;
    });
    let draw_arrow= Button::new("↘").on_click(move |_ctx, data: &mut AppState, _: &Env| {
        data.mouse_position=Point::new(0.0, 0.0);
        data.initial_point=None;
        data.final_point=None;
        data.current_rectangle= None;
        data.rectangles= Vec::new();
        data.draw_arrow_mode= !data.draw_arrow_mode;
    });
    let draw_lines= Button::new("✏").on_click(move |_ctx, data: &mut AppState, _: &Env| {
        data.mouse_position=Point::new(0.0, 0.0);
        data.initial_point=None;
        data.final_point=None;
        data.current_rectangle= None;
        data.rectangles= Vec::new();
        data.draw_lines_mode= !data.draw_lines_mode;
    });

    let highlight= Button::new("🖍").on_click(move |_ctx, data: &mut AppState, _: &Env| {
        data.mouse_position=Point::new(0.0, 0.0);
        data.initial_point=None;
        data.final_point=None;
        data.current_rectangle= None;
        data.rectangles= Vec::new();
        data.is_highliting= true;
    });

    let change_color= Button::new("Change Color").on_click(move |ctx, _data: &mut AppState, _: &Env| {
        ctx.new_window(WindowDesc::new(ColorGrid).window_size((500.0,200.0)).set_position((50.0,50.0)));
        ctx.window().close();
    });

    let mut editing_row=Flex::row().with_child(draw_rectangle).with_child(draw_circle)
        .with_child(draw_arrow)
        .with_child(draw_lines)
        .with_child(highlight)
        .with_child(change_color)
        .with_child(insert_input);
    if !my_data.is_macos{
        editing_row=editing_row.with_child(color_circle);
    }

    let mut save_row=Flex::row().with_child(save_as_png)
        .with_child(save_as_jpg)
        .with_child(save_as_gif);
    if !my_data.is_macos{
        save_row=Flex::row().with_child(save_button)
    }
    Flex::column()
        .with_child(toggle_crop_button)
        .with_child(save_row
        )
        .with_child(copy_to_clipboard)
        .with_child(
            editing_row
        )
        .with_child(SizedBox::new(ZStack::new(Image::new(my_data.image.clone().unwrap()))
            .with_centered_child(Croptest))
            .width(my_data.image_width as f64)
            .height(my_data.image_height as f64))
        .with_child(KeyDetectionApp)

}
pub fn build_ui_save_file(img: Vec<u8>, _data: &mut AppState, img_format: i32) -> impl Widget<AppState>{

    let button_save = Button::new("Save image")
        .on_click(move |ctx, data: &mut AppState, _env|{
            let extension = match img_format{
                0 => ".png",
                1 => ".jpg",
                2 => ".gif",
                _ => ".png"
            };
            if data.file_path.is_empty(){
                ctx.new_window(WindowDesc::new(ui_builder())
                    .set_window_state(Maximized)
                    .set_position(Point::new(0 as f64, 0 as f64))
                    .show_titlebar(true)
                    .transparent(true)
                );
                ctx.window().close();
            } else {
                let file_name = data.clone().file_path + "_" + &data.screen_saved_counter.to_string() + extension ;
                data.screen_saved_counter += 1;
                fs::write(file_name, img.clone()).expect("error in saving the file");
                ctx.new_window(WindowDesc::new(ui_builder())
                    .set_window_state(Maximized)
                    .set_position(Point::new(0 as f64, 0 as f64))
                    .show_titlebar(true)
                    .transparent(true)
                );
                ctx.window().close();
            }
        });

    let text_box = TextBox::new()
        .lens(AppState::file_path)
        .expand_width()
        .fix_height(20.0);

    Flex::column()
        .with_child(text_box)
        .with_spacer(20.0)
        .with_child(button_save)

}


pub fn build_input_box(_data:&mut AppState) -> impl Widget<AppState> {
    let input_label = Label::new("Enter text:");

    let text_box = TextBox::new()
        .lens(AppState::input_text)
        .expand_width()
        .fix_height(30.0);

    let submit_button = Button::new("Submit")
        .on_click(|ctx, data: &mut AppState, _env| {

            let rgba_data = data.image.as_ref().unwrap().raw_pixels();
            let rgba=Rgba([data.selected_color.as_rgba8().0,data.selected_color.as_rgba8().1,data.selected_color.as_rgba8().2,data.selected_color.as_rgba8().3]);

            let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                data.image.as_ref().unwrap().width() as u32,
                data.image.as_ref().unwrap().height() as u32,
                rgba_data.to_vec(),
            ).expect("Failed to create ImageBuffer"));

            let font_data: &[u8] = include_bytes!("../Montserrat-Italic.otf");
            let font = Font::try_from_bytes(font_data);
            let new_image= Some(draw_text(&dynamic_image, rgba, data.initial_point.unwrap().x as i32, data.initial_point.unwrap().y as i32, Scale::uniform((data.final_point.unwrap().y-data.initial_point.unwrap().y) as f32),
                                          &font.unwrap(), &*data.input_text));

            let rgba_data_drawn = new_image.clone().unwrap().into_raw().to_vec();
            let drawn_image_buf=Some(ImageBuf::from_raw(rgba_data_drawn.clone(),FormatImage::RgbaPremul,new_image.clone().unwrap().width() as usize,new_image.unwrap().height() as usize));
            data.image=drawn_image_buf.clone();
            let image=screenshots::Image::new(dynamic_image.width() as u32,dynamic_image.height() as u32,rgba_data_drawn);
            data.is_inserting_text=false;
            data.initial_point = None;
            data.final_point = None;

            editing_window(ctx,image,  data);
            // Clear the text input field after submission
            data.input_text.clear();
            ctx.request_update(); // Request a UI update to clear the text box
        });

    Flex::column()
        .with_child(input_label)
        .with_spacer(10.0)
        .with_child(text_box)
        .with_spacer(10.0)
        .with_child(submit_button)
}

