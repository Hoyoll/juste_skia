use juste::{
    element::Bound,
    genus::{Edit, Token},
    style::{ColorId, DEFAULT, FontId, Sheet},
    util::Vec2,
};
use libloading::{Library, Symbol};
use skia_safe::Canvas;

use crate::renderer::Cache;

pub struct Plug<T> {
    pub data: *mut T,
    pub dll: Library,
    pub path: String,
    pub symbol: String,
}

impl<T> Plug<T> {
    pub fn new(path: &str, symbol: &str) -> Self {
        unsafe {
            let dll = Library::new(path).expect("Dll Not Found");
            let data: Symbol<extern "C" fn() -> *mut T> =
                dll.get(symbol.as_bytes()).expect("Symbol does not exist!");
            Self {
                data: data(),
                dll,
                path: path.to_string(),
                symbol: symbol.to_string(),
            }
        }
    }

    pub fn reload(&mut self) {
        unsafe {
            drop(Box::from_raw(self.data));
            let dll =
                Library::new(&self.path).expect(&format!("Dll not found! for: {}", self.path));
            let data: Symbol<extern "C" fn() -> *mut T> = dll
                .get(self.symbol.as_bytes())
                .expect(&format!("Symbol does not exist! for: {}", self.symbol));
            self.data = data();
            self.dll = dll;
        }
    }
}

pub enum Node<T> {
    None,
    Exist(T),
}

#[repr(C)]
pub enum Cmd {
    Draw(ColorId),
    Hold,
    Skip,
    Pull(ColorId, ColorId),
    Future(ColorId, ColorId),
}

#[repr(C)]
pub struct ParseTable {
    pub group: fn(&[u8]) -> Cmd,
    pub single: fn(&u8) -> Cmd,
}

struct Parse {
    pub pos: Vec2<f32>,
    pub prev: Node<*mut [u8]>,
    pub prev_pos: Vec2<f32>,
    pub future: Node<ColorId>,
    pub table: ParseTable,
}

pub trait Parser<T> {
    fn parse(
        &mut self,
        edit: &mut Edit,
        canvas: T,
        sheet: &Sheet,
        cache: &mut Cache,
        bound: &mut Bound,
    );
}

impl Parser<&mut Canvas> for Parse {
    fn parse(
        &mut self,
        edit: &mut Edit,
        canvas: &mut Canvas,
        sheet: &Sheet,
        cache: &mut Cache,
        bound: &mut Bound,
    ) {
        self.pos = bound.pos;
        edit.buffer.iter_mut(|token| match token {
            Token::Group(group) => match &mut (self.table.group)(&group) {
                Cmd::Draw(i) => {
                    if let Node::Exist(f) = self.future {
                        *i = f;
                    }
                    draw(
                        u8_to_str(&group),
                        canvas,
                        sheet,
                        cache,
                        &self.pos,
                        &i,
                        &edit.font,
                    );
                    self.pos.x += edit.size.x * group.len() as f32;
                }
                Cmd::Hold => {
                    self.prev_pos = self.pos;
                    self.prev = Node::Exist(group.as_mut_slice());
                }
                Cmd::Skip => self.pos.x += edit.size.x * group.len() as f32,
                Cmd::Future(current, future) => {
                    if let Node::Exist(f) = self.future {
                        *current = f;
                    }
                    draw(
                        u8_to_str(&group),
                        canvas,
                        sheet,
                        cache,
                        &self.pos,
                        &current,
                        &edit.font,
                    );
                    self.future = Node::Exist(*future);
                    self.pos.x += edit.size.x * group.len() as f32;
                }
                _ => (),
            },
            Token::Single(i) => match &mut (self.table.single)(i) {
                Cmd::Draw(col) => {
                    if let Node::Exist(f) = self.future {
                        *col = f;
                    }
                    draw(
                        u8_to_str(&[*i]),
                        canvas,
                        sheet,
                        cache,
                        &self.pos,
                        &col,
                        &edit.font,
                    );
                    self.pos.x += edit.size.x;
                }
                Cmd::Pull(now, prev) => {
                    if let Node::Exist(p) = self.prev {
                        draw(
                            u8_to_str(unsafe { &*p }),
                            canvas,
                            sheet,
                            cache,
                            &self.prev_pos,
                            &prev,
                            &edit.font,
                        );
                    }
                    if let Node::Exist(f) = self.future {
                        *now = f;
                    }
                    draw(
                        u8_to_str(&[*i]),
                        canvas,
                        sheet,
                        cache,
                        &self.pos,
                        &now,
                        &edit.font,
                    );
                    self.pos.x += edit.size.x;
                }
                Cmd::Future(current, future) => {
                    if let Node::Exist(f) = self.future {
                        *current = f;
                    }
                    draw(
                        u8_to_str(&[*i]),
                        canvas,
                        sheet,
                        cache,
                        &self.pos,
                        &current,
                        &edit.font,
                    );
                    self.future = Node::Exist(*future);
                    self.pos.x += edit.size.x;
                }
                Cmd::Skip => self.pos.x += edit.size.x,
                _ => (),
            },
            Token::Space => self.pos.x += edit.size.x,
            Token::Break => {
                if let Node::Exist(node) = self.prev {
                    draw(
                        u8_to_str(unsafe { &*node }),
                        canvas,
                        sheet,
                        cache,
                        &self.prev_pos,
                        &DEFAULT,
                        &edit.font,
                    );
                }
                self.pos.reset(0.0);
                self.prev = Node::None;
                self.future = Node::None;
            }
        });
    }
}

fn draw(
    word: &str,
    canvas: &mut Canvas,
    sheet: &Sheet,
    cache: &mut Cache,
    pos: &Vec2<f32>,
    color_id: &ColorId,
    font_id: &FontId,
) {
}
fn u8_to_str(u8: &[u8]) -> &str {
    unsafe { str::from_utf8_unchecked(u8) }
}
