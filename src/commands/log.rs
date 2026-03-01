use crate::logger::get_log_path_for_read;
use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

pub fn run_log_command(lines: Option<usize>) -> Result<()> {
    let log_path = get_log_path_for_read();

    if !log_path.exists() {
        println!("No log file found at: {}", log_path.display());
        return Ok(());
    }

    let file = File::open(&log_path)?;
    let reader = BufReader::new(file);

    let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let count = all_lines.len();
    let lines_to_show = lines.unwrap_or(50);

    let start = count.saturating_sub(lines_to_show);

    for line in &all_lines[start..] {
        println!("{}", line);
    }

    Ok(())
}

pub fn follow_log() -> Result<()> {
    let log_path = get_log_path_for_read();

    if !log_path.exists() {
        println!("No log file found at: {}", log_path.display());
        return Ok(());
    }

    let mut file = File::open(&log_path)?;
    file.seek(SeekFrom::End(0))?;

    let reader = BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        println!("{}", line);
    }

    Ok(())
}
