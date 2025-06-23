use anyhow::Result;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;

pub async fn stream_to_file<R>(
    label: &str,
    reader: R,
    output_path: &Path,
    total_size: u64,
) -> Result<()>
where
    R: AsyncRead + Unpin,
{
    let pb = ProgressBar::new(total_size);

    let template = format!(
        "{label} {{bar:40.cyan/blue}} {{bytes}}/{{total_bytes}} ({{bytes_per_sec}}, ETA: {{eta}})"
    );
    pb.set_style(
        ProgressStyle::with_template(&template)
            .unwrap()
            .progress_chars("=> "),
    );

    let mut file = File::create(output_path)?;
    let mut stream = ReaderStream::new(reader);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Done");

    Ok(())
}
