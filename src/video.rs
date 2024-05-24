use std::path::PathBuf;

use ffmpeg_the_third as ffmpeg;

// enum LoopState {
//     Running,
//     Sleep(u64),
//     Exit,
// }

#[derive(Default)]
pub struct VideoFile {
    path: Option<PathBuf>,
}

impl VideoFile {
    pub fn open_file() -> Self {
        let path = rfd::FileDialog::new().pick_file();

        Self { path }
    }

    pub fn decode_file(&self) -> Result<(), ffmpeg::Error> {
        if let Some(file) = &self.path {
            ffmpeg::init().unwrap();
            log::info!("{:?}", file);

            // --- read the file
            let mut input = ffmpeg::format::input(&file)?;
            let video_stream = input
                .streams()
                .best(ffmpeg::util::media::Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)?;

            let video_stream_index = video_stream.index();

            // --- setup decoder
            let decoder_context =
                ffmpeg::codec::Context::from_parameters(video_stream.parameters())?;
            let mut packet_decoder = decoder_context.decoder().video()?;

            let mut scaler = ffmpeg::software::scaling::Context::get(
                packet_decoder.format(),
                packet_decoder.width(),
                packet_decoder.height(),
                ffmpeg::format::Pixel::RGB24,
                packet_decoder.width(),
                packet_decoder.height(),
                ffmpeg::software::scaling::Flags::BILINEAR,
            )?;

            let mut frame_idx = 0;

            let mut receive_decoded_frame = |decoder: &mut ffmpeg::decoder::Video| {
                let mut decoded = ffmpeg::util::frame::Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = ffmpeg::util::frame::Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;
                    // a function here to process the frame and idx

                    frame_idx += 1;
                }

                Ok::<(), ffmpeg::Error>(())
            };

            // --- decoding happens here i guess?
            while let Some(Ok((stream, packet))) = input.packets().next() {
                // log::info!("{:?}", packet.data());

                if stream.index() == video_stream_index {
                    packet_decoder.send_packet(&packet)?;
                    receive_decoded_frame(&mut packet_decoder)?;
                }
            }

            packet_decoder.send_eof()?;
            receive_decoded_frame(&mut packet_decoder)?;
        }
        Ok(())
    }
}
