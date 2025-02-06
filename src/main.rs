mod project;
mod ui;

use anyhow::Context;
use fltk::{app, enums::Color, prelude::*, window::Window};

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Oxy);

    let mut ui = ui::UserInterface::make_window();
    let mut main_window = ui.main_window.clone();

    main_window.set_label(concat!(
        env!("CARGO_CRATE_NAME"),
        " v",
        env!("CARGO_PKG_VERSION")
    ));
    main_window.clone().center_screen();

    let mut subwindow = Window::new(0, 0, 100, 100, None);
    subwindow.set_color(Color::Black);
    subwindow.end();
    ui.preview_group.add(&subwindow);
    subwindow.clone().size_of_parent();
    subwindow.clone().center_of_parent();
    subwindow.show();

    main_window.show();

    let state = futures_lite::future::block_on(ui::State::new(subwindow));
    while app.wait() {
        let frame = state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&state.render_pipeline);
            rpass.draw(0..3, 0..1);
        }
        state.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
