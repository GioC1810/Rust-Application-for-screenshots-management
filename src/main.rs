use druid::{AppLauncher, Point, WindowDesc, KeyEvent};
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
            cropping_mode: true,
            cropped_area: None,
            image: None,
            hotkeys: Vec::new(),
            hotkey_to_register: HotKey::new(),
            actual_hotkey: HotKey::new()
        })
        .expect("Failed to launch app");

    
}



