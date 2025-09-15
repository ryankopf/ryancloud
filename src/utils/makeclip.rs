use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

pub fn create_video_clip(
    source: &str,
    start: i64,
    end: i64,
    output_dir: Option<&str>,
) -> Result<PathBuf, String> {
    let ffmpeg_path = std::env::var("FFMPEG_PATH")
        .map_err(|_| "FFMPEG_PATH not defined in environment".to_string())?;

    let duration = end - start;
    if duration <= 0 {
        return Err("Invalid clip duration".to_string());
    }

    let source_path = Path::new(source);

    // Build the filename: originalstem-start-end.mp4
    let filename = format!(
        "{}-{}-{}.mp4",
        source_path.file_stem().unwrap_or_default().to_string_lossy(),
        start,
        end
    );

    // Default clip directory = <source_parent>/clips
    let output_path = if let Some(dir) = output_dir {
        PathBuf::from(dir).join(&filename)
    } else {
        let clip_dir = source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("clips");

        std::fs::create_dir_all(&clip_dir)
            .map_err(|e| format!("Failed to create clips dir: {}", e))?;

        clip_dir.join(filename)
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
