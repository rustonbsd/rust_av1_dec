use mp4parse::{read_mp4, ParseStrictness};
use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::Path;

fn main() -> io::Result<()> {
    // Path to your MP4 file
    let input_path = "./samples/Sintel_1080_10s_10MB.mp4";
    // Path where you want to save the raw AV1 stream
    let output_path = "./samples/output_av1_stream.bin";
    
    // Extract AV1 byte stream
    let av1_data = extract_av1_from_mp4(input_path)?;
    
    // Write to output file
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&av1_data)?;
    
    println!("Extracted {} bytes of AV1 data to {}", av1_data.len(), output_path);
    Ok(())
}

fn extract_av1_from_mp4<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    // Open the MP4 file
    let mut file = File::open(path)?;
    
    // Parse the MP4 file
    let context = read_mp4(&mut file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse MP4: {:?}", e)))?;
    
    // Find the AV1 track and extract its sample data
    let av1_track = context.tracks.iter()
        .find(|track| track.track_type == mp4parse::TrackType::Video)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No video track found in MP4"))?;
    
    // We need to use the unstable API to get sample data
    let indices = mp4parse::unstable::create_sample_table(av1_track, 0.into())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Failed to create sample table"))?;
    
    // Collect all sample data
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
    
    // Check if we have any data
    if av1_data.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No sample data found in AV1 track"));
    }
    
    // Return a copy of the data
    Ok(av1_data.to_vec())
}
