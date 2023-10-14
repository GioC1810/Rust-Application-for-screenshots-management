use druid::{AppLauncher, Point, WindowDesc, KeyEvent, Color};
use druid::kurbo::BezPath;
use druid::WindowState::Maximized;
use gui_image::{AppState, HotKey, KeyDetectionApp, ui_builder};

fn main() {

        let main_window = WindowDesc::new(ui_builder())
            .transparent(true)
            .set_window_state(Maximized)
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
                    all_positions:Vec::new(),
                    draw_path:BezPath::new(),
                    image: None,
                    hotkeys: Vec::new(),
                    hotkey_to_register: HotKey::new(),
                    actual_hotkey: HotKey::new(),
                    image_width:0,
                    image_height:0
            })
            .expect("Failed to launch app");


}



