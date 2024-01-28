use druid::{AppLauncher, Point, WindowDesc, Color, Application, AppDelegate, commands, DelegateCtx, Handled, Selector, WindowId, WindowHandle};
use druid::kurbo::BezPath;
use screenshots::{Screen};
use gui_image::{AppState, HotKey, initial_window, MyApp, screen_window, ui_builder};
use std::{env, thread};
use std::sync::Arc;
use std::time::Duration;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{HotKey as HotKeyGlobal, Modifiers, Code}};
use druid::WindowState::{Maximized, Restored};
use std::sync::mpsc::channel;
use druid::commands::{CLOSE_ALL_WINDOWS, CLOSE_WINDOW, HIDE_WINDOW, QUIT_APP, SHOW_WINDOW};
use druid::{Command, Env, Target};
use global_hotkey::HotKeyState::Pressed;

pub struct Delegate{
    last_window_id:Option<WindowId>
}

const HOTKEY_SCREEN: Selector<u32> = Selector::new("hotkey screen");

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) ->Handled{

        if cmd.is(HOTKEY_SCREEN) {

            // Puoi eseguire altre azioni prima di chiudere la finestra
            // the event keep processing and the window is closed

            //ctx.submit_command(CLOSE_WINDOW);
            data.rectangles=Vec::new();
            data.current_rectangle=None;
            data.image_width=0;
            data.image_height=0;
            let screen_window_hotkey=WindowDesc::new(MyApp)
                .set_window_state(Maximized)
                .set_position(Point::new(data.screen.display_info.x as f64, data.screen.display_info.y as f64))
                .show_titlebar(data.is_macos)
                .transparent(true);
            println!("{:?}",screen_window_hotkey.id);


            ctx.new_window(
                screen_window_hotkey
            );

            println!("in the new command");
            return Handled::No;
        }
        else if cmd.is(commands::CLOSE_WINDOW) {

            println!("Finestra chiusa!");
            // Puoi eseguire altre azioni prima di chiudere la finestra
            // the event keep processing and the window is closed

            return Handled::No;
        }
        else if cmd.is(commands::CLOSE_ALL_WINDOWS) {

            println!("Finestra chiusa!");
            // Puoi eseguire altre azioni prima di chiudere la finestra
            // the event keep processing and the window is closed

            return Handled::No;
        }

        Handled::No
    }
    fn window_added(
        &mut self,
        id: WindowId,
        handle: WindowHandle,
        data: &mut AppState,
        env: &Env,
        ctx: &mut DelegateCtx<'_>
    ){
        data.rectangles=Vec::new();
        data.current_rectangle=None;
        self.last_window_id=Some(id);
        println!("{:?}", id);
    }
    fn window_removed(&mut self, id: WindowId, data: &mut AppState, env: &Env, ctx: &mut DelegateCtx) {
        //println!("window_removed");
        data.initial_point=None;
        data.final_point=None;
        data.current_rectangle=None;
    }
}

fn main() {
    let mut is_macos = false;
    if env::consts::OS.eq("macos") {
        is_macos = true;
    }
    let main_window = WindowDesc::new(ui_builder())
        .window_size((550.0,200.0))
        .title("Screenshot App")
        .show_titlebar(true);

    let delegate= Delegate{last_window_id:None};
    /*let delegate=Arc::new(Delegate{last_window_id:None});
    let delegate1=delegate.clone();*/
    let window_id= Arc::new(main_window.id);
    let launcher= AppLauncher::with_window(main_window).delegate(delegate);
    let event_sink= launcher.get_external_handle();

    let hotkey_manager =GlobalHotKeyManager::new().unwrap();
    //let hotkey = HotKeyGlobal::new(Some(Modifiers::SHIFT), Code::KeyD);
    //let hotkey2 = HotKeyGlobal::new(Some(Modifiers::SHIFT), Code::KeyB);
    //hotkey_manager.register(hotkey).expect("error in registering the hotkey");
    //hotkey_manager.register(hotkey2).expect("error in registering the hotkey");


    let id=window_id.clone();
    println!("{:?}", *id);
    thread::spawn(move || {
        loop {
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state==Pressed{
                    event_sink.submit_command(HOTKEY_SCREEN,1,Target::Window(*id)).unwrap();
                    event_sink.submit_command(CLOSE_ALL_WINDOWS,Box::new(()),Target::Global);
                }

            }
            thread::sleep(Duration::from_secs(1));
    }});

    println!("here");
    launcher.launch(AppState{
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
        hotkey_manager: Arc::new(hotkey_manager),
        is_macos
    })
        .expect("Failed to launch app");
}



