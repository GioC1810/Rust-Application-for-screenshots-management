    use std::cell::RefCell;
    use std::{fs,env};
    use std::f64::consts::PI;
    use std::fs::File;
    use std::io::BufWriter;
    use std::rc::Rc;
    use druid::widget::{Align, Axis, Button, Flex, Image, KnobStyle, Label, LineBreaking, Painter, Radio, RadioGroup, RangeSlider, SizedBox, Slider, TextBox, ViewSwitcher, ZStack};
    use druid::{Point, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget, WindowDesc, ImageBuf, KbKey, WidgetExt, Lens, Cursor, KeyOrValue, Selector};
    use druid::WindowState::Maximized;
    use druid::piet::ImageFormat as FormatImage;
    use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
    use screenshots::{Compression, Screen};
    use arboard::{Clipboard,ImageData};
    use druid::kurbo::BezPath;
    use image::ColorType::Rgba8;
    use imageproc::drawing::{Canvas,draw_line_segment, draw_hollow_rect, draw_hollow_circle, draw_text, draw_hollow_rect_mut};
    use imageproc::rect::Rect as OtherRect;
    use rusttype::{Font,Scale};

    pub struct SaveImageCommand {
        pub img_format: i32,
        pub img: Vec<u8>
    }

    pub const SAVE_IMAGE_COMMAND: Selector<SaveImageCommand> = Selector::new("save-image-command");

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
        pub file_path: String
    }

    impl Widget<AppState> for MyApp {

        fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {

            match event {
                Event::MouseMove(mouse_event) => {
                    data.mouse_position = mouse_event.pos;
                    ctx.request_paint(); // Request a redraw
                    ctx.set_cursor(&Cursor::Crosshair);
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

                let screenshot_width=data.final_point.unwrap().x-data.initial_point.unwrap().x ;
                let mut screenshot_height=data.final_point.unwrap().y-data.initial_point.unwrap().y ;

                let mut initial_height = data.initial_point.unwrap().y as i32;

                if env::consts::OS.eq("macos") {
                    initial_height += 55;
                }
                let image=Screen::from_point(data.screen.display_info.x,data.screen.display_info.y).unwrap().capture_area(data.initial_point.unwrap().x as i32, initial_height as i32,screenshot_width as u32, screenshot_height as u32).unwrap();

                let image_buf=ImageBuf::from_raw(image.rgba().clone(),FormatImage::RgbaPremul,image.width() as usize,image.height() as usize);
                data.image=Some(image_buf.clone());
                data.image_width=image.width();
                data.image_height=image.height();
                editing_window(ctx,image,  data);

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
                    if let Some(rect) = &mut data.current_rectangle {
                        rect.update(mouse_event.pos);
                    }
                    if data.draw_lines_mode==true{
                        if data.is_drawing==true{
                            data.all_positions.push(mouse_event.pos);
                            data.draw_path.line_to(data.mouse_position);
                        }
                    }

                    ctx.request_paint();
                }
                Event::MouseDown(mouse_event) => {
                    // Check if cropping mode is active and update cropping area
                    if (data.cropping_mode || data.draw_rect_mode || data.draw_circle_mode
                        || data.draw_arrow_mode || data.draw_lines_mode || data.is_highliting
                        || data.is_inserting_text) && mouse_event.button == MouseButton::Left {

                        data.initial_point = Some(data.mouse_position);
                        let expandable_rect = ExpandableRect::new(mouse_event.pos);

                        if data.cropping_mode || data.draw_rect_mode || data.is_inserting_text {
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
                    let cropped_image_buf = Some(ImageBuf::from_raw(rgba_data_cropped.clone(), FormatImage::RgbaPremul, cropped_dyn_image.width() as usize, cropped_dyn_image.height() as usize));
                    let cropped_image = screenshots::Image::new(cropped_dyn_image.width() as u32, cropped_dyn_image.height() as u32, rgba_data_cropped);

                    // Get image dimensions
                    data.cropping_mode = !data.cropping_mode;
                    data.initial_point = None;
                    data.final_point = None;
                    data.image = cropped_image_buf.clone();
                    data.image_width = cropped_image.width();
                    data.image_height = cropped_image.height();
                    editing_window(ctx,cropped_image,  data);

                }
                let rgba=Rgba([data.selected_color.as_rgba8().0,data.selected_color.as_rgba8().1,data.selected_color.as_rgba8().2,data.selected_color.as_rgba8().3]);
                if data.draw_rect_mode==true {
                    let rect_width = data.final_point.unwrap().x - data.initial_point.unwrap().x;
                    let mut rect_height = data.final_point.unwrap().y - data.initial_point.unwrap().y;
                    if env::consts::OS.eq("macos") {
                        rect_height += 100.0;
                    }
                    let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                    let mut dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                        data.image.as_ref().unwrap().width() as u32,
                        data.image.as_ref().unwrap().height() as u32,
                        rgba_data.to_vec(),
                    ).expect("Failed to create ImageBuffer"));
                    let mut new_image=None;

                    new_image= Some(draw_hollow_rect(&dynamic_image,OtherRect::at(data.initial_point.unwrap().x as i32,data.initial_point.unwrap().y as i32).of_size(rect_width as u32,rect_height as u32),rgba));

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
                    if env::consts::OS.eq("macos") {
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

                    let rgba_data = data.image.as_ref().unwrap().raw_pixels();

                    let mut dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
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
            }
        }


        fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) { }

        fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {   }

        fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppState, env: &Env) -> Size {
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
                    if env::consts::OS.eq("macos") {
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
                println!("nel paint delle linee");
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

    //ui generation functions

    pub fn initial_window(ctx:&mut EventCtx){
        ctx.new_window(WindowDesc::new(ui_builder())
            .window_size((550.0,200.0))
            .show_titlebar(true)
        );
        ctx.window().close();
    }
    pub fn screen_window(ctx:&mut EventCtx, data: &mut AppState){

        let screens=Screen::all().unwrap();


        let mut is_macos=false;
        if env::consts::OS.eq("macos") {
            is_macos = true;
        }


        ctx.new_window(WindowDesc::new(MyApp)
            .set_window_state(Maximized)
            .set_position(Point::new(data.screen.display_info.x as f64, data.screen.display_info.y as f64))
            .show_titlebar(is_macos)
            .transparent(true)
        );

        ctx.window().close();
    }

    pub fn editing_window(ctx:&mut EventCtx,img: screenshots::Image, my_data:&mut AppState) {
        ctx.new_window(WindowDesc::new(build_ui(img,  my_data))
            .set_window_state(Maximized));
        ctx.window().close();

    }
    pub fn ui_builder() -> impl Widget<AppState> {

        fn take_screenshot(ctx: &mut EventCtx, data: &mut AppState) {
            let mut is_macos = false;
            if env::consts::OS.eq("macos") {
                is_macos = true;
            }
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
           monitors_buttons = monitors_buttons.with_child(Button::new("Monitor ".to_owned() + &counter.to_string())
               .on_click(move |ctx, data: &mut AppState, _env| {
                   data.screen = screen;
                   take_screenshot(ctx,data);

           }));
            counter += 1;
        }

        /*let screen_button = Button::new("Screen")
            .on_click(|ctx, data: &mut AppState, _env| {
                take_screenshot(ctx,data);

            });*/

        let memorize_hotkey = Button::new("Add hotkey")
            .on_click(|ctx, data, _env| {
                let mut is_macos = false;
                if env::consts::OS.eq("macos") {
                    is_macos = true;
                }
                ctx.new_window(WindowDesc::new(build_hotkey_ui(data))
                    .title("digit hotkey")
                    .set_window_state(Maximized)
                    .set_position(Point::new(0 as f64, 0 as f64))
                    .show_titlebar(true)
                    .transparent(false)
                );
                ctx.window().close();
            });

        #[tokio::main]
        pub async fn timer_handling(ctx: &mut EventCtx,monitor_index: usize, time: u64) {
            // Sleep for time seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(time)).await;
            // take the screenshot
            println!("AFTER TIMER ----------");
        }

        let timer_button = Button::new("Start timer")
            .on_click(|ctx, data:&mut AppState, _: &Env| {
                println!("Timer button clicked");
                timer_handling(ctx,1,data.value as u64);
                take_screenshot(ctx,data);

            });

        let value = Flex::row()
            .with_child(Label::dynamic(|value: &AppState, _| {
                format!("Seconds: {:?}", value.value)
            }))
            .with_default_spacer()
            .with_child(ViewSwitcher::new(
                |data: &AppState, _| (0.0,10.0),
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

    fn build_ui(img: screenshots::Image, my_data:&mut AppState) -> impl Widget<AppState> {

        //let selected_color_label  = Label::new("Selected Color:");

        let color_circle = Painter::new(|ctx, data: &AppState, _env| {
            let circle_rect = Size{width:20.0, height:20.0}.to_rect();
            ctx.fill(circle_rect, &data.selected_color);
        }).fix_size(20.0, 20.0)
            .border(Color::BLACK, 2.0);

        let toggle_crop_button = Button::new("Toggle Crop")
            .on_click(|ctx, data:&mut AppState, _: &Env| {

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
        //let copy_to_clipboard_data = Rc::clone(&img_data);


        let save_as_png = Button::new("Save as png")
            .on_click(move |ctx, data: &mut AppState, _: &Env| {
                let mut is_macos = false;
                if env::consts::OS.eq("macos") {
                    is_macos = true;
                }
                let img_data = Rc::clone(&save_as_png_data);
                let img_cloned = img_data.borrow().to_owned();
                ctx.submit_command(SAVE_IMAGE_COMMAND.with(SaveImageCommand{img_format: 1,  img: img_cloned}));
            });

        let save_as_jpg = Button::new("Save as jpg")
            .on_click(move |ctx, data: &mut AppState, _: &Env| {
                let mut is_macos = false;
                if env::consts::OS.eq("macos") {
                    is_macos = true;
                }
                let img_data = Rc::clone(&save_as_jpg_data);
                let img_cloned = img_data.borrow().to_owned();
                ctx.submit_command(SAVE_IMAGE_COMMAND.with(SaveImageCommand{img_format: 1,  img: img_cloned}));
            });

        let save_as_gif = Button::new("Save as gif")
            .on_click(move |ctx, data: &mut AppState, _: &Env| {
                let mut is_macos = false;
                if env::consts::OS.eq("macos") {
                    is_macos = true;
                }
                let img_data = Rc::clone(&save_as_gif_data);
                let img_cloned = img_data.borrow().to_owned();
                ctx.submit_command(SAVE_IMAGE_COMMAND.with(SaveImageCommand{img_format: 1,  img: img_cloned}));
            });

        let copy_to_clipboard = Button::new("Copy to clipboard")
            .on_click(move |ctx, data: &mut AppState, _: &Env| {
                //let img_data = Rc::clone(&copy_to_clipboard_data);
                Clipboard::new().unwrap().set_image(ImageData { width: img.width() as usize, height: img.height() as usize, bytes: img.rgba().into() }).expect("Error in copying");
                initial_window(ctx);
            });

        let insert_input=Button::new("Insert Text")
            .on_click(move |ctx, data: &mut AppState, _: &Env| {
                data.mouse_position=Point::new(0.0, 0.0);
                data.initial_point=None;
                data.final_point=None;
                data.current_rectangle= None;
                data.rectangles= Vec::new();
                data.is_inserting_text=true;

            });
        let draw_rectangle= Button::new("⬜").on_click(move |ctx, data: &mut AppState, _: &Env| {
            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.draw_rect_mode= !data.draw_rect_mode;
        });
        let draw_circle= Button::new("⚪").on_click(move |ctx, data: &mut AppState, _: &Env| {
            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.draw_circle_mode= !data.draw_circle_mode;
        });
        let draw_arrow= Button::new("↘").on_click(move |ctx, data: &mut AppState, _: &Env| {
            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.draw_arrow_mode= !data.draw_arrow_mode;
        });
        let draw_lines= Button::new("✏").on_click(move |ctx, data: &mut AppState, _: &Env| {
            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.draw_lines_mode= !data.draw_lines_mode;
        });

        let highlight= Button::new("🖍").on_click(move |ctx, data: &mut AppState, _: &Env| {
            data.mouse_position=Point::new(0.0, 0.0);
            data.initial_point=None;
            data.final_point=None;
            data.current_rectangle= None;
            data.rectangles= Vec::new();
            data.is_highliting= true;
        });

        let change_color= Button::new("Change Color").on_click(move |ctx, data: &mut AppState, _: &Env| {
            ctx.new_window(WindowDesc::new(ColorGrid).window_size((500.0,200.0)).set_position((50.0,50.0)));
            ctx.window().close();
        });


        Flex::column()
            .with_child(toggle_crop_button)
            .with_child(Flex::row().with_child(save_as_png)
                .with_child(save_as_jpg)
                .with_child(save_as_gif)
            )
            .with_child(copy_to_clipboard)
            .with_child(Flex::row().with_child(draw_rectangle).with_child(draw_circle)
                .with_child(draw_arrow)
                .with_child(draw_lines)
                .with_child(highlight)
                .with_child(color_circle)
                .with_child(change_color)
                .with_child(insert_input)
            )
            .with_child(SizedBox::new(ZStack::new(Image::new(my_data.image.clone().unwrap()))
                .with_centered_child(Croptest))
                .width(my_data.image_width as f64)
                .height(my_data.image_height as f64))
            .with_child(KeyDetectionApp)

    }

    fn build_input_box(_my_data:&mut AppState) -> impl Widget<AppState> {
        let input_label = Label::new("Enter text:");

        let text_box = TextBox::new()
            .lens(AppState::input_text)
            .expand_width()
            .fix_height(30.0);

        let submit_button = Button::new("Submit")
            .on_click(|ctx, data: &mut AppState, _env| {
                // Handle the submitted text here
                let input_text = &data.input_text;
                println!("Input Text: {}", input_text);

                // You can perform further actions with the input text, e.g., send it to a server, process it, etc.
                // For now, we'll just print it to the console.

                let rgba_data = data.image.as_ref().unwrap().raw_pixels().to_vec();
                //let image=screenshots::Image::new(data.image_width as u32,data.image_height as u32,rgba_data);


                let rgba_data = data.image.as_ref().unwrap().raw_pixels();
                let rgba=Rgba([data.selected_color.as_rgba8().0,data.selected_color.as_rgba8().1,data.selected_color.as_rgba8().2,data.selected_color.as_rgba8().3]);

                let mut dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                    data.image.as_ref().unwrap().width() as u32,
                    data.image.as_ref().unwrap().height() as u32,
                    rgba_data.to_vec(),
                ).expect("Failed to create ImageBuffer"));
                let mut new_image=None;
                let font_data: &[u8] = include_bytes!("../Montserrat-Italic.otf");
                let font = Font::try_from_bytes(font_data);
                new_image= Some(draw_text(&dynamic_image, rgba, data.initial_point.unwrap().x as i32, data.initial_point.unwrap().y as i32, Scale::uniform((data.final_point.unwrap().y-data.initial_point.unwrap().y) as f32),
                                          &font.unwrap(), &*data.input_text));

                let rgba_data_drawn = new_image.clone().unwrap().into_raw().to_vec();
                let drawn_image_buf=Some(ImageBuf::from_raw(rgba_data_drawn.clone(),FormatImage::RgbaPremul,new_image.clone().unwrap().width() as usize,new_image.unwrap().height() as usize));
                data.image=drawn_image_buf.clone();
                let image=screenshots::Image::new(dynamic_image.width() as u32,dynamic_image.height() as u32,rgba_data_drawn);
                data.is_inserting_text=false;
                data.initial_point = None;
                data.final_point = None;


                /*ctx.new_window(WindowDesc::new(build_ui( image,  data)).set_window_state(Maximized));
                ctx.window().close();*/
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
    //struct RectangleDrawer;

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
    fn point_equal(r1: &Vec<Point>, r2: &Vec<Point>)->bool {
        if r1.len() != r2.len() {
            return false;
        }

        for i in 0..r1.len(){
            if r1[i].x==r2[i].x && r1[i].y==r2[i].y {
                return true;
            }
            else{
                return false;
            }
        }
        return false;
    }

    fn screen_equal(s1: &Screen, s2: &Screen)->bool {

        if s1.display_info.id == s2.display_info.id {
            return true
        }else{
            return false
        }
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
    pub struct HotKeyRecord;
    impl Widget<AppState> for HotKeyRecord {

        fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
            ctx.set_focus(ctx.widget_id());
            match event {
                Event::KeyDown(key_event) => {
                    if data.hotkey_to_register.keys.len() < 4 && key_event.key != KbKey::Escape
                        && (data.hotkey_to_register.keys.len() == 0 || key_event.key.ne(data.hotkey_to_register.keys.get(data.hotkey_to_register.keys.len()-1).unwrap())) {
                        data.hotkey_to_register.keys.push(key_event.key.clone());
                        println!("insert new hotkey: {:?}", data.hotkey_to_register.keys.get(data.hotkey_to_register.keys.len() - 1));
                    }

                }
                Event::KeyUp(key_event) => {
                    if key_event.key == KbKey::Escape {
                       initial_window(ctx);
                    }

                    else{
                        data.hotkeys.push(data.hotkey_to_register.clone());

                        for hotkey in &data.hotkeys{
                            print_hotkeys(&hotkey.keys);
                        }
                        println!("hoykeys registered after escape: ");
                        data.hotkey_to_register.keys.clear();
                        println!("hoykeys memorized: ");
                        initial_window(ctx);
                    }
                }
                _ => {}
            }
        }

        fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) { }

        fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {   }

        fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
            bc.max()
        }

        fn paint(&mut self, _ctx: &mut PaintCtx, _data: &AppState, _env: &Env) {
        }
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
                Event::Command(cmd) if cmd.is(SAVE_IMAGE_COMMAND) => {
                    if let image_prop = cmd.get_unchecked(SAVE_IMAGE_COMMAND) {
                        let path_name = match image_prop.img_format {
                            0 => "test_crop_values.png",
                            1 => "test_crop_values.jpg",
                            2 => "test_crop_values.gif",
                            _ => "",
                        };
                        ctx.new_window(WindowDesc::new(build_ui_save_file(path_name, image_prop.img.clone(), data, image_prop.img_format)));
                        ctx.window().close();
                    }
                }
                Event::KeyDown(key_event) => {
                    if data.actual_hotkey.keys.len() < 4 && (data.actual_hotkey.keys.len() == 0 || key_event.key.ne(data.actual_hotkey.keys.get(data.actual_hotkey.keys.len() - 1).unwrap())) {
                        println!("button pressed to trigger combination: {:?}", key_event.key);
                        data.actual_hotkey.keys.push(key_event.key.clone());
                        if find_hotkey_match(&data.actual_hotkey, &data.hotkeys) {
                            println!("combination triggered!!");
                            data.current_rectangle = None;
                            data.rectangles.clear();
                            data.cropping_mode = false;
                            data.initial_point = None;
                            data.final_point = None;
                            data.image_height=0;
                            data.image_width=0;


                            screen_window(ctx,data);
                            data.actual_hotkey.keys.clear();
                        }
                    } else if data.actual_hotkey.keys.len() == 4 {
                        println!("overreach the max number of button for the hotkey, start again!");
                        data.actual_hotkey.keys.clear();
                        initial_window(ctx);
                    }
                }
                Event::KeyUp(key_event) => {
                    println!("Hotkey pressed: {:?}", key_event.key);
                    data.actual_hotkey.keys.clear();
                    if key_event.key == KbKey::Escape{
                       initial_window(ctx);
                    }
                }
                _ => {}
            }
        }

        fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) { }

        fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {   }

        fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
            bc.max()
        }

        fn paint(&mut self, _ctx: &mut PaintCtx, _data: &AppState, _env: &Env) {
        }
    }

    fn build_hotkey_ui(data: &mut AppState) -> impl Widget<AppState> {
        // Create a widget that displays the hotkey items
        // You can use a Flex to lay out the hotkey items vertically
        let mut hotkey_list = Flex::column();

        // Add a button next to each hotkey item
        for (index, hotkey) in data.hotkeys.iter().enumerate() {
            let delete_button = Button::new(format!("Delete Hotkey {}", index + 1))
                .on_click(move |ctx, data: &mut AppState, _env| {
                    // Handle the click event to delete the corresponding item
                    data.hotkeys.remove(index);
                    ctx.new_window(WindowDesc::new(build_hotkey_ui(data))
                        .title("digit hotkey")
                        .set_window_state(Maximized)
                        .set_position(Point::new(0 as f64, 0 as f64))
                        .show_titlebar(true)
                        .transparent(false)
                    );
                    ctx.window().close();
                });

            hotkey_list = hotkey_list.with_child(
                Flex::row()
                    .with_child(Label::new(format!("Hotkey {}: {:?}", index + 1, hotkey.keys)))
                    .with_spacer(8.0) // Add some spacing between the label and button
                    .with_child(delete_button),
            );
        }

        Flex::column()
            .with_child(hotkey_list)
            .with_child(HotKeyRecord)
    }



    struct ColorGrid;

    impl Widget<AppState> for ColorGrid {
        fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
            if let Event::MouseDown(mouse_event) = event {
                let cell_size = ctx.size().width / 12.0;
                let click_pos = mouse_event.pos;
                let p = click_pos;
                let cell_index = (p.x / cell_size).floor() as usize;
                if cell_index < 12 {
                    let colors: [Color; 12] = [
                        Color::RED,
                        Color::GREEN,
                        Color::BLUE,
                        Color::YELLOW,
                        Color::PURPLE,
                        Color::GRAY,
                        Color::WHITE,
                        Color::BLACK,
                        Color::LIME,
                        Color::OLIVE,
                        Color::TEAL,
                        Color::NAVY
                    ];
                    data.selected_color = colors[cell_index];
                    println!("{:?}", data.selected_color);
                    ctx.request_paint(); // Trigger a repaint to show the selected color.
                    let rgba_data = data.image.as_ref().unwrap().raw_pixels().to_vec();
                    let image=screenshots::Image::new(data.image_width as u32,data.image_height as u32,rgba_data);


                    editing_window(ctx,image,  data);

                }
            }
        }
        fn paint(&mut self, paint_ctx: &mut PaintCtx, _data: &AppState, _env: &Env) {
            let cell_size = paint_ctx.size().width / 12.0;
            let colors: [Color; 12] = [
                Color::RED,
                Color::GREEN,
                Color::BLUE,
                Color::YELLOW,
                Color::PURPLE,
                Color::GRAY,
                Color::WHITE,
                Color::BLACK,
                Color::LIME,
                Color::OLIVE,
                Color::TEAL,
                Color::NAVY
            ];

            for (i, color) in colors.iter().enumerate() {
                let rect = Rect::from_origin_size(
                    (i as f64 * cell_size, 0.0),
                    (cell_size, paint_ctx.size().height),
                );
                paint_ctx.fill(rect, color);
            }
        }

        fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) { }

        fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {   }

        fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
            bc.max()
        }

    }


    fn build_ui_save_file(file_name: &str, img: Vec<u8>, data: &mut AppState, img_format: i32) -> impl Widget<AppState>{

        let button_save = Button::new("Save image")
            .on_click(move |ctx, data: &mut AppState, _env|{
                let extension = match img_format{
                    0 => ".png",
                    1 => ".jpg",
                    2 => ".gif",
                    _ => ".png"
                };
                let file_name = data.clone().file_path + extension;
                fs::write(file_name, img.clone()).expect("error in saving the file");
                ctx.new_window(WindowDesc::new(ui_builder())
                    .set_window_state(Maximized)
                    .set_position(Point::new(0 as f64, 0 as f64))
                    .show_titlebar(true)
                    .transparent(true)
                );
                ctx.window().close();
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


