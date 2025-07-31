use std::str::FromStr;
use anyhow::anyhow;
use tower_lsp_server::lsp_types::Uri;
use urlencoding::encode;

#[cfg(target_os = "windows")]
pub fn convert_uri(uri: Uri) -> anyhow::Result<Uri> {
    let segments = uri.path().segments();
    let mut segments = segments
        .map(|segments| segments.to_string())
        .collect::<Vec<_>>();

    let drive_letter = segments[0].chars().next().unwrap();

    let drive = format!("{}:", drive_letter.to_ascii_lowercase());
    segments[0] = encode(&drive).to_string();

    let path = format!("file:///{}", segments.join("/"));
    Uri::from_str(&path).map_err(|e| anyhow!(e))
}

#[cfg(not(target_os = "windows"))]
pub fn convert_uri(uri: Uri) -> anyhow::Result<Uri> {
    Ok(uri)
}