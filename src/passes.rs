use juste::{Bound, Element, Genus, Gravity, Size, Style, Tag, Vec2};
use skia_safe::{Canvas, ClipOp, Font, Image, Matrix, Paint, Path, Point, Rect};

use crate::renderer::Cache;

pub fn first_pass(element: &mut Element, cache: &mut Cache) {
    match element.tag {
        Tag::None => return,
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
                    recure_pass(children, cache);
                    if calculate_box(&mut element.bound, children, cache, size, ceil, style) {
                        element.bound.overflow.make_clip();
                    }
                }
                Genus::Text {
                    style,
                    text,
                    font,
                    size,
                    spacing: _,
                } => {
                    calculate_text(&mut element.bound, cache, font, text, *size, style);
                }
                Genus::Img {
                    style,
                    img_path,
                    scale,
                } => match cache.image.load(&img_path) {
                    Some(img) => {
                        calculate_image(&mut element.bound, img, style, &scale);
                    }
                    None => {
                        reset(element);
                    }
                },
            }
        }
    }
}

fn reset(element: &mut Element) {
    element.tag = Tag::None;
    element.bound.dim = Vec2::new(0.0, 0.0);
    element.bound.pos = Vec2::new(0.0, 0.0);
    element.bound.shadow = [0.0, 0.0, 0.0, 0.0];
}

fn calculate_image(bound: &mut Bound, img: &Image, style: &Style, scale: &f32) {
    bound.dim.x = img.width() as f32 * scale;
    bound.dim.y = img.height() as f32 * scale;
    bound.shadow = [
        style.pad.left,
        style.pad.right,
        style.pad.top,
        style.pad.low,
    ];
}

fn calculate_text(
    bound: &mut Bound,
    cache: &mut Cache,
    font: &juste::Font,
    text: &str,
    size: f32,
    style: &Style,
) {
    let font = match cache.font.load(font) {
        Some(tf) => {
            let font = Font::new(tf, Some(size));
            font
        }
        None => {
            let font = Font::default().with_size(size);
            font.unwrap()
        }
    };
    let (_sc, rec) = font.measure_str(text, Some(&build_paint(style)));
    bound.shadow = [
        style.pad.left,
        style.pad.right,
        style.pad.top,
        style.pad.low,
    ];
    //dbg!(rec);
    bound.dim.x = rec.width();
    bound.dim.y = rec.height();
}

fn calculate_box(
    bound: &mut Bound,
    children: &mut Vec<Element>,
    cache: &mut Cache,
    dim: &[Size; 2],
    ceil: &Option<[Size; 2]>,
    style: &Style,
) -> bool {
    bound.shadow = [
        style.pad.left,
        style.pad.right,
        style.pad.top,
        style.pad.low,
    ];
    let dim_x = dim[0];
    let dim_y = dim[1];

    bound.dim.x = match dim_x {
        Size::Window => cache.io.window_size.x,
        Size::Man(m) => m,
        Size::Func(f) => f(&cache.io),
        Size::Child => children
            .iter()
            .map(|c| c.bound.dim.x + c.bound.shadow[0] + c.bound.shadow[1])
            .sum(),
    };

    bound.dim.y = match dim_y {
        Size::Window => cache.io.window_size.y,
        Size::Man(m) => m,
        Size::Func(f) => f(&cache.io),
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
            Size::Func(f) => {
                let i = f(&cache.io);
                if i < bound.dim.x {
                    bound.dim.x = i;
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
            Size::Func(f) => {
                let i = f(&cache.io);
                if i < bound.dim.y {
                    bound.dim.y = i;
                    clip = true;
                }
            }
            Size::Child => (),
        }
    }
    clip
}

fn recure_pass(children: &mut Vec<Element>, cache: &mut Cache) {
    for child in children.iter_mut() {
        first_pass(child, cache);
    }
}

fn listen(element: &mut Element, cache: &mut Cache) {
    if let Some((idx, msg)) = element.listen_io(&cache.io) {
        cache.bus.insert(idx, msg);
    }
    element.listen_signal(&mut cache.bus);
}

pub fn second_pass(element: &mut Element, canvas: &Canvas, cache: &mut Cache) {
    match element.tag {
        Tag::None => return,
        _ => match &mut element.genus {
            Genus::Box {
                style,
                gravity,
                size: _,
                ceil: _,
                children,
            } => {
                let rec = Rect::from_xywh(
                    element.bound.pos.x,
                    element.bound.pos.y,
                    element.bound.dim.x,
                    element.bound.dim.y,
                );

                if element.bound.angle != 0.0 {
                    scope(canvas, |c| {
                        let pivot = Point::new(
                            element.bound.pos.x + (element.bound.dim.x / 2.0),
                            element.bound.pos.y + (element.bound.dim.y / 2.0),
                        );
                        let matrix = Matrix::rotate_deg_pivot(element.bound.angle, pivot);
                        c.concat(&matrix);
                        c.draw_rect(rec, &build_paint(style));
                    });
                } else {
                    canvas.draw_rect(rec, &build_paint(style));
                }
                scope(canvas, |c| {
                    if element.bound.overflow.need_clip() {
                        c.clip_path(
                            &build_path(&rec, &mut element.bound),
                            ClipOp::Intersect,
                            Some(true),
                        );
                    }
                    let mut offset_x = element.bound.pos.x + element.bound.offset.x;
                    let mut offset_y = element.bound.pos.y + element.bound.offset.y;

                    match gravity {
                        Gravity::Horizontal => {
                            for child in children.iter_mut() {
                                if let Tag::Prime = child.tag {
                                    continue;
                                }
                                offset_x += child.bound.shadow[0];
                                child.bound.pos.x = offset_x;
                                child.bound.pos.y = offset_y + child.bound.shadow[2];
                                offset_x += child.bound.dim.x + child.bound.shadow[1];
                                second_pass(child, canvas, cache);
                            }
                        }
                        Gravity::Vertical => {
                            for child in children.iter_mut() {
                                if let Tag::Prime = child.tag {
                                    continue;
                                }
                                offset_y += child.bound.shadow[2];
                                child.bound.pos.x = offset_x + child.bound.shadow[0];
                                child.bound.pos.y = offset_y;
                                offset_y += child.bound.dim.y + child.bound.shadow[3];
                                second_pass(child, canvas, cache);
                            }
                        }
                    };
                });
            }
            Genus::Text {
                style,
                text,
                font,
                size,
                spacing: _,
            } => {
                let font = match cache.font.load(font) {
                    Some(tf) => {
                        let font = Font::new(tf, Some(*size));
                        font
                    }
                    None => {
                        let font = Font::default().with_size(*size);
                        font.unwrap()
                    }
                };
                let (_, met) = font.metrics();
                let pos_y = element.bound.pos.y + met.ascent.abs();
                if element.bound.angle != 0.0 {
                    scope(canvas, |c| {
                        let pivot = Point::new(
                            element.bound.pos.x + (element.bound.dim.x / 2.0),
                            element.bound.pos.y + (element.bound.dim.y / 2.0),
                        );
                        let matrix = Matrix::rotate_deg_pivot(element.bound.angle, pivot);
                        c.concat(&matrix);
                        c.draw_str(
                            &mut *text,
                            (element.bound.pos.x, pos_y),
                            &font,
                            &build_paint(style),
                        );
                    });
                } else {
                    canvas.draw_str(
                        text,
                        (element.bound.pos.x, pos_y),
                        &font,
                        &build_paint(style),
                    );
                }
            }
            Genus::Img {
                style,
                img_path,
                scale: _,
            } => match cache.image.load(&img_path) {
                None => return,
                Some(img) => {
                    let rec = Rect::from_xywh(
                        element.bound.pos.x,
                        element.bound.pos.y,
                        element.bound.dim.x,
                        element.bound.dim.y,
                    );
                    if element.bound.angle != 0.0 {
                        scope(canvas, |c| {
                            let pivot = Point::new(
                                element.bound.pos.x + (element.bound.dim.x / 2.0),
                                element.bound.pos.y + (element.bound.dim.y / 2.0),
                            );
                            let matrix = Matrix::rotate_deg_pivot(element.bound.angle, pivot);
                            c.concat(&matrix);
                            c.draw_image_rect(img, None, rec, &build_paint(style));
                        });
                    } else {
                        canvas.draw_image_rect(img, None, rec, &build_paint(style));
                    }
                }
            },
        },
    }
}

fn build_path(rec: &Rect, bound: &mut Bound) -> Path {
    let cx = rec.center_x();
    let cy = rec.center_y();

    let mut path = Path::new();
    path.add_rect(rec, None);
    let mut matrix = Matrix::new_identity();
    matrix.pre_translate((cx, cy));
    matrix.pre_rotate(bound.angle, None);
    matrix.pre_translate((-cx, -cy));
    path.transform(&matrix);
    path
}

fn build_paint(style: &Style) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_argb(
        style.color[3],
        style.color[0],
        style.color[1],
        style.color[2],
    );
    paint
}

fn scope<T>(canvas: &Canvas, mut fun: T)
where
    T: FnMut(&Canvas),
{
    canvas.save();
    fun(canvas);
    canvas.restore();
}

fn snap(i: f32) -> f32 {
    i.round()
}

// fn matrix_rotation(element: &Element) -> Matrix {
//     let pivot = Point::new(
//         element.bound.pos.x + (element.bound.dim.x / 2.0),
//         element.bound.pos.y + (element.bound.dim.y / 2.0),
//     );
//     Matrix::rotate_deg_pivot(element.bound.angle, pivot)
// }
