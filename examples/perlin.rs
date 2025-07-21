use juste::{Bound, Element, Font, Genus, Gravity, Io, Message, Pad, Process, Size, Style, Tag};
use juste_skia::run;
use winit::window::WindowAttributes;

struct Sp {}

impl Process for Sp {
    fn transform(&mut self, _element: &mut Element) {}
    fn destroy(&mut self) {}
    fn message(&mut self, _message: Message) {}
}

fn laop(element: &mut Element, _io: &Io) -> Option<(Tag, Message)> {
    element.bound.angle += 0.0;
    let msg: Message = Message::Proc(Box::new(Sp {}));
    Some((Tag::Id(2), msg))
}

fn box_loop(element: &mut Element, _io: &Io) -> Option<(Tag, Message)> {
    element.bound.angle -= 1.0;
    element.bound.overflow.make_clip();
    None
}

fn size(io: &Io) -> f32 {
    io.window_size.y / 10.0
}

fn pillar() -> Element {
    let b = Element {
        tag: Tag::Def,
        genus: Genus::Box {
            style: Style {
                pad: Pad {
                    top: 0.0,
                    low: 10.0,
                    right: 0.0,
                    left: 0.0,
                },
                color: [0, 0, 255, 255],
            },
            gravity: Gravity::Horizontal,
            size: [Size::Window, Size::Func(size)],
            ceil: None,
            children: vec![],
        },
        bound: Bound::new(),
        signal_listener: None,
        io_listener: None,
    };
    b
}

fn more_pillar(size: usize) -> Vec<Element> {
    let mut v = Vec::new();
    for _i in 0..size {
        v.push(pillar());
    }
    v
}

fn main() {
    let img = Element {
        tag: Tag::Def,
        genus: Genus::Img {
            style: Style {
                pad: Pad::new(),
                color: [0, 0, 0, 255],
            },
            img_path: "C:/Users/HP/Pictures/waifu/ptsd.jpeg".to_string(),
            scale: 1.0,
        },
        bound: Bound::new(),
        signal_listener: None,
        io_listener: None,
    };
    let element = Element {
        tag: Tag::Def,
        genus: Genus::Text {
            style: Style {
                pad: Pad {
                    top: 20.0,
                    low: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                color: [0, 0, 255, 255],
            },
            text: String::from("String!"),
            font: Font::Sys("Arial", juste::Mode::Normal),
            size: 100.0,
            spacing: 10.0,
        },
        bound: Bound::new(),
        signal_listener: None,
        io_listener: Some(laop),
    };
    let b = Element {
        tag: Tag::Def,
        genus: Genus::Box {
            style: Style {
                pad: Pad {
                    top: 0.0,
                    low: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                color: [255, 0, 0, 255],
            },
            gravity: Gravity::Vertical,
            size: [Size::Window, Size::Window],
            ceil: None,
            children: more_pillar(5),
        },
        bound: Bound::new(),
        signal_listener: None,
        io_listener: None,
    };
    let attr = WindowAttributes::default();
    run(b, attr);
}
