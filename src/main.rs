mod project;
mod ui;

use env_logger::Env;
use ffmpeg_next::{Codec, Rational, codec::Context, media::Type};
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
                info!("import file \"{}\"", chooser.filename().to_string_lossy());

                match ffmpeg_next::format::input(&chooser.filename()) {
                    Ok(context) => {
                        for (k, v) in context.metadata().iter() {
                            println!("{}: {}", k, v);
                        }

                        if let Some(stream) =
                            context.streams().best(ffmpeg_next::media::Type::Video)
                        {
                            println!("Best video stream index: {}", stream.index());
                        }

                        if let Some(stream) =
                            context.streams().best(ffmpeg_next::media::Type::Audio)
                        {
                            println!("Best audio stream index: {}", stream.index());
                        }

                        if let Some(stream) =
                            context.streams().best(ffmpeg_next::media::Type::Subtitle)
                        {
                            println!("Best subtitle stream index: {}", stream.index());
                        }

                        println!(
                            "duration (seconds): {:.2}",
                            context.duration() as f64 / f64::from(ffmpeg_next::ffi::AV_TIME_BASE)
                        );

                        for stream in context.streams() {
                            println!("stream index {}:", stream.index());
                            println!("\ttime_base: {}", stream.time_base());
                            println!("\tstart_time: {}", stream.start_time());
                            println!("\tduration (stream timebase): {}", stream.duration());
                            println!(
                                "\tduration (seconds): {:.2}",
                                stream.duration() as f64 * f64::from(stream.time_base())
                            );
                            println!("\tframes: {}", stream.frames());
                            println!("\tdisposition: {:?}", stream.disposition());
                            println!("\tdiscard: {:?}", stream.discard());
                            println!("\trate: {}", stream.rate());

                            let codec = ffmpeg_next::codec::context::Context::from_parameters(
                                stream.parameters(),
                            )
                            .expect("failed to get codec context");
                            println!("\tmedium: {:?}", codec.medium());
                            println!("\tid: {:?}", codec.id());

                            if codec.medium() == ffmpeg_next::media::Type::Video {
                                if let Ok(video) = codec.decoder().video() {
                                    println!("\tbit_rate: {}", video.bit_rate());
                                    println!("\tmax_rate: {}", video.max_bit_rate());
                                    println!("\tdelay: {}", video.delay());
                                    println!("\tvideo.width: {}", video.width());
                                    println!("\tvideo.height: {}", video.height());
                                    println!("\tvideo.format: {:?}", video.format());
                                    println!("\tvideo.has_b_frames: {}", video.has_b_frames());
                                    println!("\tvideo.aspect_ratio: {}", video.aspect_ratio());
                                    println!("\tvideo.color_space: {:?}", video.color_space());
                                    println!("\tvideo.color_range: {:?}", video.color_range());
                                    println!(
                                        "\tvideo.color_primaries: {:?}",
                                        video.color_primaries()
                                    );
                                    println!(
                                        "\tvideo.color_transfer_characteristic: {:?}",
                                        video.color_transfer_characteristic()
                                    );
                                    println!(
                                        "\tvideo.chroma_location: {:?}",
                                        video.chroma_location()
                                    );
                                    println!("\tvideo.references: {}", video.references());
                                    println!(
                                        "\tvideo.intra_dc_precision: {}",
                                        video.intra_dc_precision()
                                    );
                                }
                            } else if codec.medium() == ffmpeg_next::media::Type::Audio {
                                if let Ok(audio) = codec.decoder().audio() {
                                    println!("\tbit_rate: {}", audio.bit_rate());
                                    println!("\tmax_rate: {}", audio.max_bit_rate());
                                    println!("\tdelay: {}", audio.delay());
                                    println!("\taudio.rate: {}", audio.rate());
                                    println!("\taudio.channels: {}", audio.channels());
                                    println!("\taudio.format: {:?}", audio.format());
                                    println!("\taudio.frames: {}", audio.frames());
                                    println!("\taudio.align: {}", audio.align());
                                    println!(
                                        "\taudio.channel_layout: {:?}",
                                        audio.channel_layout()
                                    );
                                }
                            }
                        }
                    }
                    Err(err) => error!("failed to open file with ffmpeg: {err:?}"),
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
