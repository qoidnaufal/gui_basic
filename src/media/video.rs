use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use ffmpeg_the_third as ffmpeg;

pub struct VideoStreamData {
    pub video_index: usize,
    pub data: Arc<Mutex<BTreeMap<usize, ffmpeg::util::frame::Video>>>,
}

impl VideoStreamData {
    pub fn new() -> Self {
        Self {
            video_index: 0,
            data: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
    pub fn open_video(&self) -> Result<Option<()>, ffmpeg::Error> {
        let maybe_path = rfd::FileDialog::new().pick_file();

        let video_data = self.data.clone();

        if let Some(path) = maybe_path {
            tokio::spawn(async move {
                ffmpeg::init().unwrap();
                log::info!("{:?}", path);

                // --- read the file from the path
                let mut input = ffmpeg::format::input(&path)?;
                let video_stream = input
                    .streams()
                    .best(ffmpeg::util::media::Type::Video)
                    .ok_or(ffmpeg::Error::StreamNotFound)?;

                let video_stream_index = video_stream.index();

                let decoder_context =
                    ffmpeg::codec::Context::from_parameters(video_stream.parameters())?;
                // --- ffmpeg::codec::decoder::video::Video
                let mut packet_decoder = decoder_context.decoder().video()?;

                let mut scaler = ffmpeg::software::scaling::Context::get(
                    packet_decoder.format(),
                    packet_decoder.width(),
                    packet_decoder.height(),
                    ffmpeg::format::Pixel::RGBA,
                    packet_decoder.width(),
                    packet_decoder.height(),
                    ffmpeg::software::scaling::Flags::BILINEAR,
                )?;

                let mut frame_idx = 0usize;

                let mut receive_decoded_frame =
                    |p_dec: &mut ffmpeg::codec::decoder::video::Video| {
                        let mut decoded = ffmpeg::util::frame::Video::empty();
                        if p_dec.receive_frame(&mut decoded).is_ok() {
                            // --- the output frame
                            let mut rgba_frame = ffmpeg::util::frame::Video::empty();

                            // --- No accelerated colorspace conversion found from yuv420p to rgba.
                            scaler.run(&decoded, &mut rgba_frame)?;

                            // --- register the rgba_fram to avoid vec allocations
                            // --- but, inserting into BTreeMap is still expensive i guess
                            video_data.lock().unwrap().insert(frame_idx, rgba_frame);

                            frame_idx += 1;
                        }

                        Ok::<(), ffmpeg::Error>(())
                    };

                while let Some(Ok((stream, packet))) = input.packets().next() {
                    if stream.index() == video_stream_index {
                        packet_decoder.send_packet(&packet)?;
                        receive_decoded_frame(&mut packet_decoder)?;
                    }
                }

                packet_decoder.send_eof()?;
                receive_decoded_frame(&mut packet_decoder)?;

                Ok::<(), ffmpeg::Error>(())
            });

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}
