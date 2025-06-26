use indicatif::{ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::fs::{self};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use crate::types::DownloadProgress;

pub fn display_progress_bar(progress_path: &Path) -> anyhow::Result<()> {
    let pb_style = ProgressStyle::default_bar()
        .template("{msg} {bar:40.cyan/blue} {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("=> ");

    let pb = ProgressBar::new(100);

    pb.set_style(pb_style);

    let mut window: VecDeque<(Instant, u64)> = VecDeque::new();
    loop {
        let content = match fs::read_to_string(progress_path) {
            Ok(c) => c,
            Err(_) => {
                pb.finish_with_message("Done");
                break;
            }
        };

        let progress: DownloadProgress = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        pb.set_length(progress.total_bytes);
        pb.set_position(progress.progress_bytes);

        let now = Instant::now();
        let bytes = progress.progress_bytes;
        window.push_back((now, bytes));

        // Window of 20 updates, maybe change later
        if window.len() > 20 {
            window.pop_front();
        }

        let speed_bps =
            if let (Some((old_t, old_b)), Some((new_t, new_b))) = (window.front(), window.back()) {
                let byte_delta = new_b.saturating_sub(*old_b);
                let time_delta = (*new_t - *old_t).as_secs_f64();
                if time_delta > 0.0 {
                    byte_delta as f64 / time_delta
                } else {
                    0.0
                }
            } else {
                0.0
            };

        pb.set_message(format!(
            "{}\n{:.2} MB/s",
            progress.file_name,
            speed_bps / 1_048_576.0
        ));

        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}
