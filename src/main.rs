use clap::Parser;
use std::path::Path;
use std::process::Command;

#[derive(Parser, Debug)]
struct Args {
    input: String,
    #[arg(short, long)]
    size: String,
    #[arg(short, long, default_value_t = "fast".to_string())]
    encode: String,
}

fn get_info(input:String) -> Result<String, String> {
    let output = match Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            &input,
        ]).output() {
            Ok(output) => output,
            Err(error) => {
                eprintln!("Could not run ffprobe: {error}");
                return Err("ffprobe failed".to_string());
            }
        };

    if !output.status.success() {
        return Err("ffprobe failed".to_string());
    }

    let duration = match String::from_utf8(output.stdout) {
        Ok(duration) => duration,
        Err(error) => {
            eprintln!("Could not convert utf8 to String: {error}");
            return Err("string convertion failed".to_string());
        }
    };
    Ok(duration.trim().to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get arguments
    let args = Args::parse();
    println!("Input: {}, Size: {}, Encoding: {}", args.input, args.size, args.encode);

    //Check if input file exists
    assert!(Path::new(&args.input).exists(), "File given doesn't exist");

    // Get media duration with ffprobe
    let output = get_info(args.input)?;
    let duration: f64 = output.parse()?;

    println!("Duration: {} seconds", duration);

    // Convert the target size into bits
    let target_size: f64 = args.size.trim().parse()?;
    let target_bits = (target_size * 0.96) * 8.0 * 1024.0 * 1024.0;

    // Determine total bitrate
    let bps = target_bits / duration;

    // Reserve bandwidth for the audio
    let video_bitrate = bps - 128000.0;


    println!("Target bits: {}bps, Total Bitrate: {}bps, Video Bitrate: {}bps", target_bits, bps, video_bitrate);

    // Check if its fast or exact
    //  -> Fast : Do one pass encoding
    //  -> Exact : Do two pass encoding

    Ok(())

}
