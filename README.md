# v-compress

`v-compress` is a small Rust command-line program that uses FFmpeg's NVIDIA
hardware H.264 encoder (`h264_nvenc`) to encode a video toward a requested
output file size. It probes the input duration, estimates a video bitrate from
the target size, reserves bitrate for audio, and then invokes FFmpeg.

## Requirements

- Rust and Cargo (edition 2024; install with [rustup](https://rustup.rs/)).
- `ffmpeg` and `ffprobe` available on your `PATH`.
- An NVIDIA GPU and an FFmpeg build with the `h264_nvenc` encoder enabled.
  FFmpeg must also support the selected NVENC options (`-preset p4`; the
  `exact` mode additionally uses `-rc vbr` and `-multipass fullres`).
- Enough free disk space for the encoded output.

Check the external tools with:

```sh
ffmpeg -version
ffprobe -version
ffmpeg -encoders | grep h264_nvenc
```

The final command is for Unix-like shells; on other platforms, inspect the
FFmpeg encoder list using the platform's usual search tool.

## Build

From the project directory:

```sh
cargo build --release
```

The executable is created at `target/release/v-compress` (Windows adds `.exe`).
You can also run it directly during development with `cargo run -- ...`.

## Usage

```text
v-compress <INPUT> --size <SIZE> [--encode <MODE>]
```

Arguments:

| Argument | Required | Description |
| --- | --- | --- |
| `<INPUT>` | Yes | Path to the input media file. |
| `-s, --size <SIZE>` | Yes | Desired output size, expressed as a number of MiB (the program uses 1024 × 1024 bytes per MiB). |
| `-e, --encode <MODE>` | No | Encoding mode: `fast` (default) or `exact`. |

Examples:

```sh
# Compress input.mp4 toward 25 MiB using the default fast mode
cargo run --release -- input.mp4 --size 25

# Use the multipass NVENC mode instead
cargo run --release -- input.mp4 --size 25 --encode exact

# If the release executable is on your PATH/current location
./target/release/v-compress input.mp4 --size 25 --encode fast
```

## What the program does

1. Parses the input path, target size, and encoding mode.
2. Checks that the input path exists.
3. Runs `ffprobe` to read the media duration in seconds.
4. Estimates the total bitrate using 96% of the requested size, allowing some
   headroom for container overhead:

   ```text
   target bits = size in MiB × 0.96 × 8 × 1024 × 1024
   total bitrate = target bits ÷ duration
   video bitrate = total bitrate − 128,000 bits/second
   ```

   The remaining 128 kbit/s is reserved for audio. The encoder is configured
   with AAC audio at 128 kbit/s.
5. Runs FFmpeg with H.264 NVENC (`-preset p4`) and prints the calculated
   duration and bitrate values.

### Encoding modes and output files

- **`fast`**: one FFmpeg encode using the calculated video bitrate, a maximum
  rate 12.5% above it, and a buffer twice the target video bitrate. Writes
  `output1.mp4`.
- **`exact`**: uses NVENC VBR rate control and full-resolution multipass
  encoding with the same bitrate, maximum-rate, and buffer calculations.
  Writes `output2.mp4`.

Both output names are fixed and are written to the current working directory;
existing files with those names are overwritten. The requested size is an
estimate, not a guarantee: bitrate allocation, audio, and container overhead
can make the actual file differ from the target. The `exact` mode is a
multipass quality/rate-control option, not a guarantee of an exact byte size.

## Troubleshooting and limitations

- **`ffmpeg` or `ffprobe` not found:** install FFmpeg and ensure both programs
  are on `PATH`.
- **`h264_nvenc` unavailable or encoder initialization fails:** use an
  FFmpeg build with NVENC support, a compatible NVIDIA driver/GPU, and working
  GPU access.
- **Invalid mode:** only `fast` and `exact` are implemented. The current
  program does not provide a custom output path or configurable audio bitrate.
- **Unrealistically small target or short input:** the bitrate calculation can
  result in an unusably low or negative video bitrate; choose a larger target
  size. The program currently does not validate this case explicitly.
- Input media must be readable by the installed FFmpeg. The output container
  is MP4 and the video codec is H.264.

## Project structure

```text
src/main.rs   CLI parsing, bitrate calculation, ffprobe call, and FFmpeg runs
Cargo.toml    Rust package metadata and clap dependency
Cargo.lock    Locked dependency versions
```
