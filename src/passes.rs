use juste::{Bound, Element, Genus, Overflow, Pad, Size, Tag};
use skia_safe::Size;

use crate::renderer::Cache;

pub fn first_pass(element: &mut Element, cache: &mut Cache) {
    match element.tag {
        Tag::None => return,
        Tag::Prime => {
            listen(element, cache);
            match &mut element.genus {
                Genus::Box {
                    style: _,
                    gravity: _,
                    size: _,
                    ceil: _,
                    children,
                } => {
                    recure_first_pass(children, cache);
                }
                Genus::Text {
                    style,
                    text,
                    font,
                    size,
                    spacing,
                } => {}
                Genus::Img {
                    style,
                    img_path,
                    scale,
                } => {}
            }
        }
        _ => {
            listen(element, cache);
            match &mut element.genus {
                Genus::Box {
                    style,
                    gravity: _,
                    size,
                    ceil,
                    children,
                } => {
                    if calculate_box(
                        &mut element.bound,
                        children,
                        cache,
                        &size,
                        &ceil,
                        &style.pad,
                    ) {
                        if let Overflow::Clip { active } = &mut element.bound.overflow {
                            *active = true
                        }
                    }
                    recure_first_pass(children, cache);
                }
                Genus::Text {
                    style,
                    text,
                    font,
                    size,
                    spacing,
                } => {}
                Genus::Img {
                    style,
                    img_path,
                    scale,
                } => {}
            }
        }
    }
}

fn put_shadow(element: &mut Element, pad: &Pad) {
    element.bound.shadow[0] = pad.left;
    element.bound.shadow[1] = pad.right;
    element.bound.shadow[2] = pad.top;
    element.bound.shadow[3] = pad.low;
}
fn calculate_box(
    bound: &mut Bound,
    children: &mut Vec<Element>,
    cache: &mut Cache,
    dim: &[Size; 2],
    ceil: &Option<[Size; 2]>,
    pad: &Pad,
) -> bool {
    bound.shadow = [pad.left, pad.right, pad.top, pad.low];
    let dim_x = dim[0];
    let dim_y = dim[1];

    bound.dim.x = match dim_x {
        Size::Window => cache.io.window_size.x,
        Size::Man(m) => m,
        Size::Child => children
            .iter()
            .map(|c| c.bound.dim.x + c.bound.shadow[0] + c.bound.shadow[1])
            .sum(),
    };

    bound.dim.y = match dim_y {
        Size::Window => cache.io.window_size.y,
        Size::Man(m) => m,
        Size::Child => children
            .iter()
            .map(|c| c.bound.dim.y + c.bound.shadow[2] + c.bound.shadow[3])
            .sum(),
    };
    let mut clip = false;
    if let Some(c) = ceil {
        let ceil_x = c[0];
        let ceil_y = c[1];

        match ceil_x {
            Size::Window => {
                if cache.io.window_size.x < bound.dim.x {
                    bound.dim.x = cache.io.window_size.x;
                    clip = true;
                }
            }
            Size::Man(m) => {
                let man = m;
                if man < bound.dim.x {
                    bound.dim.x = man;
                    clip = true;
                }
            }
            Size::Child => (),
        }

        match ceil_y {
            Size::Window => {
                if cache.io.window_size.y < bound.dim.y {
                    bound.dim.y = cache.io.window_size.y;
                    clip = true;
                }
            }
            Size::Man(m) => {
                let man = m;
                if man < bound.dim.y {
                    bound.dim.y = man;
                    clip = true;
                }
            }
            Size::Child => (),
        }
    }
    clip
}

pub fn clip(element: &mut Element, clip: bool) {
    match &mut element.bound.overflow {
        Overflow::Leak => return,
        Overflow::Clip { active } => *active = clip,
    }
}

fn recure_first_pass(children: &mut Vec<Element>, cache: &mut Cache) {
    let length = children.len();
    for i in 0..length {
        first_pass(&mut children[i], cache);
    }
}

fn listen(element: &mut Element, cache: &mut Cache) {
    if let Some((idx, msg)) = element.listen_io(&cache.io) {
        cache.bus.insert(idx, msg);
    }
    element.listen_signal(&mut cache.bus);
}
