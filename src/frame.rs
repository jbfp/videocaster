use anyhow::Error;
use rocket::{
    http::ContentType,
    response::{content::Custom, Debug},
};
use tokio::process::Command;

#[get("/frame?<path>")]
pub(crate) async fn handler(path: String) -> Result<Custom<Vec<u8>>, Debug<Error>> {
    let image = extract_jpeg(&path).await?;
    let content_type = ContentType::JPEG;
    let content = Custom(content_type, image);
    Ok(content)
}

async fn extract_jpeg(path: &str) -> Result<Vec<u8>, Error> {
    let args = [
        "-ss",          // seek to
        "00:00:30",     // 30 seconds
        "-i",           // set input to
        path,           // the path of the video
        "-vframes",     // take n video frame
        "1",            // n = 1
        "-q:v",         // set output quality to
        "6",            // medium
        "-nostats",     // hide stats from stdout
        "-hide_banner", // hide banner from stdout
        "-f",           // set output format to
        "mpjpeg",       // jpeg
        "-",            // pipe to stdout
    ];

    debug!("ffmpeg args: {:#?}", args);

    let output = create_command().args(&args).output().await?;

    if output.status.success() {
        if let Err(e) = String::from_utf8(output.stdout) {
            // skip ffmpeg metadata (content-type, content-length)
            let bytes = e.as_bytes();
            let inner = e.utf8_error();
            let valid = inner.valid_up_to();
            let (header, rest) = bytes.split_at(valid);
            let header = String::from_utf8_lossy(header);
            debug!("skipped output from ffmpeg: \"{}\"", header);
            Ok(rest.to_vec())
        } else {
            debug!("no image data was returned, maybe input is not long enough");
            Ok(vec![])
        }
    } else {
        // unknown error occurred
        let stderr = output.stderr;
        let e = String::from_utf8_lossy(&stderr);
        Err(anyhow!(e.to_string()))
    }
}

#[cfg(target_os = "windows")]
fn create_command() -> Command {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut command = Command::new("ffmpeg.exe");
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
fn create_command() -> Command {
    Command::new("ffmpeg")
}
