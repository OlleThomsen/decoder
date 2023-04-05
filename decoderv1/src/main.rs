


use image::{ImageBuffer, Rgb};
use std::{error::Error, fs::{File, read_dir}};
use std::io::{Read, Write, Seek};
use chrono::{DateTime, Utc};
use std::process::{Command, Stdio};
use image::io::Reader as ImageReader;
use std::sync::{Arc, Mutex};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use zip::read::ZipArchive;
use std::io::{BufReader, BufWriter};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const SQUARE_W: usize = 4;
const SQUARE_H: usize = 4; 
const FRAME_SIZE: usize = (WIDTH * HEIGHT) / (SQUARE_W * SQUARE_H); //Amount of squares on the screen 


fn main() -> Result<(), Box<dyn std::error::Error>> {
    

    let file = FileEncode {
        input_path: "testfiles/enwik9.zip",
        output_path: "output/enwik9.mp4",
    };

    encoder2(file);
    Ok(())
    //decode_video("output/bible copy1.mp4", "input/bible copy1.txt");
 
    // // Read the client secrets file
    // let secrets_file = "credentials.json";
    // let mut secrets_buf = String::new();
    // let mut secrets_file = File::open(secrets_file)?;
    // secrets_file.read_to_string(&mut secrets_buf)?;
    // let secrets = read_application_secret(&mut secrets_buf.as_bytes())?;

    // // Create the authenticator
    // let auth = Authenticator::new(
    //     &secrets,
    //     DefaultDeviceFlowDelegate,
    //     hyper::Client::builder().build::<_, Body>(HttpsConnector::new()),
    // );

    // // Build the HTTPS connector with the platform's root certificates
    // let mut http_conn = HttpsConnector::new();
    // let native_roots = rustls_native_certs::load_native_certs()?;
    // http_conn.set_certificate_verifier(native_roots);
    // http_conn.set_protocols(&["h2".into(), "http/1.1".into()]);

    // // Create the HTTP client
    // let client = Client::builder().build(http_conn);

    // // Create a request to upload a video to YouTube
    // let request = Request::post("https://www.googleapis.com/upload/youtube/v3/videos")
    //     .header("Content-Type", "application/json")
    //     .header("Authorization", format!("Bearer {}", auth.token().await?.access_token()))
    //     .body(Body::from("{
    //         \"snippet\": {
    //             \"title\": \"My Video\",
    //             \"description\": \"This is a test video uploaded via the YouTube API\",
    //             \"categoryId\": \"22\"
    //         },
    //         \"status\": {
    //             \"privacyStatus\": \"unlisted\"
    //         }
    //     }"))?;

    // // Send the request and get the response
    // let response = client.request(request).await?;

    // println!("{:?}", response);

    // Ok(())
}

struct FileEncode<'a> {
    input_path: &'a str,
    output_path: &'a str,
}

fn encoder1(file: FileEncode) {
    let file_size = std::fs::metadata(&file.input_path)
        .expect("Failed to get input file metadata")
        .len() as usize;

    // Create a progress bar
    let pb = Arc::new(Mutex::new(ProgressBar::new(file_size as u64)));
    pb.lock()
        .unwrap()
        .set_style(
            ProgressStyle::default_bar()
                .template("{elapsed_precise} [{bar:40.cyan/blue}] {percent}%")
                .unwrap(),
        );

    let num_bits = file_size * 8;
    let num_frames = (num_bits + FRAME_SIZE - 1) / FRAME_SIZE;

    let title = &format!("title={}", file.input_path.split('/').last().unwrap());
    let datatype = &format!("author={}", title.split('.').last().unwrap());

    let now: DateTime<Utc> = Utc::now();
    let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // Convert the frames to an MP4 video using FFmpeg
    let mut ffmpeg_process = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate",
            "30",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            &format!("{}x{}", WIDTH, HEIGHT),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-crf",
            "0",
            "-b:v",
            "1000M",
            "-maxrate",
            "1000M",
            "-bufsize",
            "1000M",
            "-movflags",
            "+faststart",
            "-map_metadata",
            "0",
            "-metadata",
            title,
            "-metadata",
            datatype,
            "-metadata",
            &format!("time={}", time_str),
            &file.output_path,
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    // Write the frames to the FFmpeg process
    let mut buf = [0u8; FRAME_SIZE * SQUARE_W * SQUARE_H * 3];
    for frame_index in 0..num_frames {
        let start = frame_index * FRAME_SIZE / 8;
        let end = std::cmp::min(start + FRAME_SIZE / 8, file_size);

        let mut chunk = vec![0u8; end - start];
        let mut input_file = File::open(&file.input_path).unwrap();
        input_file.seek(std::io::SeekFrom::Start(start as u64)).unwrap();
        input_file.read_exact(&mut chunk).unwrap();

        let mut bytes_written = 0usize;
        for (j, byte) in chunk.iter().enumerate() {
            for bit_index in 0..8 {
                let bit = (byte & (1 << bit_index)) != 0;
                let color = if bit {
                    Rgb([255, 255, 255])
                } else {
                    Rgb([0, 0, 0])
                };
                let pixel_x = ((j * 8 * SQUARE_W) % WIDTH + bit_index * SQUARE_W) as u32;
                let pixel_y = (((j * 8 * SQUARE_H) / WIDTH) * SQUARE_H) as u32;
                for y in pixel_y..(pixel_y + SQUARE_H as u32) {
                    for x in pixel_x..(pixel_x + SQUARE_W as u32) {
                        let offset = (y as usize * WIDTH as usize + x as usize) * 3;
                        buf[offset..offset+3].copy_from_slice(&color.0);
                    }
                }
            }
            bytes_written += 8 * SQUARE_W * SQUARE_H * 3;
        }
        ffmpeg_process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(&buf[..bytes_written])
            .unwrap();
        
        pb.lock().unwrap().inc(chunk.len() as u64);
    }
    // Wait for the FFmpeg process to finish
    ffmpeg_process.wait().unwrap();
}

fn encoder2(file: FileEncode) {

    let file_size = std::fs::metadata(&file.input_path)
    .expect("Failed to get input file metadata")
    .len() as usize;

    // Create a progress bar
    let pb = Arc::new(Mutex::new(ProgressBar::new(file_size as u64)));
    pb.lock()
        .unwrap()
        .set_style(ProgressStyle::default_bar().template("{elapsed_precise} [{bar:40.cyan/blue}] {percent}% {bytes}/{total_bytes}  ({eta})").unwrap());

    let num_bits = file_size * 8;
    let num_frames = (num_bits + FRAME_SIZE - 1) / FRAME_SIZE;

    let title = &format!("title={}", file.input_path.split('/').last().unwrap());
    let datatype = &format!("author={}", title.split('.').last().unwrap());

    let now: DateTime<Utc> = Utc::now();
    let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // Convert the frames to an MP4 video using FFmpeg
    let ffmpeg = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", "30",
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "-s", &format!("{}x{}", WIDTH, HEIGHT),
            "-i", "-",
            "-c:v", "libx264",
            "-crf", "18",
            "-preset", "ultrafast",
            "-b:v", "1000M",
            "-maxrate", "1000M",
            "-bufsize", "1000M",
            "-movflags", "+faststart",
            "-map_metadata", "0",
            "-metadata",  title,
            "-metadata", datatype,
            "-metadata", &format!("time={}", time_str),
            &file.output_path,
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
        let ffmpeg_process = Arc::new(Mutex::new(ffmpeg));

        (0..num_frames).into_par_iter().for_each(|frame_index| {
            let mut frame = ImageBuffer::<Rgb<u8>, _>::new(WIDTH as u32, HEIGHT as u32);

            let start = frame_index * FRAME_SIZE / 8;
            let end = std::cmp::min(start + FRAME_SIZE / 8, file_size);

            let mut chunk = vec![0u8; end - start];
            let mut input_file = File::open(&file.input_path).unwrap();
            input_file.seek(std::io::SeekFrom::Start(start as u64)).unwrap();
            input_file.read_exact(&mut chunk).unwrap();

            for (j, byte) in chunk.iter().enumerate() {
                for bit_index in 0..8 {
                    let bit = (byte & (1 << bit_index)) != 0;
                    let color = if bit { Rgb([255, 255, 255]) } else { Rgb([0, 0, 0]) };
                    let pixel_x = ((j * 8 * SQUARE_W) % WIDTH + bit_index * SQUARE_W) as u32;
                    let pixel_y = (((j * 8 * SQUARE_H) / WIDTH ) * SQUARE_H) as u32;
                    for y in pixel_y..(pixel_y + SQUARE_H as u32) {
                        for x in pixel_x..(pixel_x + SQUARE_W as u32) {
                            frame.put_pixel(x, y, color);
                        }
                    }  
                }
            }

            {
                let mut process = ffmpeg_process.lock().unwrap();
                process.stdin.as_mut().unwrap().write_all(&frame.into_raw()).unwrap();
            }
            pb.lock().unwrap().inc(chunk.len() as u64);

            // Release used memory
            drop(chunk);
            drop(input_file);

        });

}

fn decode_video(input_path: &str, output_path: &str) {
    // Use FFmpeg to read the video frames as raw RGB data
    let output = Command::new("ffmpeg")
        .args(&[
            "-i",
            input_path,
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-lossless",
            "1",
            "-",
        ])
        .output()
        .expect("Failed to execute FFmpeg command");

        let mut index = 0;
        let mut bytes = Vec::new();
        while index < output.stdout.len() {
            let mut frame_bytes = output.stdout[index..(index + WIDTH * HEIGHT * 3)].to_vec();
            // Convert the pixels to binary values (1 for white, 0 for black)
            for i in (0..frame_bytes.len()).step_by(3) {
                let r = frame_bytes[i];
                let g = frame_bytes[i + 1];
                let b = frame_bytes[i + 2];
                let val = if (r as u32 + g as u32 + b as u32) / 3 > 63 {
                    1u8
                } else {
                    0u8
                };
                frame_bytes[i / 3] = val;
            }
            bytes.extend_from_slice(&frame_bytes);
            index += WIDTH * HEIGHT * 3;
        }

    // Convert the bits to bytes
    let mut byte_buffer = Vec::new();
    let mut bit_buffer = 0u8;
    let mut bit_count = 0;
    for bit in bytes {
        bit_buffer |= bit << bit_count;
        bit_count += 1;
        if bit_count == 8 {
            byte_buffer.push(bit_buffer);
            bit_buffer = 0;
            bit_count = 0;
        }
    }

    // Write the decoded bytes to a text file
    let file = File::create(output_path).expect("Failed to create output file");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(&byte_buffer)
        .expect("Failed to write to output file");
}

fn unzip_file(zip_file_path: &str, output_folder: &str) -> std::io::Result<()> {
    let mut archive = ZipArchive::new(File::open(zip_file_path)?)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_folder.to_owned() + "/" + path.to_str().unwrap(),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = std::path::Path::new(&outpath).parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

