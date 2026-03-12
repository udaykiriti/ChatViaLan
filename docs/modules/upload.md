# Module: upload.rs

**Role:** Handles multipart file upload requests and saves files to the `uploads/` directory.

---

## Entry Point

### handle_upload

```rust
pub async fn handle_upload(form: FormData, clients: Clients) -> Result<impl Reply, Rejection>
```

Warp handler for `POST /upload`. Processes a multipart form body.

---

## Processing Steps

1. Iterate over form parts. For each part that has a filename:
   a. Read the full part body up to the Warp-configured size limit (set to 5 GB in `main.rs`).
   b. Sanitize the filename: replace any character outside `[A-Za-z0-9._-]` with `_`.
   c. Generate a storage filename: `<uuid_prefix>_<sanitized_original>` where `uuid_prefix` is the first 8 characters of a UUID v4.
   d. Create the `uploads/` directory if it does not exist.
   e. Write the file asynchronously using `tokio::fs::write`.
   f. Log the upload with `tracing::info`.

2. If no file parts were found, return `400 Bad Request`.

3. On file write failure, return `500 Internal Server Error`.

4. On success, return `201 Created` with a JSON body:

```json
{
  "filename": "<uuid_prefix>_<original_name>",
  "url": "/uploads/<uuid_prefix>_<original_name>",
  "size": 204800
}
```

---

## Notes

- The `clients` parameter is accepted by the handler signature for future use (e.g., broadcasting an upload notification to the room) but is not currently used.
- Filename sanitization prevents directory traversal and shell-unsafe characters. The UUID prefix prevents collisions between files with the same original name.
- Files in `uploads/` are served statically by the `GET /uploads/:file` route registered in `main.rs`.
- There is no file type restriction or virus scanning. The server is designed for trusted LAN use.
