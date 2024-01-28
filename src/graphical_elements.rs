
use druid::{Point, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,  PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget};

use crate::ui_functions::editing_window;
use crate::data::{AppState};

pub struct ColorGrid;

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

//Rectangle drawer section
#[derive(Clone)]
pub struct ExpandableRect {
    pub rect: Rect,
}

impl ExpandableRect {
    pub fn new(origin: Point) -> Self {
        ExpandableRect {
            rect: Rect::from_origin_size(origin, Size::ZERO),
        }
    }
    pub fn update(&mut self, new_point: Point, init_point:Point) {
        let width = (new_point.x - init_point.x).abs();
        let height = (new_point.y - init_point.y).abs();
        let size = Size::new(width, height);
        //println!("update");
        if new_point.x < init_point.x &&  new_point.y < init_point.y{
            self.rect = Rect::from_origin_size(new_point, size);
        }
        else if new_point.x < init_point.x && new_point.y > init_point.y{
            self.rect = Rect::from_origin_size((new_point.x,init_point.y), size);
        }
        else if new_point.x > init_point.x && new_point.y < init_point.y{
            self.rect = Rect::from_origin_size((init_point.x,new_point.y), size);
        }
        else{
            self.rect = Rect::from_origin_size(init_point, size);
        }
    }
}


impl Data for ExpandableRect {
    fn same(&self, other: &Self) -> bool {
        self.rect.same(&other.rect)
    }
}

pub fn rectangles_equal(r1: &Vec<ExpandableRect>, r2: &Vec<ExpandableRect>) -> bool {
    r1.len() == r2.len() && r1.iter().zip(r2).all(|(a, b)| a.same(b))
}
