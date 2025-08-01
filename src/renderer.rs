use std::{
    collections::{HashMap, HashSet},
    ffi::CString,
    fs::read,
    num::NonZeroU32,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::{Duration, Instant},
};

use gl::{GetIntegerv, types};
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::{ApiPreference, DisplayBuilder};

use juste::{From, Io, Message, Mode, On, SignalBus, Src, Vec2, Win};
use raw_window_handle::HasWindowHandle;
use reqwest::blocking;
use skia_safe::{
    Data, FontMgr, FontStyle, Image, Typeface,
    gpu::{
        DirectContext, Protected, SurfaceOrigin, backend_render_targets,
        ganesh::gl::direct_contexts,
        gl::{Format, FramebufferInfo, Interface},
        surfaces::wrap_backend_render_target,
    },
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    app::App,
    io::{filter_keyboard, filter_mouse},
};

pub fn run<T: App>(app: T, attr: WindowAttributes) {
    let event_loop: EventLoop<Message> = EventLoop::with_user_event().build().unwrap();

    let (window, gl_config) = {
        let display_builder = DisplayBuilder::new()
            .with_window_attributes(Some(attr))
            .with_preference(ApiPreference::FallbackEgl);
        let template = ConfigTemplateBuilder::new().with_alpha_size(8);
        let (window, config) = display_builder
            .build(&event_loop, template, |mut config| config.next().unwrap())
            .unwrap();
        (window.unwrap(), config)
    };
    let mut io = Io::new();
    let size = window.inner_size();
    io.window_size = Vec2::new(size.width as f32, size.height as f32);
    let proxy = event_loop.create_proxy();
    let mut app = Renderer::<T> {
        app,
        graphic: None,
        cache: Cache {
            io,
            bus: HashMap::new(),
            image: Images::new(),
            proxy,
            font: Fonts {
                font_mgr: FontMgr::new(),
                fonts: HashMap::new(),
            },
            window,
            gl_config,
        },
    };
    let _ = event_loop.run_app(&mut app);
}

struct Graphic {
    gl_surface: Surface<WindowSurface>,
    gr_context: DirectContext,
    fb_info: FramebufferInfo,
    context: PossiblyCurrentContext,
    sk_surface: skia_safe::Surface,
}

impl Graphic {
    fn rebuild_skia_surface(&mut self, size: Vec2<i32>) {
        let backend_render_target =
            backend_render_targets::make_gl((size.x, size.y), 0, 8, self.fb_info);
        self.sk_surface = wrap_backend_render_target(
            &mut self.gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();
    }

    pub fn destroy(&mut self) {
        self.gr_context.abandon();
    }
}

pub enum Pick<T> {
    One(T),
    All,
}
pub struct Images {
    pub img: HashMap<Src, Image>,
    pub sender: Sender<(Src, Image)>,
    pub receiver: Receiver<(Src, Image)>,
    pending: HashSet<Src>,
}

impl Images {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            img: HashMap::new(),
            sender,
            receiver,
            pending: HashSet::new(),
        }
    }

    pub fn load(&mut self, name: &Src) -> Option<&Image> {
        if !self.img.contains_key(name) {
            match name {
                Src::Sys(file) => match read(file) {
                    Err(_) => return None,
                    Ok(bytes) => {
                        let data = Data::new_copy(&bytes);
                        match Image::from_encoded(&data) {
                            Some(img) => {
                                self.img.insert(name.clone(), img);
                            }
                            None => return None,
                        }
                    }
                },
                Src::Url(url) => {
                    if !self.pending.contains(name) {
                        self.pending.insert(name.clone());
                        load_url(url.to_string(), self.sender.clone());
                    }
                    match self.receiver.try_recv() {
                        Ok((key, image)) => {
                            self.pending.remove(&key);
                            self.img.insert(key, image);
                        }
                        Err(_) => (),
                    }
                }
            }
        }
        self.img.get(name)
    }

    pub fn invalidate(&mut self, pick: Pick<Src>) {
        match pick {
            Pick::All => self.img.clear(),
            Pick::One(img) => {
                self.img.remove(&img);
            }
        }
    }
}

fn load_url(url: String, sender: Sender<(Src, Image)>) {
    thread::spawn(move || {
        let req = blocking::get(&url);
        match req {
            Ok(response) => match response.bytes() {
                Ok(b) => {
                    let data = Data::new_copy(&b);
                    let img = Image::from_encoded(data);
                    if let Some(image) = img {
                        let _ = sender.send((Src::Url(url), image));
                    }
                }
                Err(_) => (),
            },
            Err(_) => (),
        }
    });
}

pub struct Fonts {
    pub font_mgr: FontMgr,
    pub fonts: HashMap<juste::Font, Typeface>,
}

impl Fonts {
    pub fn new() -> Self {
        let font_mgr = FontMgr::new();
        Self {
            font_mgr,
            fonts: HashMap::new(),
        }
    }

    pub fn load(&mut self, font: &juste::Font) -> Option<&Typeface> {
        if !self.fonts.contains_key(font) {
            match font {
                juste::Font::File(name, idx) => match read(name) {
                    Err(_) => return None,
                    Ok(byte) => {
                        let data = Data::new_copy(&byte);
                        let tf = self.font_mgr.new_from_data(&data, Some(*idx as usize));
                        match tf {
                            Some(tfc) => {
                                self.fonts.insert(*font, tfc);
                            }
                            None => return None,
                        }
                    }
                },
                juste::Font::Sys(str, mode) => {
                    let tf = self.font_mgr.match_family_style(str, font_style(mode));
                    match tf {
                        Some(tfc) => {
                            self.fonts.insert(*font, tfc);
                        }
                        None => {
                            println!("font does not exist on the system!");
                            return None;
                        }
                    }
                }
            }
        }
        self.fonts.get(font)
    }

    pub fn invalidate(&mut self, pick: Pick<juste::Font>) {
        match pick {
            Pick::One(f) => {
                self.fonts.remove(&f);
            }
            Pick::All => self.fonts.clear(),
        }
    }
}

fn font_style(mode: &Mode) -> FontStyle {
    match mode {
        Mode::Normal => FontStyle::normal(),
        Mode::Bold => FontStyle::bold(),
        Mode::Italic => FontStyle::italic(),
    }
}

pub struct Cache {
    pub io: Io,
    pub bus: SignalBus,
    pub image: Images,
    pub proxy: EventLoopProxy<Message>,
    pub font: Fonts,
    pub window: Window,
    pub gl_config: Config,
}

pub struct Renderer<T: App> {
    pub cache: Cache,
    pub app: T,
    graphic: Option<Graphic>,
}

impl<T: App> Renderer<T> {
    fn draw(&mut self) {
        match self.graphic.as_mut() {
            Some(graphic) => {
                self.app.draw(&mut self.cache, graphic.sk_surface.canvas());
                graphic.gr_context.flush_and_submit();
                graphic.gl_surface.swap_buffers(&graphic.context).unwrap();
            }
            None => (),
        }
    }

    fn resize_canvas(&mut self) {
        match self.graphic.as_mut() {
            Some(graphic) => {
                let size = self.cache.window.inner_size();
                graphic.rebuild_skia_surface(Vec2::new(size.width as i32, size.height as i32));
            }
            None => (),
        }
    }

    fn build_canvas(&mut self) {
        // A bunch of boring config basically
        let raw_handle = self.cache.window.window_handle().unwrap();
        let gl_display = self.cache.gl_config.display();

        let context_attr = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(None)) // I just pick whatever version here, idk my laptop pretty old
            .build(Some(raw_handle.into()));
        let width = NonZeroU32::new(self.cache.window.inner_size().width.max(1)).unwrap();
        let height = NonZeroU32::new(self.cache.window.inner_size().height.max(1)).unwrap();
        let gl_attr = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_handle.into(),
            width,
            height,
        );

        // now this is where the fun stuff starts
        let not_current = unsafe {
            gl_display
                .create_context(&self.cache.gl_config, &context_attr)
                .unwrap()
        };
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&self.cache.gl_config, &gl_attr)
                .unwrap()
        };
        // swap the buffer, you're gonna do this a lot btw
        let context = not_current.make_current(&gl_surface).unwrap();
        // We load opengl function pointers here
        gl::load_with(|s| {
            let cstr = CString::new(s).unwrap();
            gl_display.get_proc_address(&cstr) as *const _
        });

        // basically just a bunch config for skia
        let interface = Interface::new_native().unwrap();
        let mut gr_context = direct_contexts::make_gl(interface, None).unwrap();
        let fb_info = {
            let mut fboid: types::GLint = 0;
            unsafe {
                GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid);
            }
            FramebufferInfo {
                fboid: fboid as u32,
                format: Format::RGBA8.into(),
                protected: Protected::No, // you want access to the fb info y'know
            }
        };
        let size = self.cache.window.inner_size();
        let backend_render_target =
            backend_render_targets::make_gl((size.width as i32, size.height as i32), 0, 8, fb_info);

        // now build the damn canvas finally
        let surface = wrap_backend_render_target(
            &mut gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();
        self.graphic = Some(Graphic {
            gl_surface,
            gr_context,
            fb_info,
            context,
            sk_surface: surface,
        })
    }
}

impl<T: App> ApplicationHandler<Message> for Renderer<T> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.build_canvas();
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::ResumeTimeReached { .. } | StartCause::Init => {
                self.cache.window.request_redraw();
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + Duration::from_millis(16),
                ));
            }
            _ => (),
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event {
                KeyEvent {
                    physical_key,
                    logical_key: _,
                    text: _,
                    location: _,
                    state,
                    repeat: _,
                    ..
                } => match physical_key {
                    PhysicalKey::Code(key) => {
                        let k = filter_keyboard(key);
                        let input = match state {
                            ElementState::Pressed => On::Press(From::Key(k)),
                            ElementState::Released => On::Release(From::Key(k)),
                        };
                        self.cache.io.pool(input);
                    }

                    _ => (),
                },
            },
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let b = filter_mouse(button);
                let m = match state {
                    ElementState::Pressed => On::Press(From::Mouse(b)),
                    ElementState::Released => On::Release(From::Mouse(b)),
                };
                self.cache.io.pool(m);
            }
            WindowEvent::Resized(size) => {
                self.cache.io.window_size = Vec2::new(size.width as f32, size.height as f32);
                self.cache.io.pool(On::Window(Win::Resize));
                self.resize_canvas();
            }

            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    self.cache.io.scroll = y;
                }
                MouseScrollDelta::PixelDelta(_) => {}
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => (),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphic.as_mut().unwrap().destroy();
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Message) {
        self.app.user_event(event, event_loop);
    }
}
