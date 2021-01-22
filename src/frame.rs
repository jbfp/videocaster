use anyhow::Error;
use rocket::{
    http::ContentType,
    response::{Content, Debug},
};
use std::env::{self, consts};
use tokio::process::Command;

#[get("/frame?<path>")]
pub(crate) async fn handler(path: String) -> Result<Content<Vec<u8>>, Debug<Error>> {
    let image = extract_jpeg(&path).await?;
    let content_type = ContentType::JPEG;
    let content = Content(content_type, image);
    Ok(content)
}

async fn extract_jpeg(path: &str) -> Result<Vec<u8>, Error> {
    let mut ffmpeg = env::current_dir()?;
    ffmpeg.push("bin");
    ffmpeg.push("ffmpeg");
    ffmpeg.push(consts::OS);
    ffmpeg.push("ffmpeg");
    #[cfg(target_os = "windows")]
    ffmpeg.set_extension("exe");

    debug!("ffmpeg path: {}", ffmpeg.display());
    debug!("video path: {}", path);

    let output = Command::new(ffmpeg)
        .args(&[
            "-ss",          // seek to
            "00:00:30",     // 30 seconds
            "-i",           // set input to
            &path,          // the path of the video
            "-vframes",     // take n video frame
            "1",            // n = 1
            "-q:v",         // set output quality to
            "6",            // medium
            "-nostats",     // hide stats from stdout
            "-hide_banner", // hide banner from stdout
            "-f",           // set output format to
            "mpjpeg",       // jpeg
            "-",            // pipe to stdout
        ])
        .output()
        .await?;

    if output.status.success() {
        let stdout = output.stdout;
        let e = String::from_utf8(stdout).unwrap_err();
        let bytes = e.as_bytes();
        let inner = e.utf8_error();
        let valid = inner.valid_up_to();
        let (_, rest) = bytes.split_at(valid);
        Ok(rest.to_vec())
    } else {
        let stderr = output.stderr;
        let e = String::from_utf8_lossy(&stderr);
        Err(anyhow!(e.to_string()))
    }
}
