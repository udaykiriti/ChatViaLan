//! File upload handling.

use bytes::Buf;
use futures::TryStreamExt;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::multipart::{FormData, Part};
use futures::StreamExt;

/// Handle multipart file upload.
pub async fn handle_upload(form: FormData) -> Result<impl warp::Reply, warp::Rejection> {
    // Ensure uploads dir exists
    if let Err(e) = tokio::fs::create_dir_all("uploads").await {
        eprintln!("failed to create uploads dir: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"ok": false, "error": "internal"})),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let mut parts = form;
    let mut saved_urls: Vec<serde_json::Value> = Vec::new();

    while let Some(part_result) = parts.try_next().await.map_err(|e| {
        eprintln!("multipart error: {}", e);
        warp::reject::reject()
    })? {
        let part: Part = part_result;
        let filename_opt: Option<String> = part.filename().map(|s| s.to_string());

        if let Some(filename) = filename_opt {
            // Sanitize filename
            let safe_name = filename.replace(
                |c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'),
                "_",
            );
            let id = Uuid::new_v4().to_string();
            let stored_name = format!("{}_{}", id, safe_name);
            let path = format!("uploads/{}", stored_name);

            let mut file = tokio::fs::File::create(&path).await.map_err(|e| {
                eprintln!("create file error: {}", e);
                warp::reject::reject()
            })?;

            let mut stream = part.stream();

            while let Some(chunk_res) = stream.next().await {
                let mut buf = chunk_res.map_err(|e| {
                    eprintln!("chunk error: {}", e);
                    warp::reject::reject()
                })?;
                while buf.has_remaining() {
                    let bytes = buf.chunk();
                    if !bytes.is_empty() {
                        file.write_all(bytes).await.map_err(|e| {
                            eprintln!("write error: {}", e);
                            warp::reject::reject()
                        })?;
                        let n = bytes.len();
                        buf.advance(n);
                    } else {
                        break;
                    }
                }
            }

            let meta_len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
            let url = format!("/uploads/{}", stored_name);
            saved_urls.push(json!({ "filename": filename, "url": url, "size": meta_len }));
        }
    }

    if saved_urls.is_empty() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({"ok": false, "files": []})),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({"ok": true, "files": saved_urls})),
            StatusCode::OK,
        ))
    }
}
