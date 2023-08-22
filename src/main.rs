use druid::{AppLauncher, Point, WindowDesc};
use gui_image::{AppState, ui_builder};


fn main() {

    let main_window = WindowDesc::new(ui_builder())
        .transparent(true)
        .title("Screenshot App")
        .show_titlebar(true);

    AppLauncher::with_window(main_window)
        .launch(AppState{
            mouse_position: Point::new(0.0, 0.0),
            initial_point: None,
            final_point: None,
            current_rectangle:None,
            rectangles: Vec::new()
        })
        .expect("Failed to launch app");
}



