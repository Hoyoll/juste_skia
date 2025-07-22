use juste::{Bound, Element, Font, Genus, Gravity, Io, Message, Pad, Size, Src, Style, Tag};
use juste_skia::run;
use winit::window::WindowAttributes;

fn laop(element: &mut Element, _io: &Io) -> Option<(Tag, Message)> {
    element.bound.angle += 0.5;

    None
}

fn box_loop(element: &mut Element, _io: &Io) -> Option<(Tag, Message)> {
    element.bound.angle -= 1.0;
    element.bound.overflow.make_clip();
    None
}

fn fallback(_io: &Io) -> Element {
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
            size: [Size::Window, Size::Man(30.0)],
            ceil: None,
            children: vec![],
        },
        bound: Bound::new(),
        signal_listener: None,
        io_listener: None,
    };
    b
}
fn main() {
    let img = Element {
        tag: Tag::Def,
        genus: Genus::Img {
            style: Style {
                pad: Pad::new(),
                color: [0, 0, 0, 255],
            },
            img_path: Src::Url("https://placehold.co/800@3x.png".to_string()),
            scale: 0.2,
            fallback: Some(fallback),
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
            gravity: Gravity::Horizontal,
            size: [Size::Window, Size::Window],
            ceil: None,
            children: vec![element, img],
        },
        bound: Bound::new(),
        signal_listener: None,
        io_listener: Some(box_loop),
    };
    let attr = WindowAttributes::default();
    run(b, attr);
}
