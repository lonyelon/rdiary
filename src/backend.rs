use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::Command;

use chrono::{Local, NaiveDate};
use crossterm::{
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen
    },
};

#[derive(Debug, Clone)]
pub struct DiaryEntry {
    pub path: String,
    pub date: NaiveDate,
    pub exists: bool,
}

pub fn get_entries_in_path(path: String) -> Vec<DiaryEntry> {
    let mut dates = Vec::new();

    let files = fs::read_dir(&path).unwrap();
    let mut file_dates: Vec<String> = Vec::new();
    for file in files {
        let okfile = file.unwrap();
        if okfile.file_type().unwrap().is_file() {
            file_dates.push(
                okfile.file_name()
                    .to_str().unwrap()
                    .chars()
                    .take(10)
                    .into_iter()
                    .collect()
            );
        }
    };

    file_dates.sort();

    let today = Local::now().date_naive();
    let mut i = 0;
    if file_dates.len() > 0 {
        loop {
            let date_loop = today - chrono::Duration::days(i);
            let entry_loop = get_entry_path(&path.as_str(), &date_loop);
            if date_loop.format("%Y-%m-%d").to_string() != file_dates[0] {
                dates.push(DiaryEntry {
                    path: entry_loop.0,
                    date: date_loop,
                    exists: entry_loop.1,
                });
                i += 1;
            } else {
                dates.push(DiaryEntry {
                    path: entry_loop.0,
                    date: date_loop,
                    exists: entry_loop.1,
                });
                break;
            }
        }
    }

    for j in 0..100 {
        let date_loop = today - chrono::Duration::days(i + j);
        dates.push(DiaryEntry {
            path: get_entry_path(&path.as_str(), &date_loop).0,
            date: date_loop,
            exists: false,
        });
    }

    dates
}

pub fn read_entry(entry: &DiaryEntry) -> String {
    if Path::new(&entry.path).exists() {
        fs::read_to_string(&entry.path).unwrap()
    } else {
        String::from("No entry for this date.")
    }
}

// Return the path for the date file and wether it actually exists or not.
fn get_entry_path(base_path: &str, date: &NaiveDate) -> (String, bool) {
    let filename_org = format!("{}/{}.org", base_path, date.format("%Y-%m-%d"));
    let filename_md = format!("{}/{}.md", base_path, date.format("%Y-%m-%d"));

    if Path::new(&filename_org).exists() {
        (filename_org, true)
    } else if Path::new(&filename_md).exists() {
        (filename_md, true)
    } else {
        (filename_md, false)
    }
}

pub fn edit_entry(editor: &String, entry: &DiaryEntry) -> io::Result<()> {
    if !Path::new(&entry.path).exists() {
        File::create(&entry.path)?;
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let status = Command::new(editor).arg(&entry.path).status()?;

    if !status.success() {
        eprintln!("Failed to open editor");
    }
    
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    Ok(())
}
