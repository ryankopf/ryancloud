use std::process::{Command, Stdio};

pub fn create_video_clip(
    source: &str,
    start: i64,
    end: i64,
    output_dir: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    let ffmpeg_path = std::env::var("FFMPEG_PATH")
        .map_err(|_| "FFMPEG_PATH not defined in environment".to_string())?;

    let duration = end - start;
    if duration <= 0 {
        return Err("Invalid clip duration".to_string());
    }

    let filename = format!(
        "{}-{}-{}.mp4",
        std::path::PathBuf::from(source)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy(),
        start,
        end
    );

    let output_path = if let Some(dir) = output_dir {
        std::path::PathBuf::from(dir).join(&filename)
    } else {
        std::path::PathBuf::from(&filename)
    };

    let args = vec![
        "-i".to_string(),
        source.to_string(),
        "-ss".to_string(),
        start.to_string(),
        "-t".to_string(),
        duration.to_string(),
        "-c".to_string(),
        "copy".to_string(),
        output_path.to_string_lossy().to_string(),
    ];

    // Debug print of full command
    println!("Running command: {} {}", ffmpeg_path, args.join(" "));

    Command::new(&ffmpeg_path)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    Ok(output_path)
}
