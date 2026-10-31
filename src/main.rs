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

fn one_pass(input:&str, bitrate:f64) {
    let max = bitrate + (bitrate/8.0);
    let buf = bitrate * 2.0;

    let bits = format!("{:.0}k", bitrate / 1000.0);
    let maxbits = format!("{:.0}k", max / 1000.0);
    let bufsize = format!("{:.0}k", buf / 1000.0);

    println!("Bits: {}, maxbits: {}, bufsize: {}", bits, maxbits, bufsize);
    let status = match Command::new("ffmpeg")
        .args([
            "-y", "-i", &input,
            "-c:v", "h264_nvenc",
            "-preset", "p4",
            "-b:v", &bits,
            "-maxrate", &maxbits,
            "-bufsize", &bufsize,
            "-c:a", "aac",
            "-b:a", "128k",
            "output1.mp4",
        ]).status() {
            Ok(status) => status,
            Err(error) => {
                eprintln!("Failed 1 pass: {error}");
                return;
            }
        };
}

fn two_pass(input:&str, bitrate:f64) {
    println!("2pass");
}

fn get_info(input:&str) -> Result<String, String> {
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
    let output = get_info(&args.input)?;
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
    match args.encode.as_str() {
        "fast" => {
            one_pass(&args.input, video_bitrate);
        },
        "exact" => {
            two_pass(&args.input, video_bitrate);
        },
        &_ => todo!(),
    }
    //  -> Fast : Do one pass encoding
    //  -> Exact : Do two pass encoding

    Ok(())

}
