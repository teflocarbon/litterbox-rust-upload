use reqwest::multipart;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use humansize::{format_size, BINARY};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::env::args()
        .nth(1)
        .expect("Please provide a file path");
    
    let file = File::open(&file_path).await?;
    let file_size = file.metadata().await?.len();
    
    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));

    // Read file first
    let mut buffer = Vec::with_capacity(file_size as usize);
    let mut file = File::open(&file_path).await?;
    file.read_to_end(&mut buffer).await?;

    let filename = PathBuf::from(&file_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let form = multipart::Form::new()
        .text("reqtype", "fileupload")
        .text("time", "1h")
        .part("fileToUpload", multipart::Part::bytes(buffer)
            .file_name(filename));

    // Start timing here for upload speed
    let start = Instant::now();
    
    let client = reqwest::Client::new();
    let response = client.post("https://litterbox.catbox.moe/resources/internals/api.php")
        .multipart(form)
        .send()
        .await?;

    // Calculate upload speed
    let duration = start.elapsed();
    let speed = format_size((file_size as f64 / duration.as_secs_f64()) as u64, BINARY);
    pb.set_position(file_size);

    if response.status().is_success() {
        println!("\nUpload speed: {}/s", speed);
        println!("Upload successful: {}", response.text().await?);
    } else {
        eprintln!("Upload failed: {}", response.status());
    }

    pb.finish_and_clear();
    Ok(())
}