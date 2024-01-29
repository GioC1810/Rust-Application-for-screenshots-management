use druid::{BoxConstraints, Data, Env, Event, EventCtx, KbKey, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Selector, Size, UpdateCtx, Widget, WindowDesc};
use druid::widget::{Button, Flex, Label};
use global_hotkey::{hotkey::{HotKey as HotKeyGlobal}};
use screenshots::Screen;

use crate::data::AppState;
use crate::ui_functions::{build_ui_save_file, initial_window, screen_window};

use crate::hashmap_hotkey::{HKdict, HKdictModifiers};

pub struct SaveImageCommand {
    pub img_format: i32,
    pub img: Vec<u8>
}

pub const SAVE_IMAGE_COMMAND: Selector<SaveImageCommand> = Selector::new("save-image-command");


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
pub fn hotkeys_equal(r1: &Vec<HotKey>, r2: &Vec<HotKey>) -> bool{
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
pub fn point_equal(r1: &Vec<Point>, r2: &Vec<Point>)->bool {
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

pub fn screen_equal(s1: &Screen, s2: &Screen)->bool {

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
        let hkdict =  HKdict::new();
        let hkdict_modifiers = HKdictModifiers::new();
        match event {
            Event::KeyDown(key_event) => {

                if data.hotkey_to_register.keys.len() < 2 && key_event.key != KbKey::Escape
                    && (data.hotkey_to_register.keys.len() == 0 || key_event.key.ne(data.hotkey_to_register.keys.get(data.hotkey_to_register.keys.len()-1).unwrap())) {
                    if data.hotkey_to_register.keys.len() == 0 && hkdict_modifiers.my_map.contains_key(&key_event.key.to_string()){
                        data.hotkey_to_register.keys.push(key_event.key.clone());
                    }
                    else if data.hotkey_to_register.keys.len() > 0 && hkdict.my_map.contains_key(&key_event.key.to_string().to_lowercase()) {
                        data.hotkey_to_register.keys.push(key_event.key.clone());
                    }
                    else {
                        println!("Invalid Hotkeys");
                    }
                }

            }
            Event::KeyUp(key_event) => {
                if key_event.key == KbKey::Escape {
                    println!("EscapePressed");
                    initial_window(ctx);
                }
                else{

                    if data.hotkey_to_register.keys.len() == 2 {
                        data.hotkeys.push(data.hotkey_to_register.clone());
                        let modifier = *hkdict_modifiers.my_map.get(&data.hotkey_to_register.keys[0].to_string()).unwrap();
                        let key = *hkdict.my_map.get(&data.hotkey_to_register.keys[1].to_string().to_lowercase()).unwrap();

                        let hotkey = HotKeyGlobal::new(Some(modifier), key);
                        data.hotkey_manager.register(hotkey).expect("error in registering hotkey");
                    }

                    data.hotkey_to_register.keys.clear();
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

pub struct KeyDetectionApp;
impl Widget<AppState> for KeyDetectionApp {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        ctx.set_focus(ctx.widget_id());
        match event {
            Event::Command(cmd) if cmd.is(SAVE_IMAGE_COMMAND) => {
                let image_prop = cmd.get_unchecked(SAVE_IMAGE_COMMAND);
                ctx.new_window(WindowDesc::new(build_ui_save_file(image_prop.img.clone(), data, image_prop.img_format)));
                ctx.window().close();
            }
            Event::KeyDown(key_event) => {
                if data.actual_hotkey.keys.len() < 4 && (data.actual_hotkey.keys.len() == 0 || key_event.key.ne(data.actual_hotkey.keys.get(data.actual_hotkey.keys.len() - 1).unwrap())) {
                    data.actual_hotkey.keys.push(key_event.key.clone());
                    if find_hotkey_match(&data.actual_hotkey, &data.hotkeys) {
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
                    data.actual_hotkey.keys.clear();
                    initial_window(ctx);
                }
            }
            Event::KeyUp(key_event) => {
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

pub fn build_hotkey_ui(data: &mut AppState) -> impl Widget<AppState> {
    // Create a widget that displays the hotkey items
    // You can use a Flex to lay out the hotkey items vertically
    let mut hotkey_list = Flex::column();


    // Add a button next to each hotkey item
    for (index, hotkey) in data.hotkeys.iter().enumerate() {
        let hotkey_cloned=hotkey.clone();
        let delete_button = Button::new(format!("Delete Hotkey {}", index + 1))
            .on_click(move |ctx, d: &mut AppState, _env| {

                let hkdict =  HKdict::new();
                let hkdict_modifiers = HKdictModifiers::new();
                let modifier = hkdict_modifiers.my_map.get(&hotkey_cloned.keys[0].to_string()).unwrap().clone();
                let key = hkdict.my_map.get(&hotkey_cloned.keys[1].to_string().to_lowercase()).unwrap().clone();

                let hotkey_to_cancel = HotKeyGlobal::new(Some(modifier), key);
                d.hotkey_manager.unregister(hotkey_to_cancel).expect("error in registering hotkey");

                // Handle the click event to delete the corresponding item
                d.hotkeys.remove(index);
                ctx.new_window(WindowDesc::new(build_hotkey_ui(d))
                    .title("Digit hotkey")
                    .window_size((500.0,200.0))
                    .set_always_on_top(true)
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
        .with_child(Label::new("Choose one from ( Shift, Alt, Control ) + a letter"))
        .with_child(hotkey_list)
        .with_child(HotKeyRecord)
}
