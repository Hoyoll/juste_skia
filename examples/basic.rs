use std::any::Any;

use juste::{
    element::{Bound, Element, Tag},
    genus::{Frame, Genus},
    renderer,
    style::DEFAULT,
};
use juste_skia::{
    app::App,
    passes::{first_pass, second_pass},
};
use skia_safe::{Size, runtime_effect::RuntimeShaderBuilder};
use winit::window::WindowAttributes;
struct Head {
    element: Element,
}

impl Head {
    pub fn new() -> Self {
        let frame = Frame::new();
        let element = Element {
            tag: Tag::Def,
            genus: Genus::Frame(frame),
            bound: Bound::new(),
            listener: Some(DEFAULT),
        };
        Self { element }
    }
}

impl App for Head {
    fn draw(&mut self, cache: &mut juste_skia::renderer::Cache, canvas: &skia_safe::Canvas) {
        first_pass(&mut self.element, cache);
        second_pass(&mut self.element, canvas, cache);
    }

    fn user_event(
        &mut self,
        message: juste::element::Message,
        cache: &mut juste_skia::renderer::Cache,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
    }
}

fn main() {
    //
    let attr = WindowAttributes::default();
    // fs::read(path)
    // String::from_utf8(fs::read()
    // juste_skia::renderer::run(Head::new(), attr, sheet);
}
