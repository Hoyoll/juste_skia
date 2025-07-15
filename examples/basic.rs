use std::num::NonZeroU32;

use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::Display,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use raw_window_handle::HasRawWindowHandle;
use skia_safe::{ColorType, Surface as SkiaSurface, gpu};

fn main() {
    // === 1. SETUP WINDOW ===
    let event_loop = EventLoop::new().unwrap();
    let window_builder = WindowBuilder::new().with_title("Skia GPU Demo");

    // === 2. SETUP OPENGL CONTEXT ===
    let (window, gl_config) = {
        let template = ConfigTemplateBuilder::new().with_alpha_size(8).build();
        let display_builder =
            glutin_winit::DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (window, config) = display_builder
            .build(&event_loop, template, |configs| configs[0].clone())
            .unwrap();

        (window.unwrap(), config)
    };

    let raw_handle = window.raw_window_handle();
    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(
            3, 3,
        ))))
        .build(Some(raw_handle));

    let not_current = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap()
    };

    let width = NonZeroU32::new(window.inner_size().width.max(1)).unwrap();
    let height = NonZeroU32::new(window.inner_size().height.max(1)).unwrap();

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(raw_handle, width, height);

    let gl_surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let context = not_current.make_current(&gl_surface).unwrap();

    // === 3. LOAD GL FUNCTIONS ===
    gl::load_with(|s| gl_display.get_proc_address(s) as *const _);

    // === 4. SETUP SKIA CONTEXT ===
    let interface = gpu::gl::Interface::new_native().unwrap();
    let mut gr_context = gpu::DirectContext::new_gl(Some(interface), None).unwrap();

    let fb_info = {
        let mut fboid: gl::types::GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid);
        }
        gpu::gl::FramebufferInfo {
            fboid: fboid as u32,
            format: gpu::gl::Format::RGBA8.into(),
        }
    };

    let mut size = window.inner_size();

    let mut backend_render_target =
        gpu::BackendRenderTarget::new_gl((size.width as i32, size.height as i32), 0, 8, fb_info);

    let mut surface = SkiaSurface::from_backend_render_target(
        &mut gr_context,
        &backend_render_target,
        gpu::SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .unwrap();

    let canvas = surface.canvas();

    // === 5. RUN LOOP ===
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                // DRAW WITH SKIA
                canvas.clear(skia_safe::colors::WHITE);

                let paint = skia_safe::Paint::default();
                canvas.draw_str("GPU SKIA DRAWING!", (50, 100), &paint);

                surface.flush_and_submit();
                context.swap_buffers(&gl_surface).unwrap();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;

                let width = NonZeroU32::new(size.width.max(1)).unwrap();
                let height = NonZeroU32::new(size.height.max(1)).unwrap();

                gl_surface.resize(&context, width, height);

                backend_render_target = gpu::BackendRenderTarget::new_gl(
                    (size.width as i32, size.height as i32),
                    0,
                    8,
                    fb_info,
                );

                surface = SkiaSurface::from_backend_render_target(
                    &mut gr_context,
                    &backend_render_target,
                    gpu::SurfaceOrigin::BottomLeft,
                    ColorType::RGBA8888,
                    None,
                    None,
                )
                .unwrap();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        }
    });
}
