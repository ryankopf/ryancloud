use std::process::Command;

pub struct Thumb {
}

impl Thumb {
    pub fn generate(input: &str, output: &str, ffmpeg_path: &str) {
        // Construct the ffmpeg command using the provided path
        let command_args = ["-i", input, "-vf", "thumbnail,scale=320:180", "-frames:v", "1", output];
        println!("Executing command: {} {:?}", ffmpeg_path, command_args);

        let command = Command::new(ffmpeg_path)
            .args(&command_args)
            .spawn();

        // Log the command execution status
        match command {
            Ok(_) => println!("Thumbnail generation command executed successfully."),
            Err(e) => eprintln!("Failed to execute thumbnail generation command: {}", e),
        }
    }
}