use super::frame_header::FrameHeader;


#[allow(dead_code)]
pub fn choose_operating_point() -> Result<usize,std::io::Error> {
    log::debug!("obu->handlers->choose_operating_point()");
    Ok(0usize)
}

pub fn copy_last_frame_header(last_frame_header: &FrameHeader) -> Result<FrameHeader,std::io::Error> {
    log::debug!("obu->handlers->copy_last_frame_header()");
    Ok(last_frame_header.clone())
}

// 7.4 Decode frame wrapup process
pub fn decode_frame_wrapup() -> Result<(),std::io::Error> {
    log::debug!("obu->handlers->decode_frame_wrapup()");
    Ok(())
}