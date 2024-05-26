use ffmpeg_the_third as ffmpeg;

// enum LoopState {
//     Running,
//     Sleep(u64),
//     Exit,
// }

#[derive(Default)]
pub struct VideoStreamer {
    pub size: (u32, u32),
    // instead of binding the frame like this,
    // i think i should just feed this to the pipeline?
    pub frame: Vec<u8>,
}

impl VideoStreamer {
    pub fn open_file() -> Result<Option<Self>, ffmpeg::Error> {
        let maybe_path = rfd::FileDialog::new().pick_file();

        if let Some(path) = maybe_path {
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
                ffmpeg::format::Pixel::RGB24,
                packet_decoder.width(),
                packet_decoder.height(),
                ffmpeg::software::scaling::Flags::BILINEAR,
            )?;

            let mut frame_idx = 0;

            // --- output frame
            let mut rgb_frame = ffmpeg::util::frame::Video::empty();

            let mut receive_decoded_frame = |p_dec: &mut ffmpeg::codec::decoder::video::Video| {
                let mut decoded = ffmpeg::util::frame::Video::empty();
                while p_dec.receive_frame(&mut decoded).is_ok() {
                    scaler.run(&decoded, &mut rgb_frame)?;
                    // --- a function here to process the frame and idx

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

            let size = (rgb_frame.width(), rgb_frame.height());
            let frame = rgb_frame.data(0).to_vec();

            let video = Some(Self { size, frame });

            Ok(video)
        } else {
            Ok(None)
        }
    }

    fn _video_frame_to_image(&self) {}
}
