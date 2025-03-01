pub mod video_player;

use crate::project::{
    AudioMediaStream, BasicStreamInfo, MediaInfo, MediaLength, MediaStream, VideoMediaStream,
};
use anyhow::Context;
use std::{
    path::{Path, PathBuf},
    thread::JoinHandle,
};

pub fn load_media_sync(path: PathBuf) -> anyhow::Result<MediaInfo> {
    let input_ctx = ffmpeg_next::format::input(&path)
        .context("failed to get input format context for file from ffmpeg")?;
    let mut streams = vec![];

    for stream in input_ctx.streams() {
        let index = stream.index();
        let param = stream.parameters();
        let length = MediaLength {
            time_base_length: stream.duration() as u64,
            time_base: stream.time_base().into(),
        };

        streams.push(match param.medium() {
            ffmpeg_next::media::Type::Video => {
                MediaStream::Video(BasicStreamInfo { index, length }, VideoMediaStream {})
            }
            ffmpeg_next::media::Type::Audio => {
                MediaStream::Audio(BasicStreamInfo { index, length }, AudioMediaStream {})
            }
            _ => {
                log::warn!("unknown medium for stream {}", stream.index());
                continue;
            }
        });
    }

    Ok(MediaInfo { path, streams })
}

fn frames_from_packet() -> Vec<()> {
    vec![]
}
