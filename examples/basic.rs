use juste::{
    element::{Bound, Element, Listeners, Tag},
    genus::{Frame, Genus},
    io::{From, Input, Key, On},
    style::{DEFAULT, Sheet},
};
use juste_skia::{
    app::App,
    passes::{first_pass, second_pass},
    plug::Plug,
};
use libloading::{Library, Symbol};
use winit::window::WindowAttributes;

use std::{any::Any, fs};

fn load_plugin(path: &str) -> String {
    fs::read_to_string(path)
        .expect("Could not read plugin path")
        .trim()
        .to_string()
}

pub struct Head {
    element: Plug<Element>,
    sheet: Plug<Sheet>,
    listeners: Plug<Listeners>,
}

impl Head {
    pub fn new() -> Self {
        let path = load_plugin("D:/project/rust/plug/out/current_plugin.txt");
        Self {
            element: Plug::new(&path, "build_element"),
            sheet: Plug::new(&path, "build_sheet"),
            listeners: Plug::new(&path, "build_listener"),
        }
    }
}

impl Head {
    fn io_event(&mut self, io: &juste::io::Io) {
        match io.input {
            Input::Combo(_) => (),
            Input::Single(io) => match io {
                On::Press(From::Key(Key::F5)) => self.element.reload(),
                On::Press(From::Key(Key::F4)) => self.sheet.reload(),
                On::Press(From::Key(Key::F6)) => self.listeners.reload(),
                _ => (),
            },
            Input::None => (),
        }
    }
}

impl App for Head {
    fn draw(&mut self, cache: &mut juste_skia::renderer::Cache, canvas: &skia_safe::Canvas) {
        self.io_event(&cache.io);
        unsafe {
            first_pass(
                &mut *self.element.data,
                cache,
                &mut *self.listeners.data,
                &mut *self.sheet.data,
            );
            second_pass(
                &mut *self.element.data,
                canvas,
                cache,
                &mut *self.sheet.data,
            );
        }
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
    let attr = WindowAttributes::default();
    juste_skia::renderer::run(Head::new(), attr);
}
