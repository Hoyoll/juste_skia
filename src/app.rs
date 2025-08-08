use juste::{element::Message, io::Io};
use skia_safe::Canvas;
use winit::event_loop::ActiveEventLoop;

use crate::Cache;

pub trait App {
    fn draw(&mut self, cache: &mut Cache, canvas: &Canvas);
    fn user_event(&mut self, message: Message, event_loop: &ActiveEventLoop);
    fn io_event(&mut self, io: &Io);
}
