use druid::{AppLauncher, Point, WindowDesc, Color};
use druid::kurbo::BezPath;
use screenshots::{Screen};
use gui_image::{AppState, HotKey, ui_builder};
use std::env;
fn main() {
        let mut is_macos = false;
        if env::consts::OS.eq("macos") {
            is_macos = true;
        }
        let main_window = WindowDesc::new(ui_builder())
            .window_size((550.0,200.0))
            .title("Screenshot App")
            .show_titlebar(true);

        AppLauncher::with_window(main_window)
            .launch(AppState{
                    mouse_position: Point::new(0.0, 0.0),
                    initial_point: None,
                    final_point: None,
                    current_rectangle:None,
                    rectangles: Vec::new(),
                    cropping_mode: false,
                    draw_rect_mode: false,
                    draw_circle_mode:false,
                    draw_arrow_mode: false,
                    draw_lines_mode:false,
                    is_drawing:false,
                    is_highliting:false,
                    is_inserting_text:false,
                    input_text:String::new(),
                    selected_color:Color::RED,
                    value:0.0,
                    all_positions:Vec::new(),
                    draw_path:BezPath::new(),
                    image: None,
                    hotkeys: Vec::new(),
                    hotkey_to_register: HotKey::new(),
                    actual_hotkey: HotKey::new(),
                    image_width:0,
                    image_height:0,
                    screen: Screen::from_point(0, 0).unwrap(),
                    file_path: String::new(),
                    screen_saved_counter: 0,
                    is_macos:is_macos
            })
            .expect("Failed to launch app");

}



