use khronos_egl as egl;
use std::ptr;
use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_compositor::WlCompositor, wl_registry, wl_surface::WlSurface},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_egl::WlEglSurface;
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

const PLATFORM_WAYLAND_KHR: egl::Enum = 0x31D8;

pub fn run<T: egl::api::EGL1_5>(egl: &egl::Instance<T>) {
    // Setup the Wayland client.
    let connection = Connection::connect_to_env().expect("unable to connect to the wayland server");

    // Roundtrip to retrieve the globals list
    let (globals, mut event_queue) = registry_queue_init::<State>(&connection).unwrap();
    let qh = event_queue.handle();

    // Get the compositor.
    let compositor: WlCompositor = globals.bind(&qh, 1..=1, ()).unwrap();

    // Xdg protocol.
    let xdg: xdg_wm_base::XdgWmBase = globals.bind(&qh, 1..=1, ()).unwrap();

    // Create a surface.
    // Note that it must be kept alive to the end of execution.
    let surface = compositor.create_surface(&qh, ());
    let xdg_surface = xdg.get_xdg_surface(&surface, &qh, ());
    let xdg_toplevel = xdg_surface.get_toplevel(&qh, ());
    xdg_toplevel.set_app_id("khronos-egl-test".to_string());
    xdg_toplevel.set_title("Test".to_string());
    surface.commit();

    let mut state = State {
        running: true,
        pending_size: (0, 0),
        size: (800, 600),
        needs_redraw: false,
    };
    // Wait for the initial configure before creating the EGL window.
    while state.running && !state.needs_redraw {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
    if !state.running {
        return;
    }

    // Setup EGL.
    let display = unsafe {
        egl.get_platform_display(
            PLATFORM_WAYLAND_KHR,
            connection.backend().display_ptr().cast(),
            &[egl::ATTRIB_NONE],
        )
        .expect("unable to get the EGL display")
    };
    egl.initialize(display).unwrap();

    // Setup Open GL.
    egl.bind_api(egl::OPENGL_API)
        .expect("unable to select OpenGL API");
    let (context, config) = create_context(egl, display);
    let window = WlEglSurface::new(surface.id(), state.size.0, state.size.1)
        .expect("unable to create a Wayland EGL window");
    let egl_surface = unsafe {
        egl.create_platform_window_surface(
            display,
            config,
            window.ptr().cast_mut(),
            &[egl::ATTRIB_NONE],
        )
        .expect("unable to create an EGL surface")
    };
    egl.make_current(display, Some(egl_surface), Some(egl_surface), Some(context))
        .expect("unable to bind the context");
    // Redraws are configure-driven. Avoid waiting for frame callbacks that
    // the compositor may withhold while the surface is hidden.
    egl.swap_interval(display, 0)
        .expect("unable to disable EGL swap throttling");
    gl::load_with(|name| {
        egl.get_proc_address(name)
            .map_or(ptr::null(), |proc| proc as *const std::ffi::c_void)
    });

    while state.running {
        if state.needs_redraw {
            state.needs_redraw = false;
            window.resize(state.size.0, state.size.1, 0, 0);
            unsafe {
                gl::ClearColor(1.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                assert_eq!(gl::GetError(), gl::NO_ERROR, "OpenGL rendering failed");
            }
            egl.swap_buffers(display, egl_surface)
                .expect("unable to post the surface content");
        }
        event_queue.blocking_dispatch(&mut state).unwrap();
    }

    egl.make_current(display, None, None, None).unwrap();
    egl.destroy_surface(display, egl_surface).unwrap();
    egl.destroy_context(display, context).unwrap();
    egl.terminate(display).unwrap();
    egl.release_thread().unwrap();
    drop(window);
    xdg_toplevel.destroy();
    xdg_surface.destroy();
    surface.destroy();
    xdg.destroy();
    connection.flush().unwrap();
}

fn create_context<T: egl::api::EGL1_5>(
    egl: &egl::Instance<T>,
    display: egl::Display,
) -> (egl::Context, egl::Config) {
    let attributes = [
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::RENDERABLE_TYPE,
        egl::OPENGL_BIT,
        egl::SURFACE_TYPE,
        egl::WINDOW_BIT,
        // EGL silently clamps swap intervals to the configuration's limits.
        egl::MIN_SWAP_INTERVAL,
        0,
        egl::NONE,
    ];

    let config = egl
        .choose_first_config(display, &attributes)
        .expect("unable to choose an EGL configuration")
        .expect("no EGL configuration supporting swap interval zero found");

    let context_attributes = [
        egl::CONTEXT_MAJOR_VERSION,
        4,
        egl::CONTEXT_MINOR_VERSION,
        0,
        egl::CONTEXT_OPENGL_PROFILE_MASK,
        egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
        egl::NONE,
    ];

    let context = egl
        .create_context(display, config, None, &context_attributes)
        .expect("unable to create an EGL context");

    (context, config)
}

struct State {
    running: bool,
    pending_size: (i32, i32),
    size: (i32, i32),
    needs_redraw: bool,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(State: WlCompositor);
delegate_noop!(State: ignore WlSurface);

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        xdg: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            let (width, height) = state.pending_size;
            if width > 0 {
                state.size.0 = width;
            }
            if height > 0 {
                state.size.1 = height;
            }
            state.needs_redraw = true;
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Configure { width, height, .. } => {
                state.pending_size = (width, height);
            }
            xdg_toplevel::Event::Close => state.running = false,
            _ => {}
        }
    }
}
