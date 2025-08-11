use crate::renderer::Cache;
use juste::element::Message;
use skia_safe::Canvas;
use winit::event_loop::ActiveEventLoop;

pub trait App {
    fn draw(&mut self, cache: &mut Cache, canvas: &Canvas);
    fn user_event(&mut self, message: Message, cache: &mut Cache, event_loop: &ActiveEventLoop);
}
