use anyhow::Error;
use rocket::{
    http::ContentType,
    response::{Content, Debug},
};
use tokio::process::Command;

#[get("/frame?<path>")]
pub(crate) async fn handler(path: String) -> Result<Content<Vec<u8>>, Debug<Error>> {
    let image = extract_jpeg(&path).await?;
    let content_type = ContentType::JPEG;
    let content = Content(content_type, image);
    Ok(content)
}

async fn extract_jpeg(path: &str) -> Result<Vec<u8>, Error> {
    debug!("video path: {}", path);

    let ffmpeg = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

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
        if let Err(e) = String::from_utf8(output.stdout) {
            // skip ffmpeg metadata (content-type, content-length)
            let bytes = e.as_bytes();
            let inner = e.utf8_error();
            let valid = inner.valid_up_to();
            let (_, rest) = bytes.split_at(valid);
            Ok(rest.to_vec())
        } else {
            // no image data was returned, maybe input is not long enough
            Ok(vec![])
        }
    } else {
        // unknown error occurred
        let stderr = output.stderr;
        let e = String::from_utf8_lossy(&stderr);
        Err(anyhow!(e.to_string()))
    }
}
