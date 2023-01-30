use std::sync::mpsc;
use druid::BoxConstraints;
use druid::Color;
use druid::Env;
use druid::Event;
use druid::EventCtx;
use druid::FontDescriptor;
use druid::FontFamily;
use druid::FontWeight;
use druid::LayoutCtx;
use druid::LifeCycle;
use druid::LifeCycleCtx;
use druid::Screen;
use druid::UnitPoint;
use druid::UpdateCtx;
use druid::Vec2;
use druid::Widget;
use druid::WidgetExt;
use druid::commands;
use druid::widget::Image;
use druid::widget::LineBreaking;
use druid::widget::Padding;
use druid::widget::SizedBox;
use druid::widget::ZStack;
use druid::widget::{Button, Flex, Label};
use druid::Data;
use druid::WidgetId;
use druid::PaintCtx;
use druid::Size;
use druid::widget::{Svg, SvgData};
use crate::metadata::Info;
use crate::metadata::PlayerCommand;
use crate::metadata::ScreenLoc;

pub fn ui_builder(sx: mpsc::Sender<PlayerCommand>) -> impl Widget<Info> {
    // Fonts
    let title_font = FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(32.0);
    let reg_font = FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(20.0);

    let _img = Label::new(|data: &Info, _: &Env| data.art_url.to_string());
    /*
    let test = SizedBox::new(|data: &Info| {
            if let Some(art) = data.art {
                Image::new(*art)
            } else {
                Image::new(ImageBuf::default())
            }
    });
    */

    // Labels
    let title_label = Label::new(|data: &Info, _: &Env| data.title.to_string())
        .with_font(title_font)
        .with_line_break_mode(LineBreaking::Clip)
        .center()
        .expand_width();
    let artists =  SizedBox::new(Label::new(|data: &Info, _: &Env| data.artists.to_string())
        .with_font(reg_font.clone())
        .with_line_break_mode(LineBreaking::Clip)
        .center())
        .expand_width();
    let album =  Label::new(|data: &Info, _: &Env| {
        data.album_name.to_string()
    })
        .with_font(reg_font)
        .with_line_break_mode(LineBreaking::Clip)
        .center()
        .expand_width();
    
    //Buttons
    let sx1 = sx.clone();
    let sx2 = sx.clone();
    let back = Button::new("Previous").on_click(move |_, _, _| sx2.send(PlayerCommand::Prev).unwrap_or_default())
        .disabled_if(|data: &Info, _| data.can_prev);
    let pause = Button::dynamic(|data: &Info, _| {
        if data.is_paused {
            "Play".to_string()
        } else {
            "Pause".to_string()
        }
    }).on_click(move |_, _, _| sx1.send(PlayerCommand::Pause).unwrap_or_default()).disabled_if(|data: &Info, _| data.can_pause);
    let next = Button::new("Next").on_click(move |_, _, _| sx.send(PlayerCommand::Next).unwrap_or_default())
        .disabled_if(|data: &Info, _| {
            data.can_next
        });

    let close_svg: SvgData = CLOSE_SVG.parse().unwrap_or_else(|_| SvgData::default());
    let better_close = Svg::new(close_svg).on_click(|ctx, _data: &mut Info, _env| {
        ctx.submit_command(commands::QUIT_APP);
    }).fix_size(16., 16.).align_vertical(UnitPoint::TOP);

    let min_svg: SvgData = MIN_SVG.parse().unwrap();
    let minimize = Svg::new(min_svg).on_click(move |ctx, _data: &mut Info, _env| {
        if _data.minimize == false {
            ctx.window().set_size((20., 40.));
            ctx.window().set_position(place_widget(20., 40., &_data.location, _data.offset.clone()));
        } else {
            ctx.window().set_size((460., 160.));
            ctx.window().set_position(place_widget(460., 160., &_data.location, (0., 0.)));
        }

        _data.minimize = !_data.minimize;
    }).fix_size(16., 16.).align_vertical(UnitPoint::BOTTOM);

    let layout = ZStack::new(
        Flex::row()
        .with_child(Padding::new(5.0, DynImage::new()))
        .with_default_spacer()
        .with_flex_child(Flex::column()
            .with_flex_child(title_label, 0.8)
            .with_default_spacer()
            .with_child(artists)
            .with_child(album)
            .with_child(Flex::row().with_child(back).with_default_spacer()
                .with_child(pause).with_default_spacer()
                .with_child(next).with_default_spacer()
            ), 1.
        )
    ).with_child(
        Flex::column()
        .with_child(better_close).with_flex_spacer(0.1)
        .with_child(minimize), Vec2::new(1., 1.), Vec2::ZERO, UnitPoint::RIGHT, Vec2::ZERO);
    return layout;
}

struct DynImage {
    inner: Box<dyn Widget<Info>>,
}

impl DynImage {
    fn new() -> DynImage {
        DynImage {
            inner: SizedBox::empty().boxed(),
        }
    }

    fn rebuild_inner(&mut self, data: &Info) {
        self.inner = build_widget(data);
    }
}

fn build_widget(data: &Info) -> Box<dyn Widget<Info>> {
    if data.art.is_some() {
        let img = Image::new(data.art.as_ref().unwrap().as_ref().clone());
        let sized = SizedBox::new(img);
        sized.border(Color::grey(0.6), 2.0).center().boxed()
    } else {
        let sized = SizedBox::empty();
        sized.border(Color::grey(0.6), 2.0).center().boxed()
    }
}

impl Widget<Info> for DynImage {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Info, env: &Env) {
        self.inner.event(ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Info, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.rebuild_inner(data);
        }
        self.inner.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Info, data: &Info, _env: &Env) {
        if !old_data.same(data) && !(old_data.art_url == data.art_url) {
            self.rebuild_inner(data);
            ctx.children_changed();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Info,
        env: &Env,
    ) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Info, env: &Env) {
        self.inner.paint(ctx, data, env)
    }

    fn id(&self) -> Option<WidgetId> {
        self.inner.id()
    }
}

pub fn place_widget(width: f64, height: f64, pos: &ScreenLoc, offset: (f64, f64)) -> (f64, f64) {
    let rect = Screen::get_display_rect();
    match pos {
        ScreenLoc::TopRight => (rect.x1-width -offset.0, rect.y0 +offset.1),
        ScreenLoc::TopLeft => (rect.x0 +offset.0, rect.y0 +offset.1),
        ScreenLoc::BottomLeft => (rect.x0 +offset.0, rect.y1-height -offset.1),
        ScreenLoc::BottomRight => (rect.x1-width -offset.0, rect.y1-height -offset.1),
    }
}

const CLOSE_SVG: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!-- Svg Vector Icons : http://www.onlinewebfonts.com/icon -->

<svg
   version="1.1"
   x="0px"
   y="0px"
   viewBox="0 0 1000 1000"
   enable-background="new 0 0 1000 1000"
   xml:space="preserve"
   id="svg78"
   xmlns="http://www.w3.org/2000/svg"
   xmlns:svg="http://www.w3.org/2000/svg"><defs
   id="defs82" />
<metadata
   id="metadata72"> Svg Vector Icons : http://www.onlinewebfonts.com/icon </metadata>
<g
   id="g76"
   style="fill:#ffffff"><path
     d="M630.8,497.3l317.1-330.7c35.4-36.9,34.1-95.5-2.8-130.8c-36.8-35.4-95.4-34.1-130.8,2.8L497.2,369.2L189.3,73.9c-36.8-35.4-95.4-34.1-130.8,2.7l-0.1,0.1c-35.3,36.9-34,95.4,2.8,130.7l308,295.3L52.1,833.4c-35.4,36.9-34.1,95.5,2.8,130.8c36.9,35.4,95.5,34.1,130.8-2.7l317.1-330.7l308,295.2c36.8,35.4,95.3,34.1,130.6-2.6l0.3-0.3c35.2-36.9,33.9-95.3-2.9-130.6L630.8,497.3z"
     id="path74"
     style="fill:#ffffff" /></g>
</svg>"#;

const MIN_SVG: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!-- Created with Inkscape (http://www.inkscape.org/) -->

<svg width="48" height="48" viewBox="0 0 12.7 12.7" version="1.1" id="svg5" xmlns="http://www.w3.org/2000/svg" xmlns:svg="http://www.w3.org/2000/svg">
  <defs
     id="defs2" />
  <g id="layer1"> 
  <path
       id="path1219"
       style="fill:#FFFFFF;stroke-width:0.00499999;stroke-linejoin:bevel;stroke-dashoffset:0.30274"
       d="m -1.369191,-1.3834881 -0.019878,1.76759852 c 0,0 7.9619815,4.02615468 7.97265,4.43537948 0.010668,0.4092248 -7.9936825,3.9660384 -7.9936825,3.9660384 l 0.040913,2.2265177 c 0,0 10.707945,-5.7424352 10.7128624,-6.1973899 C 9.3485888,4.3597014 -1.369191,-1.3834881 -1.369191,-1.3834881 Z"
       transform="matrix(1.1825419,0,0,1.0234786,1.6511651,1.4216549)" />
  </g>
</svg>"#;
