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
use druid::UnitPoint;
use druid::UpdateCtx;
use druid::Widget;
use druid::WidgetExt;
use druid::commands;
use druid::widget::Image;
use druid::widget::LineBreaking;
use druid::widget::Padding;
use druid::widget::SizedBox;
use druid::widget::{Button, Flex, Label};
use druid::Data;
use druid::WidgetId;
use druid::PaintCtx;
use druid::Size;
use crate::metadata::Info;
use crate::metadata::PlayerCommand;

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
    let close = Button::new("X").on_click(|_ctx, _data: &mut Info, _env| {
        _ctx.submit_command(commands::CLOSE_WINDOW);
    }).fix_size(30., 30.).align_vertical(UnitPoint::TOP);

    let layout = Flex::row().with_child(Padding::new(10.0, DynImage::new()))
        .with_default_spacer()
        .with_child(SizedBox::new(Flex::column()
            .with_child(title_label)
            .with_default_spacer()
            .with_child(artists)
            .with_child(album)
            .with_child(Flex::row().with_child(back).with_default_spacer()
                .with_child(pause).with_default_spacer()
                .with_child(next).with_default_spacer()
            )).width(260.)
        ).with_child(close);
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
