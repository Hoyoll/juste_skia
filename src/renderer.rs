use std::{
    collections::{HashMap, VecDeque},
    ffi::CString,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use gl::{GetIntegerv, RenderbufferStorage, types};
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::{ApiPreference, DisplayBuilder};

use juste::{Element, Io, SignalBus, Vec2};
use raw_window_handle::HasWindowHandle;
use skia_safe::{
    Font, Image, Paint, Rect,
    colors::WHITE,
    gpu::{
        DirectContext, Protected, SurfaceOrigin, backend_render_targets,
        ganesh::gl::direct_contexts,
        gl::{Format, FramebufferInfo, Interface},
        surfaces::wrap_backend_render_target,
    },
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, KeyEvent, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

fn run(element: Element, attr: WindowAttributes) {
    let event_loop = EventLoop::builder().build().unwrap();
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
    io.window_size = Vec2::new(size.width, size.height);
    let mut app = App {
        element,
        graphic: None,
        cache: Cache {
            io,
            bus: HashMap::new(),
            image: HashMap::new(),
            font: HashMap::new(),
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

pub struct Cache {
    pub io: Io,
    pub bus: SignalBus,
    pub image: HashMap<String, Image>,
    pub font: HashMap<&'static str, Font>,
    pub window: Window,
    pub gl_config: Config,
}

struct App {
    element: Element,
    cache: Cache,
    graphic: Option<Graphic>,
}

impl App {
    fn draw(&mut self) {
        match self.graphic.as_mut() {
            Some(graphic) => {
                let canvas = graphic.sk_surface.canvas();
                canvas.clear(WHITE);
                canvas.draw_rect(
                    Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
                    Paint::default().set_argb(255, 0, 255, 0),
                );

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
                let backend_render_target = backend_render_targets::make_gl(
                    (size.width as i32, size.height as i32),
                    0,
                    8,
                    graphic.fb_info,
                );
                graphic.sk_surface = wrap_backend_render_target(
                    &mut graphic.gr_context,
                    &backend_render_target,
                    SurfaceOrigin::BottomLeft,
                    skia_safe::ColorType::RGBA8888,
                    None,
                    None,
                )
                .unwrap();
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

impl ApplicationHandler for App {
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
        _event_loop: &ActiveEventLoop,
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
                    state: _,
                    repeat: _,
                    ..
                } => match physical_key {
                    PhysicalKey::Code(key) => match key {
                        KeyCode::Space => self.cache.window.request_redraw(),
                        _ => (),
                    },
                    _ => (),
                },
            },
            WindowEvent::RedrawRequested => {
                self.draw();
            }

            WindowEvent::Resized(_) => {
                self.resize_canvas();
            }

            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => match delta {
                MouseScrollDelta::LineDelta(_, _) => {}
                MouseScrollDelta::PixelDelta(_) => {}
            },
            _ => (),
        }
    }
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
    }
}
