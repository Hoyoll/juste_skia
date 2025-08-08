use std::collections::HashMap;

use juste::{
    element::{Bound, Element, Message, Tag},
    genus::{Box, Genus},
    io::Io,
    style::{Gravity, Size, Style},
    util::Vec2,
};
// use juste::{
//     Bound, Element, Font, Genus, Gravity, Io, Message, Pad, Size, Src, Style, Tag, style::Sheet,
// };
use juste_skia::run;
use skia_safe::FourByteTag;
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
        genus: Genus::Box(Box {
            style: Style::new(),
            gravity: Gravity::Horizontal,
            size: Vec2::new(Size::Man(10.0), Size::Man(10.0)),
            ceil: None,
            children: None,
        }),
        bound: Bound::new(),
        listener: None,
    };
    b
}
fn main() {
    let img = Element {
        tag: Tag::Def,
        genus: Genus::Img(),
        bound: Bound::new(),
        listener: None,
    };
    run(b, attr, HashMap::new());
}
