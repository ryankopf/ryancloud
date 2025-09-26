use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

/// Creates a video clip centered around a point (3 seconds before and after)
/// Returns the output path on success, or an error string on failure
pub fn create_point_video(
	source: &str,
	point_time: i64, // Milliseconds
	output_path: &str, // Full output file path
) -> Result<PathBuf, String> {
	let ffmpeg_path = std::env::var("FFMPEG_PATH")
		.map_err(|_| "FFMPEG_PATH not defined in environment".to_string())?;

	let start = point_time - 3000;
	let end = point_time + 3000;
	let duration = end - start;
	if duration <= 0 {
		return Err("Invalid clip duration".to_string());
	}

	let output_path = Path::new(output_path);

	// Ensure the parent directory exists
	if let Some(parent) = output_path.parent() {
		if !parent.exists() {
			std::fs::create_dir_all(parent)
				.map_err(|e| format!("Failed to create output directory: {}", e))?;
		}
	}

	let args = vec![
		"-ss".to_string(),
		format!("{:.3}", start as f64 / 1000.0),
		"-i".to_string(),
		source.to_string(),
		"-t".to_string(),
		format!("{:.3}", duration as f64 / 1000.0),
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

	Ok(output_path.to_path_buf())
}
