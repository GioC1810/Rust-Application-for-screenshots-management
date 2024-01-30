use std::f64::consts::PI;
use std::fs;
use std::sync::Arc;
use druid::{Point, BoxConstraints, Color,Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget, WindowDesc, ImageBuf, Lens, Cursor};
use druid::piet::ImageFormat as FormatImage;
use druid::kurbo::BezPath;
use image::{DynamicImage, ImageBuffer, Rgba};
use imageproc::drawing::{draw_line_segment, draw_hollow_rect, draw_hollow_circle};
use imageproc::rect::Rect as OtherRect;
use screenshots::{Compression, Screen};
use global_hotkey::GlobalHotKeyManager;
use crate::graphical_elements::*;
use crate::hotkey_screen::*;
use crate::ui_functions::*;

pub struct MyApp;
#[derive(Clone, Data, Lens)]
pub struct AppState{
    pub mouse_position: Point,
    pub initial_point:Option<Point>,
    pub final_point:Option<Point>,
    #[data(same_fn = "rectangles_equal")]
    pub rectangles: Vec<ExpandableRect>,
    pub current_rectangle: Option<ExpandableRect>,
    pub cropping_mode: bool,
    pub draw_rect_mode:bool,
    pub draw_circle_mode:bool,
    pub draw_arrow_mode:bool,
    pub draw_lines_mode: bool,
    pub is_drawing:bool,
    pub is_highliting:bool,
    pub is_inserting_text:bool,
    pub input_text:String,
    pub selected_color: Color,
    pub value:f64,
    #[data(same_fn = "point_equal")]
    pub all_positions:Vec<Point>,
    pub draw_path: BezPath,
    pub image:Option<ImageBuf>,
    #[data(same_fn = "hotkeys_equal")]
    pub hotkeys: Vec<HotKey>,
    pub hotkey_to_register: HotKey,
    pub actual_hotkey: HotKey,
    pub image_width:u32,
    pub image_height:u32,
    #[data(same_fn = "screen_equal")]
    pub screen: Screen,
    pub file_path: String,
    pub screen_saved_counter: i32,
    pub is_macos:bool,
    pub hotkey_manager: Arc<GlobalHotKeyManager>
}

impl Widget<AppState> for MyApp {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        ctx.window().bring_to_front_and_focus();

        match event {

            Event::MouseMove(mouse_event) => {
                data.mouse_position = mouse_event.pos;
                ctx.request_paint(); // Request a redraw
                ctx.set_focus(ctx.widget_id());

                ctx.set_cursor(&Cursor::Crosshair);
                if let Some(rect) = &mut data.current_rectangle {
                    rect.update(mouse_event.pos,data.initial_point.unwrap());
                    ctx.request_paint();
                }
            }
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    /*
                     implemented for multimonitors. When the mouse is raised outside one monitor the final point is not saved.
                     Therefore when the mouse comes back to the monitor selected for the screen, the user click to define the final point.
                     If we already have an initial point, we assign the new clicked points to the final point.
                    */
                    if data.initial_point.is_none() {
                        data.initial_point = Some(data.mouse_position);

                    }else {
                        data.final_point = Some(data.mouse_position);
                    }
                    ctx.request_paint(); // Request a redraw
                    let expandable_rect = ExpandableRect::new(mouse_event.pos);
                    data.current_rectangle = Some(expandable_rect);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    data.final_point=Some(data.mouse_position);
                    ctx.set_cursor(&Cursor::Arrow);
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

            if data.final_point.unwrap().x<data.initial_point.unwrap().x{
                let temp=data.initial_point;
                data.initial_point=data.final_point;
                data.final_point=temp;
            }

            let mut divisor_factor = 1.0;
            if data.is_macos{
                divisor_factor *= 2.0;
            }

            data.initial_point = Some(Point::new(data.initial_point.unwrap().x * (ctx.scale().x())/divisor_factor , data.initial_point.unwrap().y * (ctx.scale().y())/divisor_factor));
            data.final_point = Some(Point::new(data.final_point.unwrap().x * (ctx.scale().x())/divisor_factor , data.final_point.unwrap().y * (ctx.scale().y())/divisor_factor));



            let screenshot_width=data.final_point.unwrap().x-data.initial_point.unwrap().x ;
            let screenshot_height=data.final_point.unwrap().y-data.initial_point.unwrap().y ;

            let mut initial_height = data.initial_point.unwrap().y;

            if data.is_macos{
                initial_height += 72.5;
            }

            if screenshot_width>5.0 && screenshot_height>5.0{

                let image=Screen::from_point(data.screen.display_info.x,data.screen.display_info.y).unwrap().capture_area(data.initial_point.unwrap().x as i32, initial_height as i32,screenshot_width as u32, screenshot_height as u32).unwrap();

                fs::write(&*("IMG".to_string() + &data.screen_saved_counter.to_string()+".png"),image.to_png(Compression::Default).expect("--").clone()).expect("error in saving the file");
                data.screen_saved_counter+=1;

                let image_buf=ImageBuf::from_raw(image.rgba().clone(),FormatImage::RgbaPremul,image.width() as usize,image.height() as usize);
                data.image=Some(image_buf.clone());
                data.image_width=image.width();
                data.image_height=image.height();
                data.initial_point=None;
                data.final_point=None;

                data.rectangles= Vec::new();
                data.current_rectangle=None;


                editing_window(ctx,image,  data);

            }
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) { }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {   }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {

        let rect=Rect::from_origin_size( Point{x:0.0, y:0.0}, Size{width:data.image_width as f64,height:data.image_height as f64});
        ctx.stroke(rect, &Color::WHITE,1.0);

        /*for expandable_rect in &data.rectangles {

            ctx.fill(expandable_rect.rect, &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Transparent background
            ctx.stroke(expandable_rect.rect, &Color::WHITE, 1.0); // White border
        }*/

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
                ctx.request_focus();
                data.mouse_position = mouse_event.pos;
                if let Some(rect) = &mut data.current_rectangle {
                    rect.update(mouse_event.pos,data.initial_point.unwrap());
                }
                if data.draw_lines_mode==true{
                    if data.is_drawing==true{
                        data.all_positions.push(mouse_event.pos);
                        data.draw_path.line_to(data.mouse_position);
                    }
                }
                if data.cropping_mode || data.draw_rect_mode || data.is_inserting_text {
                    ctx.set_cursor(&Cursor::Crosshair);
                }

                ctx.request_paint();
            }
            Event::MouseDown(mouse_event) => {
                // Check if cropping mode is active and update cropping area
                if (data.cropping_mode || data.draw_rect_mode || data.draw_circle_mode
                    || data.draw_arrow_mode || data.draw_lines_mode || data.is_highliting
                    || data.is_inserting_text) && mouse_event.button == MouseButton::Left {


                    /*
                     implemented for multimonitors. When the mouse is raised outside one monitor the final point is not saved.
                     Therefore when the mouse comes back to the monitor selected for the screen, the user click to define the final point.
                     If we already have an initial point, we assign the new clicked points to the final point.
                    */

                    if data.initial_point.is_none(){
                        data.initial_point = Some(data.mouse_position);
                    }else{
                        data.final_point = Some(data.mouse_position);
                    }
                    let expandable_rect = ExpandableRect::new(mouse_event.pos);

                    if data.cropping_mode || data.draw_rect_mode || data.is_inserting_text {
                        ctx.set_cursor(&Cursor::Crosshair);
                        data.current_rectangle = Some(expandable_rect);
                    }
                    if data.draw_lines_mode==true{
                        data.is_drawing=true;
                        data.draw_path.move_to(data.initial_point.unwrap());
                    }
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                ctx.set_focus(ctx.widget_id());
                // Update cropping area and disable cropping mode
                if (data.cropping_mode || data.draw_rect_mode || data.draw_circle_mode
                    || data.draw_arrow_mode || data.draw_lines_mode || data.is_highliting
                    || data.is_inserting_text) && mouse_event.button == MouseButton::Left {

                    data.final_point = Some(data.mouse_position);

                    if let Some(rect) = data.current_rectangle.take() {
                        data.rectangles.push(rect);
                        ctx.request_paint();
                    }
                    if data.draw_lines_mode==true{
                        data.is_drawing=false;
                        data.all_positions.push(data.final_point.unwrap());
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
        if data.initial_point.is_some() && data.final_point.is_some() {
            if data.cropping_mode==true{

                let cropped_width = data.final_point.unwrap().x - data.initial_point.unwrap().x;
                let cropped_height = data.final_point.unwrap().y - data.initial_point.unwrap().y;
                let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                    data.image.as_ref().unwrap().width() as u32,
                    data.image.as_ref().unwrap().height() as u32,
                    rgba_data.to_vec(),
                )
                    .expect("Failed to create ImageBuffer"));

                let initial_height = data.initial_point.unwrap().y as u32;

                if cropped_width>20.0 && cropped_height >20.0{
                    let cropped_dyn_image = dynamic_image.crop_imm(data.initial_point.unwrap().x as u32, initial_height, cropped_width as u32, cropped_height as u32);
                    let rgba_data_cropped = cropped_dyn_image.clone().into_rgba8().to_vec();
                    let cropped_image_buf = Some(ImageBuf::from_raw(rgba_data_cropped.clone(), FormatImage::RgbaPremul, cropped_dyn_image.width() as usize, cropped_dyn_image.height() as usize));
                    let cropped_image = screenshots::Image::new(cropped_dyn_image.width() as u32, cropped_dyn_image.height() as u32, rgba_data_cropped);
                    data.image = cropped_image_buf.clone();
                    data.image_width = cropped_image.width();
                    data.image_height = cropped_image.height();
                    // Get image dimensions

                    editing_window(ctx,cropped_image,  data);
                }
                data.cropping_mode = !data.cropping_mode;
                data.initial_point = None;
                data.final_point = None;
                data.rectangles=Vec::new();
                data.current_rectangle=None;

            }
            let rgba=Rgba([data.selected_color.as_rgba8().0,data.selected_color.as_rgba8().1,data.selected_color.as_rgba8().2,data.selected_color.as_rgba8().3]);
            if data.draw_rect_mode==true {

                if data.initial_point.unwrap().x>data.final_point.unwrap().x{
                    let temp=data.initial_point;
                    data.initial_point=data.final_point;

                    data.final_point=temp;

                }
                let rect_width = data.final_point.unwrap().x - data.initial_point.unwrap().x;
                let rect_height = data.final_point.unwrap().y - data.initial_point.unwrap().y;

                let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                    data.image.as_ref().unwrap().width() as u32,
                    data.image.as_ref().unwrap().height() as u32,
                    rgba_data.to_vec(),
                ).expect("Failed to create ImageBuffer"));
                //let mut new_image=None;

                let new_image= Some(draw_hollow_rect(&dynamic_image,OtherRect::at(data.initial_point.unwrap().x as i32,data.initial_point.unwrap().y as i32).of_size(rect_width as u32,rect_height as u32),rgba));

                let rgba_data_drawn = new_image.clone().unwrap().into_raw().to_vec();
                let drawn_image_buf=Some(ImageBuf::from_raw(rgba_data_drawn.clone(),FormatImage::RgbaPremul,new_image.clone().unwrap().width() as usize,new_image.unwrap().height() as usize));
                data.image=drawn_image_buf.clone();
                let drawn_image=screenshots::Image::new(dynamic_image.width() as u32,dynamic_image.height() as u32,rgba_data_drawn);
                data.draw_rect_mode=false;
                data.initial_point = None;
                data.final_point = None;
                editing_window(ctx,drawn_image,  data);

            }
            if data.draw_circle_mode==true{
                let width = data.final_point.unwrap().x - data.initial_point.unwrap().x;
                let mut height = data.final_point.unwrap().y - data.initial_point.unwrap().y;
                if data.is_macos {
                    height += 100.0;
                }
                let radius= f64::sqrt(width*width+height*height);
                let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                    data.image.as_ref().unwrap().width() as u32,
                    data.image.as_ref().unwrap().height() as u32,
                    rgba_data.to_vec(),
                ).expect("Failed to create ImageBuffer"));
                let new_image= Some(draw_hollow_circle(&dynamic_image,(data.initial_point.unwrap().x as i32,data.initial_point.unwrap().y as i32),radius as i32, rgba));
                let rgba_data_drawn = new_image.clone().unwrap().into_raw().to_vec();
                let drawn_image_buf=Some(ImageBuf::from_raw(rgba_data_drawn.clone(),FormatImage::RgbaPremul,new_image.clone().unwrap().width() as usize,new_image.unwrap().height() as usize));
                data.image=drawn_image_buf.clone();
                let drawn_image=screenshots::Image::new(dynamic_image.width() as u32,dynamic_image.height() as u32,rgba_data_drawn);
                data.draw_circle_mode=false;
                data.initial_point = None;
                data.final_point = None;
                editing_window(ctx,drawn_image,  data);

            }
            if data.draw_arrow_mode==true{

                let angle = ((data.final_point.unwrap().y - data.initial_point.unwrap().y) as f64).atan2((data.final_point.unwrap().x - data.initial_point.unwrap().x) as f64);

                // Arrowhead properties
                let arrow_length = 20.0; // Adjust as needed
                let arrow_width = 10.0;  // Adjust as needed

                // Calculate the arrowhead points
                let arrow_x1 = data.final_point.unwrap().x as f64 - arrow_length * angle.cos();
                let arrow_y1 = data.final_point.unwrap().y as f64 - arrow_length * angle.sin();
                let arrow_x2 = arrow_x1 + arrow_width * (angle + PI / 6.0).cos();
                let arrow_y2 = arrow_y1 + arrow_width * (angle + PI / 6.0).sin();
                let arrow_x3 = arrow_x1 + arrow_width * (angle - PI / 6.0).cos();
                let arrow_y3 = arrow_y1 + arrow_width * (angle - PI / 6.0).sin();

                let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                    data.image.as_ref().unwrap().width() as u32,
                    data.image.as_ref().unwrap().height() as u32,
                    rgba_data.to_vec(),
                ).expect("Failed to create ImageBuffer"));
                let mut new_image= Some(draw_line_segment(&dynamic_image,(data.initial_point.unwrap().x as f32,data.initial_point.unwrap().y as f32),(data.final_point.unwrap().x as f32,data.final_point.unwrap().y as f32), rgba));
                // Draw the arrowhead
                new_image=Some(draw_line_segment(&new_image.unwrap(),(data.final_point.unwrap().x as f32, data.final_point.unwrap().y as f32), (arrow_x2 as f32, arrow_y2 as f32), rgba));
                new_image=Some(draw_line_segment(&new_image.unwrap(), (data.final_point.unwrap().x as f32, data.final_point.unwrap().y as f32), (arrow_x3 as f32, arrow_y3 as f32), rgba));

                let rgba_data_drawn = new_image.clone().unwrap().into_raw().to_vec();
                let drawn_image_buf=Some(ImageBuf::from_raw(rgba_data_drawn.clone(),FormatImage::RgbaPremul,new_image.clone().unwrap().width() as usize,new_image.unwrap().height() as usize));
                data.image=drawn_image_buf.clone();
                let drawn_image=screenshots::Image::new(dynamic_image.width() as u32,dynamic_image.height() as u32,rgba_data_drawn);
                data.draw_arrow_mode=false;
                data.initial_point = None;
                data.final_point = None;
                editing_window(ctx,drawn_image,  data);

            }
            if data.draw_lines_mode==true{
                let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                    data.image.as_ref().unwrap().width() as u32,
                    data.image.as_ref().unwrap().height() as u32,
                    rgba_data.to_vec(),
                ).expect("Failed to create ImageBuffer"));

                let mut new_image=None;
                for i in 0..data.all_positions.len(){
                    if i==0{
                        new_image=Some(draw_line_segment(&dynamic_image,(data.all_positions[i].x as f32,data.all_positions[i].y as f32),
                                                         (data.all_positions[i+1].x as f32,data.all_positions[i+1].y as f32),
                                                         rgba));
                    }
                    else if i<(data.all_positions.len()-1) {

                        new_image=Some(draw_line_segment(&new_image.unwrap(),(data.all_positions[i].x as f32,data.all_positions[i].y as f32),
                                                         (data.all_positions[i+1].x as f32,data.all_positions[i+1].y as f32),
                                                         rgba));
                    }
                }
                let rgba_data_drawn = new_image.clone().unwrap().into_raw().to_vec();
                let drawn_image_buf=Some(ImageBuf::from_raw(rgba_data_drawn.clone(),FormatImage::RgbaPremul,new_image.clone().unwrap().width() as usize,new_image.unwrap().height() as usize));
                data.image=drawn_image_buf.clone();
                data.all_positions.clear();
                data.draw_path=BezPath::new();
                let drawn_image=screenshots::Image::new(dynamic_image.width() as u32,dynamic_image.height() as u32,rgba_data_drawn);
                data.draw_lines_mode=false;
                data.initial_point = None;
                data.final_point = None;
                editing_window(ctx,drawn_image,  data);

            }
            if data.is_inserting_text{
                ctx.new_window(WindowDesc::new(build_input_box(data))
                    .set_position(Point::new(0 as f64, 0 as f64))
                );
                ctx.window().close();

            }
            if data.is_highliting{
                println!("highlithing");
                let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                    data.image.as_ref().unwrap().width() as u32,
                    data.image.as_ref().unwrap().height() as u32,
                    rgba_data.to_vec(),
                ).expect("Failed to create ImageBuffer"));
                let mut new_image = dynamic_image.clone();
                // Create an RGBA image with a transparent background
                let mut img = ImageBuffer::new(data.image_width, data.image_height);

                // Define the dimensions of the rectangle
                let rect_x = data.initial_point.unwrap().x as u32;
                let rect_y = data.initial_point.unwrap().y as u32;
                let rect_width = (data.final_point.unwrap().x - data.initial_point.unwrap().x) as u32;
                let rect_height = 20;

                let rect_color=Rgba([data.selected_color.as_rgba8().0,data.selected_color.as_rgba8().1,data.selected_color.as_rgba8().2,100]);

                // Draw the semi-transparent rectangle on the image
                for y in rect_y..rect_y + rect_height {
                    for x in rect_x..rect_x + rect_width {
                        img.put_pixel(x, y, rect_color);
                    }
                }

                image::imageops::overlay(&mut new_image, &img,0,0);

                let rgba_data_drawn = new_image.clone().into_rgba8().to_vec();
                let drawn_image_buf = Some(ImageBuf::from_raw(rgba_data_drawn.clone(), FormatImage::RgbaPremul, new_image.clone().width() as usize, new_image.height() as usize));
                data.image = drawn_image_buf.clone();
                let drawn_image = screenshots::Image::new(dynamic_image.clone().width() as u32, dynamic_image.height() as u32, rgba_data_drawn);
                data.is_highliting = false;
                data.initial_point = None;
                data.final_point = None;

                editing_window(ctx,drawn_image,data);
            }
            //ctx.submit_command(druid::commands::CLOSE_WINDOW);
        }
    }


    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) { }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {   }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        /*let size= ctx.size();
        ctx.stroke(size.to_rect(),&Color::YELLOW,1.0);*/

        if let Some(expandable_rect) = &data.current_rectangle {
            ctx.fill(expandable_rect.rect, &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Transparent background
            ctx.stroke(expandable_rect.rect, &data.selected_color, 0.5); // Yellow border
        }

        if data.draw_circle_mode==true{
            if data.initial_point.is_some() {
                let width = data.mouse_position.x - data.initial_point.unwrap().x;
                let mut height = data.mouse_position.y - data.initial_point.unwrap().y;
                if data.is_macos {
                    height += 100.0;
                }
                let radius = f64::sqrt(width * width + height * height);


                ctx.fill(druid::kurbo::Circle::new((data.initial_point.unwrap().x, data.initial_point.unwrap().y), radius), &Color::rgba(0.0, 0.0, 0.0, 0.0));
                ctx.stroke(druid::kurbo::Circle::new((data.initial_point.unwrap().x, data.initial_point.unwrap().y), radius), &data.selected_color, 0.5); // Yellow border
            }
        }

        if data.draw_arrow_mode==true{
            if data.initial_point.is_some() {

                let angle = ((data.mouse_position.y - data.initial_point.unwrap().y) as f64).atan2(
                    (data.mouse_position.x - data.initial_point.unwrap().x) as f64);

                // Arrowhead properties
                let arrow_length = 20.0; // Adjust as needed
                let arrow_width = 10.0;  // Adjust as needed

                // Calculate the arrowhead points
                let arrow_x1 = data.mouse_position.x as f64 - arrow_length * angle.cos();
                let arrow_y1 = data.mouse_position.y as f64 - arrow_length * angle.sin();
                let arrow_x2 = arrow_x1 + arrow_width * (angle + PI / 6.0).cos();
                let arrow_y2 = arrow_y1 + arrow_width * (angle + PI / 6.0).sin();
                let arrow_x3 = arrow_x1 + arrow_width * (angle - PI / 6.0).cos();
                let arrow_y3 = arrow_y1 + arrow_width * (angle - PI / 6.0).sin();

                ctx.fill(druid::kurbo::Line::new((data.mouse_position.x , data.mouse_position.y ), (arrow_x2 , arrow_y2)), &Color::rgba(0.0, 0.0, 0.0, 0.0));
                ctx.fill(druid::kurbo::Line::new((data.mouse_position.x , data.mouse_position.y ), (arrow_x3 , arrow_y3)), &Color::rgba(0.0, 0.0, 0.0, 0.0)); // Yellow border

                ctx.stroke(druid::kurbo::Line::new((data.mouse_position.x , data.mouse_position.y ), (arrow_x2 , arrow_y2)), &data.selected_color, 0.5);
                ctx.stroke(druid::kurbo::Line::new((data.mouse_position.x , data.mouse_position.y ), (arrow_x3 , arrow_y3)), &data.selected_color, 0.5); // Yellow border

                ctx.fill(druid::kurbo::Line::new((data.initial_point.unwrap().x, data.initial_point.unwrap().y), (data.mouse_position.x, data.mouse_position.y)), &Color::rgba(0.0, 0.0, 0.0, 0.0));
                ctx.stroke(druid::kurbo::Line::new((data.initial_point.unwrap().x, data.initial_point.unwrap().y), (data.mouse_position.x, data.mouse_position.y)), &data.selected_color, 0.5); // Yellow border
            }

        }
        if data.draw_lines_mode==true{
            ctx.stroke(&data.draw_path, &data.selected_color, 2.0);
        }
        if data.is_highliting{
            if data.initial_point.is_some() {
                let rect_x = data.initial_point.unwrap().x as f64;
                let rect_y = data.initial_point.unwrap().y as f64;
                let rect_width = (data.mouse_position.x - data.initial_point.unwrap().x) as f64;
                let rect_height = 20.0;

                let size = Size::new(rect_width, rect_height);
                let rect = Rect::from_origin_size((rect_x, rect_y), size);

                ctx.fill(rect, &Color::rgba(data.selected_color.as_rgba8().0 as f64, data.selected_color.as_rgba8().1 as f64, data.selected_color.as_rgba8().2 as f64, 0.3)); // Transparent background

                ctx.stroke(rect, &Color::rgba(data.selected_color.as_rgba8().0 as f64, data.selected_color.as_rgba8().1 as f64, data.selected_color.as_rgba8().2 as f64, 0.3), 0.5); // Yellow border
            }
        }
    }
}

