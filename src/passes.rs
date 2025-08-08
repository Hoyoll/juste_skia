use juste::{
    element::{Bound, Element},
    genus::{Box, Genus, Image, Input, State, Text, Token},
    style::{Color, Gravity, Origin, Pad, Size},
};
use skia_safe::{Canvas, ClipOp, Matrix, Paint, Path, Rect};

use crate::renderer::Cache;

pub fn first_pass(element: &mut Element, cache: &mut Cache) {
    element.listen_io(&cache.io).map(|s| {
        let (idx, msg) = s;
        cache.bus.insert(idx, msg);
    });
    element.listen_signal(&mut cache.bus);
    match &mut element.genus {
        Genus::Input(input) => {
            calc_input(&mut element.bound, cache, input);
        }
        Genus::Cult(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    silent_first_pass(child, cache);
                });
            });
            calc_box(&mut element.bound, cache, b);
        }
        Genus::Box(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    first_pass(child, cache);
                });
            });
            calc_box(&mut element.bound, cache, b);
        }
        Genus::Text(text) => calc_text(&mut element.bound, cache, text),
        Genus::Img(img) => calc_image(&mut element.bound, cache, img),
    }
}

pub fn silent_first_pass(element: &mut Element, cache: &mut Cache) {
    match &mut element.genus {
        Genus::Input(input) => {
            calc_input(&mut element.bound, cache, input);
        }
        Genus::Cult(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    silent_first_pass(child, cache);
                });
            });
            calc_box(&mut element.bound, cache, b);
        }
        Genus::Box(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    silent_first_pass(child, cache);
                });
            });
            calc_box(&mut element.bound, cache, b);
        }
        Genus::Text(text) => calc_text(&mut element.bound, cache, text),
        Genus::Img(img) => calc_image(&mut element.bound, cache, img),
    }
}

fn calc_box(bound: &mut Bound, cache: &mut Cache, b: &mut Box) {
    let pad = match &b.style.pad {
        Origin::Id(id) => cache.sheet.pads.get(id).unwrap(),
        Origin::Raw(pad) => pad,
    };
    put_pad(bound, pad);
    let width = b.size.x;
    let height = b.size.y;
    bound.dim.x = match width {
        Size::Window => cache.io.window_size.x,
        Size::Man(man) => man,
        Size::Child => b.children.as_ref().map_or(0.0, |c| {
            let mut temp_w = 0.0;
            c.iter(|child| {
                temp_w += child.bound.dim.x + child.bound.shadow[0] + child.bound.shadow[1];
            });
            temp_w
        }),
        Size::Func(fun) => fun(&cache.io),
    };

    bound.dim.y = match height {
        Size::Window => cache.io.window_size.y,
        Size::Man(man) => man,
        Size::Func(fun) => fun(&cache.io),
        Size::Child => b.children.as_ref().map_or(0.0, |c| {
            let mut temp_y = 0.0;
            c.iter(|child| {
                temp_y += child.bound.dim.y + child.bound.shadow[2] + child.bound.shadow[3];
            });
            temp_y
        }),
    };

    match &b.ceil {
        Some(ceil) => {
            match ceil.x {
                Size::Window => {
                    if cache.io.window_size.x < bound.dim.x {
                        bound.dim.x = cache.io.window_size.x;
                        b.overflow.make_clip();
                    }
                }
                Size::Man(m) => {
                    if m < bound.dim.x {
                        bound.dim.x = m;
                        b.overflow.make_clip();
                    }
                }
                Size::Func(fun) => {
                    let f = fun(&cache.io);
                    if f < bound.dim.x {
                        bound.dim.x = f;
                        b.overflow.make_clip();
                    }
                }
                _ => (),
            }
            match ceil.y {
                Size::Window => {
                    if cache.io.window_size.y < bound.dim.y {
                        bound.dim.y = cache.io.window_size.y;
                        b.overflow.make_clip();
                    }
                }
                Size::Man(m) => {
                    if m < bound.dim.y {
                        bound.dim.y = m;
                        b.overflow.make_clip();
                    }
                }
                Size::Func(fun) => {
                    let f = fun(&cache.io);
                    if f < bound.dim.y {
                        bound.dim.y = f;
                        b.overflow.make_clip();
                    }
                }
                _ => (),
            }
        }
        None => (),
    }
}

fn calc_input(bound: &mut Bound, cache: &mut Cache, input: &Input) {
    let pad = match &input.style.style.pad {
        Origin::Id(id) => cache.sheet.pads.get(id).unwrap(),
        Origin::Raw(pad) => pad,
    };
    put_pad(bound, pad);
    bound.dim.y = input.token_size.y;
}
fn calc_image(bound: &mut Bound, cache: &mut Cache, img: &Image) {
    let pad = match &img.style.pad {
        Origin::Id(id) => cache.sheet.pads.get(id).unwrap(),
        Origin::Raw(pad) => pad,
    };
    put_pad(bound, pad);
    match cache.image.load(&img.img_path) {
        Some(image) => {
            bound.dim.x = image.width() as f32;
            bound.dim.y = image.height() as f32;
        }
        None => {
            let mut fb_el = (img.fallback)(&cache.io);
            silent_first_pass(&mut fb_el, cache);
            bound.dim.x = fb_el.bound.dim.x;
            bound.dim.y = fb_el.bound.dim.y;
        }
    }
}

fn calc_text(bound: &mut Bound, cache: &mut Cache, text: &Text) {
    let pad = match &text.style.style.pad {
        Origin::Id(id) => cache.sheet.pads.get(id).unwrap(),
        Origin::Raw(pad) => pad,
    };
    put_pad(bound, pad);
    let f = match &text.style.font {
        Origin::Id(id) => cache.sheet.fonts.get(id).unwrap(),
        Origin::Raw(font) => font,
    };
    cache.font.load_asset(f).map(|asset| {
        let (_, rect) = asset.font.measure_str(&text.text, None);
        bound.dim.x = rect.width();
        bound.dim.y = rect.height();
    });
}

fn put_pad(bound: &mut Bound, pad: &Pad) {
    bound.shadow = [pad.left, pad.right, pad.top, pad.low];
}

pub fn second_pass(element: &mut Element, canvas: &Canvas, cache: &mut Cache) {
    match &mut element.genus {
        Genus::Img(img) => pos_img(&mut element.bound, canvas, cache, img),
        Genus::Text(text) => pos_text(&mut element.bound, canvas, cache, text),
        Genus::Input(input) => pos_input(&mut element.bound, canvas, cache, input),
        Genus::Box(b) | Genus::Cult(b) => pos_box(&mut element.bound, canvas, cache, b),
    }
}

fn pos_box(bound: &mut Bound, canvas: &Canvas, cache: &mut Cache, b: &mut Box) {
    let rec = Rect::from_xywh(bound.pos.x, bound.pos.y, bound.dim.x, bound.dim.y);
    let col = match &b.style.color {
        Origin::Id(id) => cache.sheet.colors.get(id).unwrap(),
        Origin::Raw(c) => c,
    };
    if let Some(angle) = &bound.angle {
        scope(canvas, |c| {
            let pivot = skia_safe::Point::new(
                bound.pos.x + (bound.dim.x / 2.0),
                bound.pos.y + (bound.dim.y / 2.0),
            );
            let matrix = Matrix::rotate_deg_pivot(*angle, pivot);
            c.concat(&matrix);
            c.draw_rect(rec, &build_paint(col));
        });
    } else {
        canvas.draw_rect(rec, &build_paint(col));
    }

    scope(canvas, |c| {
        if b.overflow.need_clip() {
            c.clip_path(&build_path(&rec, bound), ClipOp::Intersect, Some(true));
        }
        let mut offset_x = bound.pos.x + bound.offset.x;
        let mut offset_y = bound.pos.y + bound.offset.y;

        match b.gravity {
            Gravity::Horizontal => {
                b.children.as_mut().map(|c| {
                    c.iter_mut(|child| {
                        offset_x += child.bound.shadow[0];
                        child.bound.pos.x = offset_x;
                        child.bound.pos.y = offset_y + child.bound.shadow[2];
                        offset_x += child.bound.dim.x + child.bound.shadow[1];
                        second_pass(child, canvas, cache);
                    })
                });
            }
            Gravity::Vertical => {
                b.children.as_mut().map(|c| {
                    c.iter_mut(|child| {
                        offset_y += child.bound.shadow[2];
                        child.bound.pos.x = offset_x + child.bound.shadow[0];
                        child.bound.pos.y = offset_y;
                        offset_y += child.bound.dim.y + child.bound.shadow[3];
                        second_pass(child, canvas, cache);
                    })
                });
            }
        };
    });
}
fn pos_img(bound: &mut Bound, canvas: &Canvas, cache: &mut Cache, img: &Image) {
    match cache.image.load(&img.img_path) {
        Some(image) => {
            let rec = Rect::from_xywh(bound.pos.x, bound.pos.y, bound.dim.x, bound.dim.y);
            let col = match &img.style.color {
                Origin::Id(id) => cache.sheet.colors.get(id).unwrap(),
                Origin::Raw(color) => color,
            };
            let paint = build_paint(col);
            if let Some(angle) = &bound.angle {
                scope(canvas, |c| {
                    let pivot = skia_safe::Point::new(
                        bound.pos.x + (bound.dim.x / 2.0),
                        bound.pos.y + (bound.dim.y / 2.0),
                    );
                    let matrix = Matrix::rotate_deg_pivot(*angle, pivot);
                    c.concat(&matrix);
                    c.draw_image_rect(image, None, rec, &paint);
                });
            } else {
                canvas.draw_image_rect(image, None, rec, &paint);
            }
        }
        None => {
            let mut fb_el = (img.fallback)(&cache.io);
            fb_el.bound = *bound;
            second_pass(&mut fb_el, canvas, cache);
        }
    }
}
fn pos_input(bound: &mut Bound, canvas: &Canvas, cache: &mut Cache, input: &Input) {
    let col = match &input.style.style.color {
        Origin::Id(id) => cache.sheet.colors.get(id).unwrap(),
        Origin::Raw(color) => color,
    };
    let paint = build_paint(col);
    let f = match &input.style.font {
        Origin::Raw(f) => f,
        Origin::Id(id) => cache.sheet.fonts.get(id).unwrap(),
    };
    let font = cache.font.load_asset(f).unwrap();
    let mut offset = 0.0;
    input.stream.left.iter().for_each(|token| match token {
        Token::Space => offset += input.token_size.x,
        Token::Char(c) => match font.get_char(c) {
            Some(c) => {
                canvas.draw_text_blob(c, (bound.pos.x + offset, bound.pos.y), &paint);
                offset += input.token_size.x;
            }
            None => offset += input.token_size.x,
        },
    });
    let offset_left = offset;
    input
        .stream
        .right
        .iter()
        .rev()
        .for_each(|token| match token {
            Token::Space => offset += input.token_size.x,
            Token::Char(c) => match font.get_char(c) {
                Some(c) => {
                    canvas.draw_text_blob(c, (bound.pos.x + offset, bound.pos.y), &paint);
                    offset += input.token_size.x;
                }
                None => offset += input.token_size.x,
            },
        });
    if let State::Hidden = input.state {
        return;
    }
    canvas.draw_rect(
        &Rect::from_xywh(
            bound.pos.x + offset_left,
            bound.pos.y,
            input.cursor.width,
            input.token_size.y,
        ),
        &paint,
    );
}
fn pos_text(bound: &mut Bound, canvas: &Canvas, cache: &mut Cache, text: &Text) {
    let col = match &text.style.style.color {
        Origin::Id(id) => cache.sheet.colors.get(id).unwrap(),
        Origin::Raw(color) => color,
    };
    let paint = build_paint(col);
    let f = match &text.style.font {
        Origin::Raw(f) => f,
        Origin::Id(id) => cache.sheet.fonts.get(id).unwrap(),
    };
    let font = cache.font.load_asset(f).unwrap();
    let (_, met) = font.font.metrics();
    let pos_y = bound.pos.y + met.ascent.abs();
    if let Some(angle) = &bound.angle {
        scope(canvas, |c| {
            let pivot = skia_safe::Point::new(
                bound.pos.x + (bound.dim.x / 2.0),
                bound.pos.y + (bound.dim.y / 2.0),
            );
            let matrix = Matrix::rotate_deg_pivot(*angle, pivot);
            c.concat(&matrix);
            c.draw_str(&*text.text, (bound.pos.x, pos_y), &font.font, &paint);
        });
    } else {
        canvas.draw_str(&*text.text, (bound.pos.x, pos_y), &font.font, &paint);
    }
}
fn build_path(rec: &Rect, bound: &mut Bound) -> Path {
    let cx = rec.center_x();
    let cy = rec.center_y();
    let mut path = Path::new();
    path.add_rect(rec, None);
    let mut matrix = Matrix::new_identity();
    matrix
        .pre_translate((cx, cy))
        .pre_rotate(*bound.angle.as_ref().unwrap(), None)
        .pre_translate((-cx, -cy));
    path.transform(&matrix);
    path
}

fn build_paint(color: &Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_argb(color.a, color.r, color.g, color.b);
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
