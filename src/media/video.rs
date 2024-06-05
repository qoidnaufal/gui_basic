// use std::sync::{Arc, Mutex};

use ffmpeg_the_third as ffmpeg;

pub struct VideoStream {
    pub video_stream_index: usize,
    pub frame_rate: f64,
    pub time_base: f64,
    pub duration_ms: i64,
    pub video_decoder: ffmpeg::codec::decoder::video::Video,
    pub input_context: ffmpeg::format::context::input::Input,
}

impl VideoStream {
    pub fn open_video() -> anyhow::Result<Option<Self>> {
        let maybe_path = rfd::FileDialog::new().pick_file();

        if let Some(path) = maybe_path {
            log::info!("{:?}", path);

            let input_context = ffmpeg::format::input(&path)?;

            let video_stream = input_context
                .streams()
                .best(ffmpeg::util::media::Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)?;

            let time_base = video_stream.time_base().numerator() as f64
                / video_stream.time_base().denominator() as f64;

            let duration_ms = (video_stream.duration() as f64 * time_base * 1000.) as i64;

            let frame_rate = (video_stream.avg_frame_rate().numerator() as f64)
                / video_stream.avg_frame_rate().denominator() as f64;

            let decoder_context =
                ffmpeg::codec::Context::from_parameters(video_stream.parameters())?;

            let video_decoder = decoder_context.decoder().video()?;

            Ok(Some(Self {
                duration_ms,
                time_base,
                video_stream_index: video_stream.index(),
                frame_rate,
                video_decoder,
                input_context,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_next_packet(&mut self) -> Option<(ffmpeg::Stream, ffmpeg::Packet)> {
        self.input_context
            .packets()
            .next()
            .filter(|iter_result| iter_result.is_ok())
            .map(|p| p.unwrap())
    }
}
