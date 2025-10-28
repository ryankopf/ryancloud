use std::process::{Command, Stdio};

pub struct Thumb {
}

impl Thumb {
    pub fn generate(input: &str, output: &str, ffmpeg_path: &str) -> Result<(), String> {
        // Construct the ffmpeg command using the provided path
        let command_args = ["-i", input, "-vf", "thumbnail,scale=320:180", "-frames:v", "1", output];
        println!("Executing command: {} {:?}", ffmpeg_path, command_args);

        let status = Command::new(ffmpeg_path)
            .args(&command_args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

        if !status.success() {
            return Err(format!("ffmpeg failed with exit code: {}", status.code().unwrap_or(-1)));
        }

        println!("Thumbnail generation command executed successfully.");
        Ok(())
    }
}