use juste::{
    element::{Bound, Element, Listeners},
    genus::{Ctx, CursorState, Dirt, Edit, Frame, Genus, Image, Text},
    style::{ColorId, DEFAULT, Gravity, Pad, Sheet, Size},
    util::Dir,
};
use skia_safe::{Canvas, ClipOp, Matrix, Paint, Path, Rect};

use crate::renderer::{Cache, FontAsset};

pub fn first_pass(
    element: &mut Element,
    cache: &mut Cache,
    listener: &mut Listeners,
    sheet: &mut Sheet,
) {
    match &element.listener {
        None => (),
        Some(id) => match listener.get_mut(id) {
            None => (),
            Some(list) => list.listen_io(element, &mut cache.io),
        },
    }
    cache.inside_window(element, |e, ca| match &mut e.genus {
        Genus::Input(input) => {
            calc_input(&mut e.bound, input, sheet);
        }
        Genus::Cult(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    silent_first_pass(child, ca, listener, sheet);
                });
            });
            calc_box(&mut e.bound, ca, b, sheet);
        }
        Genus::Frame(b) | Genus::Float(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    first_pass(child, ca, listener, sheet);
                });
            });
            calc_box(&mut e.bound, ca, b, sheet);
        }
        Genus::Text(text) => calc_text(&mut e.bound, ca, text, sheet),
        Genus::Img(img) => calc_image(&mut e.bound, ca, img, listener, sheet),
        _ => (),
    });
}

pub fn silent_first_pass(
    element: &mut Element,
    cache: &mut Cache,
    listener: &mut Listeners,
    sheet: &mut Sheet,
) {
    cache.inside_window(element, |e, ca| match &mut e.genus {
        Genus::Input(input) => {
            calc_input(&mut e.bound, input, sheet);
        }
        Genus::Cult(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    silent_first_pass(child, ca, listener, sheet);
                });
            });
            calc_box(&mut e.bound, ca, b, sheet);
        }
        Genus::Frame(b) | Genus::Float(b) => {
            b.children.as_mut().map(|c| {
                c.iter_mut(|child| {
                    silent_first_pass(child, ca, listener, sheet);
                });
            });
            calc_box(&mut e.bound, ca, b, sheet);
        }
        Genus::Text(text) => calc_text(&mut e.bound, ca, text, sheet),
        Genus::Img(img) => calc_image(&mut e.bound, ca, img, listener, sheet),
        _ => (),
    });
}

fn calc_box(bound: &mut Bound, cache: &mut Cache, b: &mut Frame, sheet: &mut Sheet) {
    let pad = match sheet.pads.get(&b.style.pad) {
        Some(p) => p,
        None => sheet.pads.get(&DEFAULT).unwrap(),
    };
    put_pad(bound, pad);
    let width = b.size.x;
    let height = b.size.y;
    bound.dim.x = match width {
        Size::Window => cache.io.window_size.x,
        Size::Man(man) => man,
        Size::Child => b.children.as_mut().map_or(0.0, |c| {
            let mut temp_w = 0.0;
            c.iter_mut(|child| {
                child.position(|bound| {
                    temp_w += bound.dim.x + bound.shadow[0] + bound.shadow[1];
                });
            });
            temp_w
        }),
        Size::Func(fun) => fun(&cache.io),
    };

    bound.dim.y = match height {
        Size::Window => cache.io.window_size.y,
        Size::Man(man) => man,
        Size::Func(fun) => fun(&cache.io),
        Size::Child => b.children.as_mut().map_or(0.0, |c| {
            let mut temp_y = 0.0;
            c.iter_mut(|child| {
                child.position(|bound| {
                    temp_y += bound.dim.y + bound.shadow[2] + bound.shadow[3];
                });
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

fn calc_input(bound: &mut Bound, input: &Edit, sheet: &mut Sheet) {
    let pad = match sheet.pads.get(&input.pad) {
        Some(p) => p,
        None => sheet.pads.get(&DEFAULT).unwrap(),
    };
    put_pad(bound, pad);
    bound.dim = input.frame_size;
}

fn calc_image(
    bound: &mut Bound,
    cache: &mut Cache,
    img: &Image,
    listener: &mut Listeners,
    sheet: &mut Sheet,
) {
    let pad = match sheet.pads.get(&img.style.pad) {
        Some(p) => p,
        None => sheet.pads.get(&DEFAULT).unwrap(),
    };
    put_pad(bound, pad);
    match cache.image.load(&img.img_path) {
        Some(image) => {
            bound.dim.x = image.width() as f32;
            bound.dim.y = image.height() as f32;
        }
        None => {
            let mut fb_el = (img.fallback)(&cache.io);
            silent_first_pass(&mut fb_el, cache, listener, sheet);
            bound.dim.x = fb_el.bound.dim.x;
            bound.dim.y = fb_el.bound.dim.y;
        }
    }
}

fn calc_text(bound: &mut Bound, cache: &mut Cache, text: &Text, sheet: &mut Sheet) {
    let pad = match sheet.pads.get(&text.style.style.pad) {
        Some(p) => p,
        None => sheet.pads.get(&DEFAULT).unwrap(),
    };
    put_pad(bound, pad);
    let f = match sheet.fonts.get(&text.style.font) {
        Some(f) => f,
        None => sheet.fonts.get(&DEFAULT).unwrap(),
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

pub fn second_pass(element: &mut Element, canvas: &Canvas, cache: &mut Cache, sheet: &mut Sheet) {
    cache.inside_window(element, |e, c| match &mut e.genus {
        Genus::Img(img) => pos_img(&mut e.bound, canvas, c, img, sheet),
        Genus::Text(text) => pos_text(&mut e.bound, canvas, c, text, sheet),
        Genus::Input(input) => pos_input(&mut e.bound, canvas, c, input, sheet),
        Genus::Frame(b) | Genus::Cult(b) | Genus::Float(b) => {
            pos_box(&mut e.bound, canvas, c, b, sheet)
        }
        _ => (),
    });
}

fn pos_box(
    bound: &mut Bound,
    canvas: &Canvas,
    cache: &mut Cache,
    b: &mut Frame,
    sheet: &mut Sheet,
) {
    let rec = Rect::from_xywh(bound.pos.x, bound.pos.y, bound.dim.x, bound.dim.y);
    let col = match sheet.colors.get(&b.style.color) {
        Some(c) => c,
        None => sheet.colors.get(&DEFAULT).unwrap(),
    };
    match &bound.angle {
        Some(angle) => {
            scope(canvas, |c| {
                let pivot = skia_safe::Point::new(
                    bound.pos.x + (bound.dim.x / 2.0),
                    bound.pos.y + (bound.dim.y / 2.0),
                );
                let matrix = Matrix::rotate_deg_pivot(*angle, pivot);
                c.concat(&matrix);
                c.draw_rect(
                    rec,
                    &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
                );
            });
        }
        None => {
            canvas.draw_rect(
                rec,
                &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
            );
        }
    }
    scope(canvas, |c| {
        if b.overflow.need_clip() {
            c.clip_path(&build_path(&rec, bound), ClipOp::Intersect, Some(true));
        }
        let mut offset_x = bound.pos.x + b.child_offset.x;
        let mut offset_y = bound.pos.y + b.child_offset.y;

        match b.gravity {
            Gravity::Horizontal => {
                b.children.as_mut().map(|c| {
                    c.iter_mut(|child| {
                        child.position(|bound| {
                            offset_x += bound.shadow[0];
                            bound.pos.x = offset_x;
                            bound.pos.y = offset_y + bound.shadow[2];
                            offset_x += bound.dim.x + bound.shadow[1];
                        });
                        second_pass(child, canvas, cache, sheet);
                    })
                });
            }
            Gravity::Vertical => {
                b.children.as_mut().map(|c| {
                    c.iter_mut(|child| {
                        child.position(|bound| {
                            offset_y += bound.shadow[2];
                            bound.pos.x = offset_x + bound.shadow[0];
                            bound.pos.y = offset_y;
                            offset_y += bound.dim.y + bound.shadow[3];
                        });
                        second_pass(child, canvas, cache, sheet);
                    })
                });
            }
        };
    });
}
fn pos_img(bound: &mut Bound, canvas: &Canvas, cache: &mut Cache, img: &Image, sheet: &mut Sheet) {
    match cache.image.load(&img.img_path) {
        Some(image) => {
            let rec = Rect::from_xywh(bound.pos.x, bound.pos.y, bound.dim.x, bound.dim.y);
            let col = match sheet.colors.get(&img.style.color) {
                Some(c) => c,
                None => sheet.colors.get(&DEFAULT).unwrap(),
            };
            match &bound.angle {
                Some(angle) => {
                    scope(canvas, |c| {
                        let pivot = skia_safe::Point::new(
                            bound.pos.x + (bound.dim.x / 2.0),
                            bound.pos.y + (bound.dim.y / 2.0),
                        );
                        let matrix = Matrix::rotate_deg_pivot(*angle, pivot);
                        c.concat(&matrix);
                        c.draw_image_rect(
                            image,
                            None,
                            rec,
                            &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
                        );
                    });
                }
                None => {
                    canvas.draw_image_rect(
                        image,
                        None,
                        rec,
                        &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
                    );
                }
            }
        }
        None => {
            let mut fb_el = (img.fallback)(&cache.io);
            fb_el.bound = *bound;
            second_pass(&mut fb_el, canvas, cache, sheet);
        }
    }
}
fn pos_input(
    bound: &mut Bound,
    canvas: &Canvas,
    cache: &mut Cache,
    edit: &mut Edit,
    sheet: &mut Sheet,
) {
    highlight(edit);

    let f = match sheet.fonts.get(&edit.font) {
        Some(f) => f,
        None => sheet.fonts.get(&DEFAULT).unwrap(),
    };

    let font = cache.font.load_asset(f).unwrap();

    let mut line_pos = bound.pos + edit.offset;
    let mut prev_line_idx = Dir::<usize>::Left(0);
    let mut cursor_pos = line_pos.x;
    let mut prev_line = line_pos.y;
    let mut prev_cursor = cursor_pos;
    let mut prev = &Ctx::Gap;
    scope(canvas, |c| {
        let rec = Rect::from_xywh(bound.pos.x, bound.pos.y, bound.dim.x, bound.dim.y);
        c.clip_path(&build_path(&rec, bound), ClipOp::Intersect, Some(true));
        for (i, line) in edit.buffer.left.iter().enumerate() {
            match bound.inside_y(&line_pos) {
                false => match line.ctx_buffer.last() {
                    None => prev = &Ctx::Gap,
                    Some(ctx) => prev = ctx,
                },
                true => {
                    match line.cursor_state {
                        CursorState::Display { char_idx } => {
                            let col = match sheet.colors.get(&edit.cursor.col) {
                                Some(c) => c,
                                None => sheet.colors.get(&DEFAULT).unwrap(),
                            };
                            let cursor_pos_x = line_pos.x + (char_idx as f32 * edit.char_size.x);
                            c.draw_rect(
                                Rect::from_xywh(
                                    cursor_pos_x,
                                    line_pos.y,
                                    edit.cursor.width,
                                    edit.char_size.y,
                                ),
                                &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
                            );
                        }
                        CursorState::Span { start_idx, length } => {
                            let cursor_pos_x = line_pos.x + (start_idx as f32 * edit.char_size.x);
                            let col = match sheet.colors.get(&edit.cursor.col) {
                                Some(c) => c,
                                None => sheet.colors.get(&DEFAULT).unwrap(),
                            };
                            c.draw_rect(
                                Rect::from_xywh(
                                    cursor_pos_x,
                                    line_pos.y,
                                    edit.cursor.width * length as f32,
                                    edit.char_size.y,
                                ),
                                &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
                            );
                        }
                        CursorState::Hidden => (),
                    }
                    line.ctx_buffer.iter().for_each(|ctx| match ctx {
                        Ctx::Put { idx, col } => {
                            match prev {
                                Ctx::Hold { idx } => {
                                    let buff = match prev_line_idx {
                                        Dir::Left(l) => &edit.buffer.left[l],
                                        Dir::Right(l) => &edit.buffer.right[l],
                                    };
                                    let buffer = &buff.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        prev_cursor,
                                        prev_line,
                                        &DEFAULT,
                                        &mut cache.reusable_paint,
                                    );
                                    prev = &Ctx::Gap;
                                }
                                _ => prev = &Ctx::Gap,
                            }
                            let buffer = &line.buffer[idx[0]..idx[1]];
                            draw(
                                c,
                                &font,
                                sheet,
                                buffer,
                                cursor_pos,
                                line_pos.y,
                                col,
                                &mut cache.reusable_paint,
                            );
                            cursor_pos += buffer.len() as f32;
                        }
                        Ctx::Hold { idx } => match prev {
                            Ctx::Hold { idx } => {
                                let buff = match prev_line_idx {
                                    Dir::Left(l) => &edit.buffer.left[l],
                                    Dir::Right(l) => &edit.buffer.right[l],
                                };
                                let buffer = &buff.buffer[idx[0]..idx[1]];
                                draw(
                                    canvas,
                                    &font,
                                    sheet,
                                    buffer,
                                    prev_cursor,
                                    prev_line,
                                    &DEFAULT,
                                    &mut cache.reusable_paint,
                                );
                                prev = ctx;
                                prev_cursor = cursor_pos;
                                prev_line = line_pos.y;
                                prev_line_idx = Dir::Left(i);
                                cursor_pos = edit.char_size.x * (idx[1] - idx[0]) as f32;
                            }
                            Ctx::Future {
                                idx,
                                col_self: _,
                                col_next,
                            } => {
                                let buffer = &line.buffer[idx[0]..idx[1]];
                                draw(
                                    c,
                                    &font,
                                    sheet,
                                    buffer,
                                    prev_cursor,
                                    prev_line,
                                    col_next,
                                    &mut cache.reusable_paint,
                                );
                                prev = &Ctx::Gap;
                            }
                            _ => {
                                prev = ctx;
                                prev_line = line_pos.y;
                                prev_cursor = cursor_pos;
                                prev_line_idx = Dir::Left(i);
                                cursor_pos = edit.char_size.x * (idx[1] - idx[0]) as f32;
                            }
                        },
                        Ctx::Pull {
                            idx,
                            col_self,
                            col_prev,
                        } => {
                            match prev {
                                Ctx::Hold { idx } => {
                                    let buff = match prev_line_idx {
                                        Dir::Left(l) => &edit.buffer.left[l],
                                        Dir::Right(l) => &edit.buffer.right[l],
                                    };
                                    let buffer = &buff.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        prev_cursor,
                                        prev_line,
                                        col_prev,
                                        &mut cache.reusable_paint,
                                    );
                                }
                                _ => (),
                            }
                            let buffer = &line.buffer[idx[0]..idx[1]];
                            draw(
                                c,
                                &font,
                                sheet,
                                buffer,
                                cursor_pos,
                                line_pos.y,
                                col_self,
                                &mut cache.reusable_paint,
                            );
                            prev = &Ctx::Gap;
                            cursor_pos += buffer.len() as f32;
                        }
                        Ctx::Future {
                            idx,
                            col_self,
                            col_next: _,
                        } => {
                            match prev {
                                Ctx::Hold { idx } => {
                                    let buff = match prev_line_idx {
                                        Dir::Left(l) => &edit.buffer.left[l],
                                        Dir::Right(l) => &edit.buffer.right[l],
                                    };
                                    let buffer = &buff.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        prev_cursor,
                                        prev_line,
                                        &DEFAULT,
                                        &mut cache.reusable_paint,
                                    );
                                }
                                Ctx::Future {
                                    idx,
                                    col_self: _,
                                    col_next,
                                } => {
                                    let buffer = &line.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        cursor_pos,
                                        line_pos.y,
                                        col_next,
                                        &mut cache.reusable_paint,
                                    );
                                    cursor_pos += buffer.len() as f32;
                                }
                                _ => {
                                    let buffer = &line.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        cursor_pos,
                                        line_pos.y,
                                        col_self,
                                        &mut cache.reusable_paint,
                                    );
                                    cursor_pos += buffer.len() as f32;
                                }
                            }
                            prev = ctx;
                        }
                        Ctx::Gap => cursor_pos += edit.char_size.x,
                    });
                }
            }
            line_pos.y += edit.char_size.y;
            cursor_pos = line_pos.x;
        }

        for (i, line) in edit.buffer.right.iter().rev().enumerate() {
            match bound.inside_y(&line_pos) {
                false => break,
                true => {
                    match line.cursor_state {
                        CursorState::Display { char_idx } => {
                            let col = match sheet.colors.get(&edit.cursor.col) {
                                Some(c) => c,
                                None => sheet.colors.get(&DEFAULT).unwrap(),
                            };
                            let cursor_pos_x = line_pos.x + (char_idx as f32 * edit.char_size.x);
                            c.draw_rect(
                                Rect::from_xywh(
                                    cursor_pos_x,
                                    line_pos.y,
                                    edit.cursor.width,
                                    edit.char_size.y,
                                ),
                                &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
                            );
                        }
                        CursorState::Span { start_idx, length } => {
                            let cursor_pos_x = line_pos.x + (start_idx as f32 * edit.char_size.x);
                            let col = match sheet.colors.get(&edit.cursor.col) {
                                Some(c) => c,
                                None => sheet.colors.get(&DEFAULT).unwrap(),
                            };
                            c.draw_rect(
                                Rect::from_xywh(
                                    cursor_pos_x,
                                    line_pos.y,
                                    edit.cursor.width * length as f32,
                                    edit.char_size.y,
                                ),
                                &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
                            );
                        }
                        CursorState::Hidden => (),
                    }
                    line.ctx_buffer.iter().for_each(|ctx| match ctx {
                        Ctx::Put { idx, col } => {
                            match prev {
                                Ctx::Hold { idx } => {
                                    let buff = match prev_line_idx {
                                        Dir::Left(l) => &edit.buffer.left[l],
                                        Dir::Right(l) => &edit.buffer.right[l],
                                    };
                                    let buffer = &buff.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        prev_cursor,
                                        prev_line,
                                        &DEFAULT,
                                        &mut cache.reusable_paint,
                                    );
                                    prev = &Ctx::Gap;
                                }
                                _ => prev = &Ctx::Gap,
                            }
                            let buffer = &line.buffer[idx[0]..idx[1]];
                            draw(
                                c,
                                &font,
                                sheet,
                                buffer,
                                cursor_pos,
                                line_pos.y,
                                col,
                                &mut cache.reusable_paint,
                            );
                            cursor_pos += buffer.len() as f32;
                        }
                        Ctx::Hold { idx } => match prev {
                            Ctx::Hold { idx } => {
                                let buff = match prev_line_idx {
                                    Dir::Left(l) => &edit.buffer.left[l],
                                    Dir::Right(l) => &edit.buffer.right[l],
                                };
                                let buffer = &buff.buffer[idx[0]..idx[1]];
                                draw(
                                    canvas,
                                    &font,
                                    sheet,
                                    buffer,
                                    prev_cursor,
                                    prev_line,
                                    &DEFAULT,
                                    &mut cache.reusable_paint,
                                );
                                prev = ctx;
                                prev_cursor = cursor_pos;
                                prev_line = line_pos.y;
                                prev_line_idx = Dir::Left(i);
                                cursor_pos = edit.char_size.x * (idx[1] - idx[0]) as f32;
                            }
                            Ctx::Future {
                                idx,
                                col_self: _,
                                col_next,
                            } => {
                                let buffer = &line.buffer[idx[0]..idx[1]];
                                draw(
                                    c,
                                    &font,
                                    sheet,
                                    buffer,
                                    prev_cursor,
                                    prev_line,
                                    col_next,
                                    &mut cache.reusable_paint,
                                );
                                prev = &Ctx::Gap;
                            }
                            _ => {
                                prev = ctx;
                                prev_line = line_pos.y;
                                prev_cursor = cursor_pos;
                                prev_line_idx = Dir::Left(i);
                                cursor_pos = edit.char_size.x * (idx[1] - idx[0]) as f32;
                            }
                        },
                        Ctx::Pull {
                            idx,
                            col_self,
                            col_prev,
                        } => {
                            match prev {
                                Ctx::Hold { idx } => {
                                    let buff = match prev_line_idx {
                                        Dir::Left(l) => &edit.buffer.left[l],
                                        Dir::Right(l) => &edit.buffer.right[l],
                                    };
                                    let buffer = &buff.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        prev_cursor,
                                        prev_line,
                                        col_prev,
                                        &mut cache.reusable_paint,
                                    );
                                }
                                _ => (),
                            }
                            let buffer = &line.buffer[idx[0]..idx[1]];
                            draw(
                                c,
                                &font,
                                sheet,
                                buffer,
                                cursor_pos,
                                line_pos.y,
                                col_self,
                                &mut cache.reusable_paint,
                            );
                            prev = &Ctx::Gap;
                            cursor_pos += buffer.len() as f32;
                        }
                        Ctx::Future {
                            idx,
                            col_self,
                            col_next: _,
                        } => {
                            match prev {
                                Ctx::Hold { idx } => {
                                    let buff = match prev_line_idx {
                                        Dir::Left(l) => &edit.buffer.left[l],
                                        Dir::Right(l) => &edit.buffer.right[l],
                                    };
                                    let buffer = &buff.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        prev_cursor,
                                        prev_line,
                                        &DEFAULT,
                                        &mut cache.reusable_paint,
                                    );
                                }
                                Ctx::Future {
                                    idx,
                                    col_self: _,
                                    col_next,
                                } => {
                                    let buffer = &line.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        cursor_pos,
                                        line_pos.y,
                                        col_next,
                                        &mut cache.reusable_paint,
                                    );
                                    cursor_pos += buffer.len() as f32;
                                }
                                _ => {
                                    let buffer = &line.buffer[idx[0]..idx[1]];
                                    draw(
                                        c,
                                        &font,
                                        sheet,
                                        buffer,
                                        cursor_pos,
                                        line_pos.y,
                                        col_self,
                                        &mut cache.reusable_paint,
                                    );
                                    cursor_pos += buffer.len() as f32;
                                }
                            }
                            prev = ctx;
                        }
                        Ctx::Gap => cursor_pos += edit.char_size.x,
                    });
                }
            }
            line_pos.y += edit.char_size.y;
            cursor_pos = line_pos.x;
        }
    });
}

fn draw(
    canvas: &Canvas,
    font: &FontAsset,
    sheet: &mut Sheet,
    buffer: &[u8],
    x: f32,
    y: f32,
    col: &ColorId,
    reusable_paint: &mut Paint,
) {
    let col = match sheet.colors.get(col) {
        Some(c) => c,
        None => sheet.colors.get(&DEFAULT).unwrap(),
    };
    canvas.draw_str(
        u8_to_str(buffer),
        (x, y),
        &font.font,
        reusable_paint.set_argb(col.a, col.r, col.g, col.b),
    );
}

fn u8_to_str(u8: &[u8]) -> &str {
    unsafe { str::from_utf8_unchecked(u8) }
}

fn highlight(edit: &mut Edit) {
    match edit.highlight.dirt {
        Dirt::On(i) => {
            let buffer = &mut edit.buffer.left[i];
            buffer.ctx_buffer.clear();
            edit.highlight.highlight(buffer);
        }
        Dirt::Range(slice) => {
            let m = &mut edit.buffer.left[slice.start..slice.end];
            for line in m.iter_mut() {
                line.ctx_buffer.clear();
                edit.highlight.highlight(line);
            }
        }
        Dirt::All => {
            edit.buffer.iter_mut(|line| {
                edit.highlight.highlight(line);
            });
        }
        Dirt::None => (),
    }
}

fn pos_text(bound: &mut Bound, canvas: &Canvas, cache: &mut Cache, text: &Text, sheet: &mut Sheet) {
    let col = match sheet.colors.get(&text.style.style.color) {
        Some(c) => c,
        None => sheet.colors.get(&DEFAULT).unwrap(),
    };

    let f = match sheet.fonts.get(&text.style.font) {
        Some(f) => f,
        None => sheet.fonts.get(&DEFAULT).unwrap(),
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
            c.draw_str(
                &*text.text,
                (bound.pos.x, pos_y),
                &font.font,
                &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
            );
        });
    } else {
        canvas.draw_str(
            &*text.text,
            (bound.pos.x, pos_y),
            &font.font,
            &cache.reusable_paint.set_argb(col.a, col.r, col.g, col.b),
        );
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
        .pre_rotate(*bound.angle.as_ref().unwrap_or(&0.0), None)
        .pre_translate((-cx, -cy));
    path.transform(&matrix);
    path
}

fn scope<T>(canvas: &Canvas, mut fun: T)
where
    T: FnMut(&Canvas),
{
    canvas.save();
    fun(canvas);
    canvas.restore();
}
