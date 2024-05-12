use ffmpeg_the_third as ffmpeg;

#[derive(Default)]
pub struct VideoPlayer {
    pub file_handle: Option<rfd::FileHandle>,
}

impl VideoPlayer {
    pub async fn open_file() -> Self {
        let file_handle = rfd::AsyncFileDialog::new().pick_file().await;

        Self { file_handle }
    }

    pub fn decode_file(&self) {
        if let Some(file) = &self.file_handle {
            ffmpeg::init().unwrap();

            let path = file.path();

            // --- demuxing happens here i guess?
            let mut input_context = ffmpeg::format::input(&path).unwrap();
            let video_stream = input_context
                .streams()
                .best(ffmpeg::util::media::Type::Video)
                .unwrap();
            let video_stream_index = video_stream.index();

            let decoder_context =
                ffmpeg::codec::Context::from_parameters(video_stream.parameters());
            let mut packet_decoder = decoder_context.unwrap().decoder().video().unwrap();

            // --- decoding happens here i guess?
            while let Some(Ok((stream, packet))) = input_context.packets().next() {
                if stream.index() == video_stream_index {
                    packet_decoder.send_packet(&packet).unwrap();
                }

                let mut decoded_frame = ffmpeg::util::frame::Video::empty();
                while packet_decoder.receive_frame(&mut decoded_frame).is_ok() {
                    let mut context = ffmpeg::software::scaling::Context::get(
                        decoded_frame.format(),
                        decoded_frame.width(),
                        decoded_frame.height(),
                        ffmpeg::format::Pixel::RGB24,
                        decoded_frame.width(),
                        decoded_frame.height(),
                        ffmpeg::software::scaling::Flags::BILINEAR,
                    )
                    .unwrap();
                    let mut rgb_frame = ffmpeg::util::frame::Video::empty();
                    context.run(&decoded_frame, &mut rgb_frame).unwrap();
                }
            }
        }
    }
}
