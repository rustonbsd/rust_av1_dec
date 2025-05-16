use mp4parse::read_mp4;
use std::fs::File;
use std::io::{self, Read, Write, Seek};
use std::path::Path;

fn main() -> io::Result<()> {
    let input_path = "./samples/Sintel_1080_10s_10MB.mp4";
    let output_path = "./samples/output_av1_stream.bin";
    
    let av1_data = extract_av1_from_mp4(input_path)?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&av1_data)?;
    
    println!("Extracted {} bytes of AV1 data to {}", av1_data.len(), output_path);
    Ok(())
}

fn extract_av1_from_mp4<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    
    let context = read_mp4(&mut file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse MP4: {:?}", e)))?;
    
    let av1_track = context.tracks.iter()
        .find(|track| track.track_type == mp4parse::TrackType::Video)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No video track found in MP4"))?;
    
    let indices = mp4parse::unstable::create_sample_table(av1_track, 0.into())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Failed to create sample table"))?;
    
    let mut av1_data = Vec::new();
    file.seek(io::SeekFrom::Start(0))?;
    
    for indice in indices {
        let start = indice.start_offset.0;
        let end = indice.end_offset.0;
        let sample_size = (end - start) as usize;
        
        file.seek(io::SeekFrom::Start(start))?;
        let mut sample = vec![0; sample_size];
        file.read_exact(&mut sample)?;
        av1_data.extend_from_slice(&sample);
    }
    
    if av1_data.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No sample data found in AV1 track"));
    }
    
    Ok(av1_data.to_vec())
}
