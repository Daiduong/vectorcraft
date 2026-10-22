//! Linux pen tablets: winit reports barrel buttons as [`winit::event::ButtonSource::Unknown`]
//! (for example `BTN_STYLUS`), and egui-winit drops those before they reach the UI (#3632).

use eframe::AppCreator;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use winit::event_loop::run_on_demand::EventLoopExtRunOnDemand as _;

/// `BTN_STYLUS` and neighbours (`linux/input-event-codes.h`): pen barrel buttons Wayland passes
/// through the pointer protocol as non-standard mouse buttons.
const BTN_STYLUS: u16 = 0x014b;
const BTN_STYLUS2: u16 = 0x014c;
const BTN_STYLUS3: u16 = 0x014d;

/// Run VectorCraft's returning Linux loop, mapping pen barrel buttons before egui sees them.
pub fn run_native(app_name: &str, mut native_options: eframe::NativeOptions, app_creator: AppCreator<'_>) -> eframe::Result {
    let mut builder = EventLoop::builder();
    if let Some(hook) = native_options.event_loop_builder.take() {
        hook(&mut builder);
    }
    let mut event_loop = builder.build().map_err(eframe::Error::WinitEventLoop)?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let inner = eframe::create_native(app_name, native_options, app_creator, &event_loop);
    // Own the handler so shutdown drops/saves it, while retaining borrowed app creators.
    let app = Forwarder { inner };
    event_loop.run_app_on_demand(app).map_err(eframe::Error::WinitEventLoop)?;
    Ok(())
}

struct Forwarder<T> {
    inner: T,
}

impl<T: ApplicationHandler> ApplicationHandler for Forwarder<T> {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.resumed(event_loop);
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        self.inner.window_event(event_loop, window_id, map_window_event(event));
    }

    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.inner.new_events(event_loop, cause);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.can_create_surfaces(event_loop);
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.proxy_wake_up(event_loop);
    }

    fn device_event(&mut self, event_loop: &dyn ActiveEventLoop, device_id: Option<DeviceId>, event: DeviceEvent) {
        self.inner.device_event(event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.about_to_wait(event_loop);
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.destroy_surfaces(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.memory_warning(event_loop);
    }
}

/// Map a tablet button code winit labels `ButtonSource::Unknown` to a standard mouse button.
pub(crate) fn map_tablet_button(code: u16) -> Option<winit::event::MouseButton> {
    use winit::event::MouseButton;
    match code {
        BTN_STYLUS | BTN_STYLUS2 => Some(MouseButton::Right),
        BTN_STYLUS3 => Some(MouseButton::Middle),
        _ => None,
    }
}

fn map_window_event(mut event: WindowEvent) -> WindowEvent {
    if let WindowEvent::PointerButton { button, .. } = &mut event
        && let winit::event::ButtonSource::Unknown(code) = *button
        && let Some(mapped) = map_tablet_button(code)
    {
        *button = winit::event::ButtonSource::Mouse(mapped);
    }
    event
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::{ElementState, MouseButton};

    #[test]
    fn stylus_barrel_buttons_become_secondary_clicks() {
        assert_eq!(map_tablet_button(BTN_STYLUS), Some(MouseButton::Right));
        assert_eq!(map_tablet_button(BTN_STYLUS2), Some(MouseButton::Right));
        assert_eq!(map_tablet_button(BTN_STYLUS3), Some(MouseButton::Middle));
        assert_eq!(map_tablet_button(0x110), None);
    }

    #[test]
    fn window_pointer_input_preserves_metadata_when_remapped() {
        use winit::dpi::PhysicalPosition;
        use winit::event::ButtonSource;
        for state in [ElementState::Pressed, ElementState::Released] {
            for (code, expected) in [(BTN_STYLUS, MouseButton::Right), (BTN_STYLUS2, MouseButton::Right), (BTN_STYLUS3, MouseButton::Middle)] {
                let mapped = map_window_event(WindowEvent::PointerButton {
                    device_id: None,
                    state,
                    position: PhysicalPosition::new(12.5, 23.75),
                    primary: false,
                    button: ButtonSource::Unknown(code),
                    is_macos_activation_click: false,
                });
                assert_eq!(
                    mapped,
                    WindowEvent::PointerButton {
                        device_id: None,
                        state,
                        position: PhysicalPosition::new(12.5, 23.75),
                        primary: false,
                        button: ButtonSource::Mouse(expected),
                        is_macos_activation_click: false,
                    }
                );
            }
        }
        for button in [ButtonSource::Unknown(0x110), ButtonSource::Mouse(MouseButton::Left)] {
            let original = WindowEvent::PointerButton {
                device_id: None,
                state: ElementState::Pressed,
                position: PhysicalPosition::new(1.0, 2.0),
                primary: true,
                button,
                is_macos_activation_click: false,
            };
            assert_eq!(map_window_event(original.clone()), original);
        }
        assert_eq!(map_window_event(WindowEvent::Focused(true)), WindowEvent::Focused(true));
    }

    /// A real native connection with no windows, files, clipboard or portal requests.
    #[test]
    #[ignore = "requires a native Wayland session; creates no windows or surfaces"]
    fn native_wayland_wrapper_forwards_surface_creation_and_wakeup() {
        use std::cell::Cell;
        use std::time::{Duration, Instant};
        use winit::event_loop::EventLoopProxy;
        use winit::platform::wayland::EventLoopBuilderExtWayland as _;

        struct Probe<'a> {
            created: &'a Cell<bool>,
            woke: &'a Cell<bool>,
            dropped: &'a Cell<bool>,
            timed_out: &'a Cell<bool>,
            started: Instant,
            proxy: EventLoopProxy,
        }
        impl ApplicationHandler for Probe<'_> {
            fn can_create_surfaces(&mut self, _: &dyn ActiveEventLoop) {
                self.created.set(true);
                self.proxy.wake_up();
            }
            fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
                self.woke.set(true);
                event_loop.exit();
            }
            fn window_event(&mut self, _: &dyn ActiveEventLoop, _: WindowId, _: WindowEvent) {}
            fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
                if self.started.elapsed() > Duration::from_secs(5) {
                    self.timed_out.set(true);
                    event_loop.exit();
                }
            }
        }
        impl Drop for Probe<'_> {
            fn drop(&mut self) {
                self.dropped.set(true);
            }
        }
        let (created, woke, dropped, timed_out) = (Cell::new(false), Cell::new(false), Cell::new(false), Cell::new(false));
        let mut builder = EventLoop::builder();
        builder.with_wayland().with_any_thread(true);
        let mut event_loop = builder.build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let proxy = event_loop.create_proxy();
        let probe = Probe { created: &created, woke: &woke, dropped: &dropped, timed_out: &timed_out, started: Instant::now(), proxy };
        event_loop.run_app_on_demand(Forwarder { inner: probe }).unwrap();
        assert!(created.get() && woke.get() && dropped.get());
        assert!(!timed_out.get(), "native wake-up did not reach the wrapped application");
    }
}
