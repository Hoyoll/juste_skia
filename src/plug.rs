use juste::{genus::Line, style::ColorId};
use libloading::{Library, Symbol};

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
// struct Parse {
//     pub pos: Vec2<f32>,
//     pub prev_pos: Vec2<f32>,
//     pub table: ParseTable,
//     pub ctx: Ctx,
//     scratch: [u8; 1],
//     limit: Vec2<f32>,
// }

// pub trait Parser<T> {
//     fn parse(
//         &mut self,
//         edit: &mut Edit,
//         canvas: T,
//         sheet: &Sheet,
//         cache: &mut Cache,
//         bound: &mut Bound,
//     );

//     fn parse_no_draw(&mut self, edit: &mut Edit);
// }

// impl Parse {
//     fn parse() {}

//     fn parse_and_draw() {}
//     fn start(
//         &mut self,
//         edit: &mut Edit,
//         canvas: &mut Canvas,
//         sheet: &Sheet,
//         cache: &mut Cache,
//         bound: &mut Bound,
//     ) {
//         let temp_pos = bound.pos + edit.offset;
//         self.pos = temp_pos;
//         self.limit = bound.pos + bound.dim;
//         for line in edit.buffer.left.iter_mut() {
//             if self.pos.y > self.limit.y {
//                 // No reason to parse something below the screen
//                 break;
//             }

//             if self.pos.y < bound.pos.y {
//                 self.parse_no_draw(edit);
//             } else {
//                 self.parse_draw();
//             }
//             line.buffer.iter_mut(|token| {
//                 // TO-DO
//             });
//             self.pos.y += edit.token_size.y;
//             self.pos.x = temp_pos.x;
//         }

//         for line in edit.buffer.right.iter_mut() {
//             if self.pos.y > self.limit.y {
//                 // No reason to parse something below the screen
//                 break;
//             }
//             line.buffer.iter_mut(|token| match token {
//                 Token::Group { group, .. } => match (self.table.group)(&group) {
//                     Cmd::Put(mut i) => {
//                         match self.ctx {
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(
//                                     st,
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.prev_pos,
//                                     &DEFAULT,
//                                     &edit.font,
//                                 );
//                                 self.ctx = Ctx::None;
//                             }
//                             Ctx::Future(_) => self.ctx = Ctx::None,
//                             Ctx::Wrap(_, wc) => i = wc,
//                             _ => (),
//                         }
//                         draw(
//                             u8_to_str(&group),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &i,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.token_size.x * group.len() as f32;
//                     }
//                     Cmd::Wrap(mut me, content) => {
//                         match self.ctx {
//                             Ctx::Wrap(o_me, o_content) => {
//                                 if me == o_me && content == o_content {
//                                     self.ctx = Ctx::None;
//                                 } else {
//                                     me = o_content;
//                                 }
//                             }
//                             _ => {
//                                 self.ctx = Ctx::Wrap(me, content);
//                             }
//                         }
//                         draw(
//                             u8_to_str(&group),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &me,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.size.x * group.len() as f32;
//                     }
//                     Cmd::Hold => {
//                         match self.ctx {
//                             Ctx::Future(col) | Ctx::Wrap(_, col) => {
//                                 draw(
//                                     u8_to_str(&group),
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.pos,
//                                     &col,
//                                     &edit.font,
//                                 );
//                                 self.ctx = Ctx::None;
//                                 return;
//                             }
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(
//                                     st,
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.prev_pos,
//                                     &DEFAULT,
//                                     &edit.font,
//                                 );
//                             }
//                             _ => (),
//                         }
//                         self.pos.x += edit.size.x * group.len() as f32;
//                         self.prev_pos = self.pos;
//                         self.ctx = Ctx::Past(group.as_mut_slice());
//                     }
//                     Cmd::Skip => self.pos.x += edit.size.x * group.len() as f32,
//                     Cmd::Future(mut current, future) => {
//                         match self.ctx {
//                             Ctx::Wrap(_, col) => {
//                                 current = col;
//                             }
//                             Ctx::Future(col) => {
//                                 current = col;
//                                 self.ctx = Ctx::Future(future);
//                             }
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(
//                                     st,
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.prev_pos,
//                                     &DEFAULT,
//                                     &edit.font,
//                                 );
//                             }
//                             Ctx::None => self.ctx = Ctx::Future(future),
//                         }
//                         draw(
//                             u8_to_str(&group),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &current,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.size.x * group.len() as f32;
//                     }
//                     Cmd::Pull(mut current, prev) => {
//                         match self.ctx {
//                             Ctx::Wrap(_, col) => {
//                                 current = col;
//                             }
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(st, canvas, sheet, cache, &self.prev_pos, &prev, &edit.font);
//                                 self.ctx = Ctx::None;
//                             }
//                             _ => (),
//                         }
//                         draw(
//                             u8_to_str(&group),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &current,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.size.x * group.len() as f32;
//                     }
//                 },
//                 Token::Single(c) => match (self.table.single)(c) {
//                     Cmd::Put(mut i) => {
//                         match self.ctx {
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(
//                                     st,
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.prev_pos,
//                                     &DEFAULT,
//                                     &edit.font,
//                                 );
//                                 self.ctx = Ctx::None;
//                             }
//                             Ctx::Future(_) => self.ctx = Ctx::None,
//                             Ctx::Wrap(_, wc) => i = wc,
//                             _ => (),
//                         }
//                         draw(
//                             u8_to_str(&[*c]),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &i,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.size.x;
//                     }
//                     Cmd::Wrap(mut me, content) => {
//                         match self.ctx {
//                             Ctx::Wrap(o_me, o_content) => {
//                                 if me == o_me && content == o_content {
//                                     self.ctx = Ctx::None;
//                                 } else {
//                                     me = o_content;
//                                 }
//                             }
//                             _ => {
//                                 self.ctx = Ctx::Wrap(me, content);
//                             }
//                         }
//                         draw(
//                             u8_to_str(&[*c]),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &me,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.size.x;
//                     }
//                     Cmd::Hold => {
//                         match self.ctx {
//                             Ctx::Future(col) | Ctx::Wrap(_, col) => {
//                                 draw(
//                                     u8_to_str(&[*c]),
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.pos,
//                                     &col,
//                                     &edit.font,
//                                 );
//                                 self.ctx = Ctx::None;
//                                 return;
//                             }
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(
//                                     st,
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.prev_pos,
//                                     &DEFAULT,
//                                     &edit.font,
//                                 );
//                             }
//                             _ => (),
//                         }
//                         self.pos.x += edit.size.x;
//                         self.prev_pos = self.pos;
//                         self.scratch[0] = *c;
//                         self.ctx = Ctx::Past(self.scratch.as_mut_slice());
//                     }
//                     Cmd::Skip => self.pos.x += edit.size.x,
//                     Cmd::Future(mut current, future) => {
//                         match self.ctx {
//                             Ctx::Wrap(_, col) => {
//                                 current = col;
//                             }
//                             Ctx::Future(col) => {
//                                 current = col;
//                                 self.ctx = Ctx::Future(future);
//                             }
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(
//                                     st,
//                                     canvas,
//                                     sheet,
//                                     cache,
//                                     &self.prev_pos,
//                                     &DEFAULT,
//                                     &edit.font,
//                                 );
//                             }
//                             Ctx::None => self.ctx = Ctx::Future(future),
//                         }
//                         draw(
//                             u8_to_str(&[*c]),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &current,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.size.x;
//                     }
//                     Cmd::Pull(mut current, prev) => {
//                         match self.ctx {
//                             Ctx::Wrap(_, col) => {
//                                 current = col;
//                             }
//                             Ctx::Past(p) => {
//                                 let st = u8_to_str(unsafe { &*p });
//                                 draw(st, canvas, sheet, cache, &self.prev_pos, &prev, &edit.font);
//                                 self.ctx = Ctx::None;
//                             }
//                             _ => (),
//                         }
//                         draw(
//                             u8_to_str(&[*c]),
//                             canvas,
//                             sheet,
//                             cache,
//                             &self.pos,
//                             &current,
//                             &edit.font,
//                         );
//                         self.pos.x += edit.size.x;
//                     }
//                 },
//                 Token::Space => self.pos.x += edit.size.x,
//                 Token::Break => {
//                     self.pos.reset(0.0);
//                 }
//             });
//             self.pos.y += edit.token_size.y;
//             self.pos.x = temp_pos.x;
//         }
//         edit.buffer.iter_mut(|token| match token {
//             Token::Group { buffer: group, .. } => match (self.table.group)(&group) {
//                 Cmd::Put(mut i) => {
//                     match self.ctx {
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(
//                                 st,
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.prev_pos,
//                                 &DEFAULT,
//                                 &edit.font,
//                             );
//                             self.ctx = Ctx::None;
//                         }
//                         Ctx::Future(_) => self.ctx = Ctx::None,
//                         Ctx::Wrap(_, wc) => i = wc,
//                         _ => (),
//                     }
//                     draw(
//                         u8_to_str(&group),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &i,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x * group.len() as f32;
//                 }
//                 Cmd::Wrap(mut me, content) => {
//                     match self.ctx {
//                         Ctx::Wrap(o_me, o_content) => {
//                             if me == o_me && content == o_content {
//                                 self.ctx = Ctx::None;
//                             } else {
//                                 me = o_content;
//                             }
//                         }
//                         _ => {
//                             self.ctx = Ctx::Wrap(me, content);
//                         }
//                     }
//                     draw(
//                         u8_to_str(&group),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &me,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x * group.len() as f32;
//                 }
//                 Cmd::Hold => {
//                     match self.ctx {
//                         Ctx::Future(col) | Ctx::Wrap(_, col) => {
//                             draw(
//                                 u8_to_str(&group),
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.pos,
//                                 &col,
//                                 &edit.font,
//                             );
//                             self.ctx = Ctx::None;
//                             return;
//                         }
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(
//                                 st,
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.prev_pos,
//                                 &DEFAULT,
//                                 &edit.font,
//                             );
//                         }
//                         _ => (),
//                     }
//                     self.pos.x += edit.size.x * group.len() as f32;
//                     self.prev_pos = self.pos;
//                     self.ctx = Ctx::Past(group.as_mut_slice());
//                 }
//                 Cmd::Skip => self.pos.x += edit.size.x * group.len() as f32,
//                 Cmd::Future(mut current, future) => {
//                     match self.ctx {
//                         Ctx::Wrap(_, col) => {
//                             current = col;
//                         }
//                         Ctx::Future(col) => {
//                             current = col;
//                             self.ctx = Ctx::Future(future);
//                         }
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(
//                                 st,
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.prev_pos,
//                                 &DEFAULT,
//                                 &edit.font,
//                             );
//                         }
//                         Ctx::None => self.ctx = Ctx::Future(future),
//                     }
//                     draw(
//                         u8_to_str(&group),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &current,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x * group.len() as f32;
//                 }
//                 Cmd::Pull(mut current, prev) => {
//                     match self.ctx {
//                         Ctx::Wrap(_, col) => {
//                             current = col;
//                         }
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(st, canvas, sheet, cache, &self.prev_pos, &prev, &edit.font);
//                             self.ctx = Ctx::None;
//                         }
//                         _ => (),
//                     }
//                     draw(
//                         u8_to_str(&group),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &current,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x * group.len() as f32;
//                 }
//             },
//             Token::Single(c) => match (self.table.single)(c) {
//                 Cmd::Put(mut i) => {
//                     match self.ctx {
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(
//                                 st,
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.prev_pos,
//                                 &DEFAULT,
//                                 &edit.font,
//                             );
//                             self.ctx = Ctx::None;
//                         }
//                         Ctx::Future(_) => self.ctx = Ctx::None,
//                         Ctx::Wrap(_, wc) => i = wc,
//                         _ => (),
//                     }
//                     draw(
//                         u8_to_str(&[*c]),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &i,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x;
//                 }
//                 Cmd::Wrap(mut me, content) => {
//                     match self.ctx {
//                         Ctx::Wrap(o_me, o_content) => {
//                             if me == o_me && content == o_content {
//                                 self.ctx = Ctx::None;
//                             } else {
//                                 me = o_content;
//                             }
//                         }
//                         _ => {
//                             self.ctx = Ctx::Wrap(me, content);
//                         }
//                     }
//                     draw(
//                         u8_to_str(&[*c]),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &me,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x;
//                 }
//                 Cmd::Hold => {
//                     match self.ctx {
//                         Ctx::Future(col) | Ctx::Wrap(_, col) => {
//                             draw(
//                                 u8_to_str(&[*c]),
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.pos,
//                                 &col,
//                                 &edit.font,
//                             );
//                             self.ctx = Ctx::None;
//                             return;
//                         }
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(
//                                 st,
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.prev_pos,
//                                 &DEFAULT,
//                                 &edit.font,
//                             );
//                         }
//                         _ => (),
//                     }
//                     self.pos.x += edit.size.x;
//                     self.prev_pos = self.pos;
//                     self.scratch[0] = *c;
//                     self.ctx = Ctx::Past(self.scratch.as_mut_slice());
//                 }
//                 Cmd::Skip => self.pos.x += edit.size.x,
//                 Cmd::Future(mut current, future) => {
//                     match self.ctx {
//                         Ctx::Wrap(_, col) => {
//                             current = col;
//                         }
//                         Ctx::Future(col) => {
//                             current = col;
//                             self.ctx = Ctx::Future(future);
//                         }
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(
//                                 st,
//                                 canvas,
//                                 sheet,
//                                 cache,
//                                 &self.prev_pos,
//                                 &DEFAULT,
//                                 &edit.font,
//                             );
//                         }
//                         Ctx::None => self.ctx = Ctx::Future(future),
//                     }
//                     draw(
//                         u8_to_str(&[*c]),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &current,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x;
//                 }
//                 Cmd::Pull(mut current, prev) => {
//                     match self.ctx {
//                         Ctx::Wrap(_, col) => {
//                             current = col;
//                         }
//                         Ctx::Past(p) => {
//                             let st = u8_to_str(unsafe { &*p });
//                             draw(st, canvas, sheet, cache, &self.prev_pos, &prev, &edit.font);
//                             self.ctx = Ctx::None;
//                         }
//                         _ => (),
//                     }
//                     draw(
//                         u8_to_str(&[*c]),
//                         canvas,
//                         sheet,
//                         cache,
//                         &self.pos,
//                         &current,
//                         &edit.font,
//                     );
//                     self.pos.x += edit.size.x;
//                 }
//             },
//             Token::Space => self.pos.x += edit.size.x,
//             Token::Break => {
//                 self.pos.reset(0.0);
//             }
//         });
//     }

//     fn parse_no_draw(&mut self, edit: &mut Edit) {
//         edit.buffer.iter_mut(|token| match token {
//             Token::Group(group) => match (self.table.group)(&group) {
//                 Cmd::Put(_) => match self.ctx {
//                     Ctx::Past(_) => {
//                         self.ctx = Ctx::None;
//                     }
//                     Ctx::Future(_) => self.ctx = Ctx::None,
//                     Ctx::Wrap(..) => (),
//                     _ => (),
//                 },
//                 Cmd::Wrap(me, content) => match self.ctx {
//                     Ctx::Wrap(o_me, o_content) => {
//                         if me == o_me && content == o_content {
//                             self.ctx = Ctx::None;
//                         }
//                     }
//                     _ => {
//                         self.ctx = Ctx::Wrap(me, content);
//                     }
//                 },
//                 Cmd::Hold => {
//                     match self.ctx {
//                         Ctx::Future(_) | Ctx::Wrap(..) => {
//                             self.ctx = Ctx::None;
//                             return;
//                         }
//                         Ctx::Past(_) => (),
//                         _ => (),
//                     }
//                     self.ctx = Ctx::Past(group.as_mut_slice());
//                 }
//                 Cmd::Skip => (),
//                 Cmd::Future(_, future) => match self.ctx {
//                     Ctx::Wrap(_, _) => (),
//                     Ctx::Future(_) => {
//                         self.ctx = Ctx::Future(future);
//                     }
//                     Ctx::Past(_) => (),
//                     Ctx::None => self.ctx = Ctx::Future(future),
//                 },
//                 Cmd::Pull(..) => match self.ctx {
//                     Ctx::Wrap(_, _) => (),
//                     Ctx::Past(_) => {
//                         self.ctx = Ctx::None;
//                     }
//                     _ => (),
//                 },
//             },
//             Token::Single(c) => match (self.table.single)(c) {
//                 Cmd::Put(..) => match self.ctx {
//                     Ctx::Past(_) => {
//                         self.ctx = Ctx::None;
//                     }
//                     Ctx::Future(_) => self.ctx = Ctx::None,
//                     Ctx::Wrap(_, _) => (),
//                     _ => (),
//                 },
//                 Cmd::Wrap(me, content) => match self.ctx {
//                     Ctx::Wrap(o_me, o_content) => {
//                         if me == o_me && content == o_content {
//                             self.ctx = Ctx::None;
//                         }
//                     }
//                     _ => {
//                         self.ctx = Ctx::Wrap(me, content);
//                     }
//                 },
//                 Cmd::Hold => {
//                     match self.ctx {
//                         Ctx::Future(_) | Ctx::Wrap(_, _) => {
//                             self.ctx = Ctx::None;
//                         }
//                         Ctx::Past(_) => (),
//                         _ => (),
//                     }
//                     self.scratch[0] = *c;
//                     self.ctx = Ctx::Past(self.scratch.as_mut_slice());
//                 }
//                 Cmd::Skip => (),
//                 Cmd::Future(_, future) => match self.ctx {
//                     Ctx::Wrap(_, _) => (),
//                     Ctx::Future(_) => {
//                         self.ctx = Ctx::Future(future);
//                     }
//                     Ctx::Past(_) => (),
//                     Ctx::None => self.ctx = Ctx::Future(future),
//                 },
//                 Cmd::Pull(..) => match self.ctx {
//                     Ctx::Wrap(_, _) => (),
//                     Ctx::Past(_) => self.ctx = Ctx::None,
//                     _ => (),
//                 },
//             },
//             Token::Space => (),
//             Token::Break => (),
//         });
//     }
// }

// fn draw(
//     word: &str,
//     canvas: &mut Canvas,
//     sheet: &Sheet,
//     cache: &mut Cache,
//     pos: &Vec2<f32>,
//     color_id: &ColorId,
//     font_id: &FontId,
// ) {
// }
// fn u8_to_str(u8: &[u8]) -> &str {
//     unsafe { str::from_utf8_unchecked(u8) }
// }
// }
// fn u8_to_str(u8: &[u8]) -> &str {
//     unsafe { str::from_utf8_unchecked(u8) }
// }
