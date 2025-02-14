mod project;
mod ui;

use env_logger::Env;
use ffmpeg_next::{Codec, Rational, frame::Video, media::Type};
use fltk::{
    app::{self, Sender},
    dialog::{FileDialogAction, FileDialogType, NativeFileChooser},
    enums::{Color, Event, Shortcut},
    menu::MenuFlag,
    prelude::*,
    window::Window,
};
use log::{error, info, warn};
use project::MediaProject;
use slotmap::SlotMap;
use ui::{UserInterface, WgpuState};

pub const APP_TITLE_AND_VERSION: &str =
    concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Copy, Clone)]
pub enum AppEvent {
    ResizePreview(u32, u32),
    RedrawPreview,
    MenuFileImport,
}

struct MainApp<'a> {
    fltk_app: app::App,
    #[allow(unused)]
    event_sender: app::Sender<AppEvent>,
    event_receiver: app::Receiver<AppEvent>,
    #[allow(unused)]
    fltk_ui: UserInterface,
    #[allow(unused)]
    preview_subwindow: Window,
    wgpu_state: WgpuState<'a>,
    #[allow(unused)]
    open_project: MediaProject,
}

impl MainApp<'_> {
    pub fn new() -> Self {
        info!("starting up!");
        info!("{APP_TITLE_AND_VERSION}");

        let fltk_app = app::App::default().with_scheme(app::Scheme::Oxy);
        let (event_sender, event_receiver) = app::channel::<AppEvent>();
        info!("created fltk app");

        let mut ui = ui::UserInterface::make_window();
        Self::init_main_window(&mut ui, event_sender);
        ui.main_window.show();
        info!("initialized main window");

        let mut preview_subwindow = Self::make_and_add_preview_subwindow(&mut ui.preview_group);
        preview_subwindow.resize_callback(move |_, _, _, w, h| {
            event_sender.send(AppEvent::ResizePreview(w as u32, h as u32));
        });
        preview_subwindow.show();
        info!("initialized preview subwindow");

        let wgpu_state =
            futures_lite::future::block_on(ui::WgpuState::new(preview_subwindow.clone()));
        wgpu_state.redraw();
        info!("initialized wgpu & preview rendering");

        Self {
            fltk_app,
            event_sender,
            event_receiver,
            fltk_ui: ui,
            preview_subwindow,
            wgpu_state,
            open_project: MediaProject {
                fps: Rational::new(30, 1).into(),
                frame_count: 300,
                media: SlotMap::default(),
            },
        }
    }

    fn init_main_window(ui: &mut UserInterface, event_sender: Sender<AppEvent>) {
        ui.main_window.set_label(concat!(
            env!("CARGO_CRATE_NAME"),
            " v",
            env!("CARGO_PKG_VERSION")
        ));
        ui.main_window.clone().center_screen();
        ui.main_window.set_callback(|_| {
            if fltk::app::event() == Event::Close {
                app::quit();
            }
        });

        ui.main_menu_bar.add_emit(
            "File/Import Ya'll",
            Shortcut::None,
            MenuFlag::empty(),
            event_sender,
            AppEvent::MenuFileImport,
        );
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
                    AppEvent::MenuFileImport => self.prompt_import_media(),
                }
            }
        }
    }

    fn prompt_import_media(&mut self) {
        let mut chooser = NativeFileChooser::new(FileDialogType::BrowseFile);
        match chooser.try_show() {
            Ok(FileDialogAction::Success) => {
                let file = chooser.filename();
                let file_str = file.to_string_lossy();
                info!("import file \"{file_str}\"");

                match ffmpeg_next::format::input(&chooser.filename()) {
                    Ok(mut ictx) => match ictx.streams().best(Type::Video) {
                        Some(input) => {
                            let video_stream_index = input.index();

                            match ffmpeg_next::codec::context::Context::from_parameters(
                                input.parameters(),
                            )
                            .and_then(|c| c.decoder().video())
                            {
                                Ok(mut video_decoder) => {
                                    match ffmpeg_next::software::scaling::context::Context::get(
                                        video_decoder.format(),
                                        video_decoder.width(),
                                        video_decoder.height(),
                                        ffmpeg_next::format::Pixel::RGB24,
                                        video_decoder.width(),
                                        video_decoder.height(),
                                        ffmpeg_next::software::scaling::flag::Flags::BILINEAR,
                                    ) {
                                        Ok(mut scaler_ctx) => {
                                            let mut frame_num: usize = 0;

                                            for (stream, packet) in ictx.packets() {
                                                if stream.index() == video_stream_index {
                                                    if let Err(_err) =
                                                        video_decoder.send_packet(&packet)
                                                    {
                                                        fltk::dialog::alert_default(
                                                            "Failed sending packet from input video stream at index {video_stream_index}:\n{_err}\nin \"{file_str}\"",
                                                        );
                                                        break;
                                                    }

                                                    let mut decoded = Video::empty();
                                                    while video_decoder
                                                        .receive_frame(&mut decoded)
                                                        .is_ok()
                                                    {
                                                        let mut rgb_frame = Video::empty();
                                                        if let Err(_err) =
                                                            scaler_ctx.run(&decoded, &mut rgb_frame)
                                                        {
                                                            fltk::dialog::alert_default(
                                                                "Failed running software rescaling on frame {frame_num} in stream at index {video_stream_index}:\n{_err}\nin \"{file_str}\"",
                                                            );
                                                            break;
                                                        }

                                                        println!(
                                                            "created output frame size {}x{}",
                                                            rgb_frame.width(),
                                                            rgb_frame.height()
                                                        );

                                                        frame_num += 1;
                                                    }

                                                    // receive_and_process_decoded_frames(
                                                    //     &mut decoder,
                                                    // )?;
                                                }
                                            }
                                        }
                                        Err(_err) => fltk::dialog::alert_default(
                                            "Error creating software scaling context for pixel format transformation:\n{_err}\nin \"{file_str}\"",
                                        ),
                                    }
                                }
                                Err(_err) => fltk::dialog::alert_default(
                                    "Error getting codec context for decoding video stream:\n{_err}\nin \"{file_str}\"",
                                ),
                            }
                        }
                        None => {
                            fltk::dialog::alert_default(
                                "Failed to locate video stream!\nin \"{file_str}\"",
                            );
                        }
                    },
                    Err(err) => {
                        fltk::dialog::alert_default("Unable to load file:{err}\nat \"{file_str}\"");
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    ffmpeg_next::init().expect("failed to initialize ffmpeg!");

    MainApp::new().run_loop();
}
