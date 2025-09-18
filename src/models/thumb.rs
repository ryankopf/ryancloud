use std::process::Command;

pub struct Thumb {

}

impl Thumb {
    pub fn generate(input: &str, output: &str, ffmpeg_path: &str) {
        // Construct the ffmpeg command using the provided path
        let command = Command::new(ffmpeg_path)
            .args(["-i", input, "-vf", "thumbnail,scale=320:240", "-frames:v", "1", output])
            .spawn();

        // Log the command execution status
        match command {
            Ok(_) => println!("Thumbnail generation command executed successfully."),
            Err(e) => eprintln!("Failed to execute thumbnail generation command: {}", e),
        }
    }
}