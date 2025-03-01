use anyhow::Context;
use ffmpeg_next::{codec, format, frame, software::scaling};
use log::info;

pub struct FfmpegVideoDecoder {
    stream_index: usize,
    input_ctx: format::context::Input,

    video_decoder: codec::decoder::Video,
    scaler_ctx: scaling::Context,
}

#[allow(unused)]
impl FfmpegVideoDecoder {
    pub fn new(mut input_ctx: format::context::Input, stream_index: usize) -> anyhow::Result<Self> {
        let video_stream = input_ctx
            .stream(stream_index)
            .with_context(|| format!("failed to locate stream at index {stream_index}"))?;
        info!("got video stream at index {stream_index}");

        let video_decoder = codec::context::Context::from_parameters(video_stream.parameters())
            .context("failed to create video decoder")
            .and_then(|c| {
                c.decoder()
                    .video()
                    .context("failed to get video from decoder context")
            })?;
        info!("created video decoder");

        let scaler_ctx = scaling::context::Context::get(
            video_decoder.format(),
            video_decoder.width(),
            video_decoder.height(),
            format::Pixel::RGBA,
            video_decoder.width(),
            video_decoder.height(),
            scaling::flag::Flags::BILINEAR,
        )
        .context("failed to create software scaler for pixel reformatting")?;
        info!("created software scaler");

        let packet_iter = input_ctx.packets();

        Ok(Self {
            stream_index,
            input_ctx,
            video_decoder,
            scaler_ctx,
        })
    }

    pub fn receive_frames_from_packet(&mut self) -> anyhow::Result<Vec<frame::Video>> {
        let mut packet_iter = self.input_ctx.packets();
        let mut output = vec![];

        info!("receiving frame(s)");

        loop {
            match packet_iter.next() {
                Some((_stream, packet)) if _stream.index() == self.stream_index => {
                    self.video_decoder
                        .send_packet(&packet)
                        .context("failed to send packet from input to video decoder")?;

                    let mut decoded_frame = frame::Video::empty();
                    while self.video_decoder.receive_frame(&mut decoded_frame).is_ok() {
                        let mut rgba_frame = frame::Video::empty();
                        self.scaler_ctx
                            .run(&decoded_frame, &mut rgba_frame)
                            .context(
                                "failed to convert decoded video frame to rgb8 pixel format",
                            )?;

                        output.push(rgba_frame);
                    }

                    info!("read {} frames", output.len());

                    return Ok(output);
                }
                None => return Ok(output),
                _ => {}
            };
        }
    }
}
