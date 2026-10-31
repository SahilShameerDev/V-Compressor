use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
struct Args {
    input: String,
    #[arg(short, long)]
    size: String,
    #[arg(short, long, default_value_t = "fast".to_string())]
    encode: String,
}

fn main() {
    // Get arguments
    let args = Args::parse();
    println!("Input: {}, Size: {}, Encoding: {}", args.input, args.size, args.encode);

    //Check if input file exists
    assert!(Path::new(&args.input).exists(), "File given doesn't exist");

    // Get media duration with ffprobe

    // Convert the target size into bits

    // Determine total bitrate
    //
    // Reserve bandwidth for the audio

    // Check if its fast or exact
    //  -> Fast : Do one pass encoding
    //  -> Exact : Do two pass encoding

}
