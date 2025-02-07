mod project;
mod ui;

use fltk::{app, enums::Color, prelude::*, window::Window};
use ui::{UserInterface, WgpuState};

#[derive(Debug, Copy, Clone)]
pub enum AppEvent {
    ResizePreview(u32, u32),
    RedrawPreview,
}

struct MainApp<'a> {
    fltk_app: app::App,
    event_sender: app::Sender<AppEvent>,
    event_receiver: app::Receiver<AppEvent>,
    fltk_ui: UserInterface,
    preview_subwindow: Window,
    wgpu_state: WgpuState<'a>,
}

impl MainApp<'_> {
    pub fn new() -> Self {
        let fltk_app = app::App::default().with_scheme(app::Scheme::Oxy);
        let (event_sender, event_receiver) = app::channel::<AppEvent>();

        let mut ui = ui::UserInterface::make_window();
        Self::init_main_window(&mut ui.main_window);
        ui.main_window.show();

        let mut preview_subwindow = Self::make_and_add_preview_subwindow(&mut ui.preview_group);
        preview_subwindow.resize_callback(move |_, _, _, w, h| {
            event_sender.send(AppEvent::ResizePreview(w as u32, h as u32));
        });
        preview_subwindow.show();

        let wgpu_state =
            futures_lite::future::block_on(ui::WgpuState::new(preview_subwindow.clone()));
        wgpu_state.redraw();

        Self {
            fltk_app,
            event_sender,
            event_receiver,
            fltk_ui: ui,
            preview_subwindow,
            wgpu_state,
        }
    }

    fn init_main_window(main_window: &mut Window) {
        main_window.set_label(concat!(
            env!("CARGO_CRATE_NAME"),
            " v",
            env!("CARGO_PKG_VERSION")
        ));
        main_window.clone().center_screen();
    }

    fn make_and_add_preview_subwindow(preview_group: &mut impl GroupExt) -> Window {
        let mut preview_subwindow = Window::new(0, 0, 100, 100, None);
        preview_subwindow.set_color(Color::Black);
        preview_subwindow.end();
        preview_group.add(&preview_subwindow);
        preview_subwindow.clone().size_of_parent();
        preview_subwindow.clone().center_of_parent();
        preview_subwindow
    }

    pub fn run_loop(&mut self) {
        while self.fltk_app.wait() {
            if let Some(event) = self.event_receiver.recv() {
                match event {
                    AppEvent::ResizePreview(width, height) => {
                        self.wgpu_state.resize_surface(width, height);
                        self.wgpu_state.redraw();
                    }
                    AppEvent::RedrawPreview => self.wgpu_state.redraw(),
                }
            }
        }
    }
}

fn main() {
    MainApp::new().run_loop();
}
