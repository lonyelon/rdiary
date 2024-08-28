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

pub fn get_entries_in_path(path: String) -> Vec<NaiveDate> {
    let mut dates = Vec::new();

    let files = fs::read_dir(path).unwrap();
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
            if date_loop.format("%Y-%m-%d").to_string() != file_dates[0] {
                dates.push(date_loop);
                i += 1;
            } else {
                break;
            }
        }
    }

    for j in 0..100 {
        dates.push(today - chrono::Duration::days(i + j));
    }

    dates
}

pub fn read_entry(base_path: &str, date: &NaiveDate) -> String {
    let filename_org = format!("{}/{}.org", base_path, date.format("%Y-%m-%d"));
    let filename_md = format!("{}/{}.md", base_path, date.format("%Y-%m-%d"));

    if Path::new(&filename_org).exists() {
        fs::read_to_string(&filename_org).unwrap()
    } else if Path::new(&filename_md).exists() {
        fs::read_to_string(&filename_md).unwrap()
    } else {
        String::from("No entry for this date.")
    }
}

pub fn edit_entry(editor: &String, base_path: &str, date: &NaiveDate) -> io::Result<()> {
    let path = get_entry_path(base_path, date).path;

    if !Path::new(&path).exists() {
        File::create(&path)?;
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let status = Command::new(editor).arg(&path).status()?;

    if !status.success() {
        eprintln!("Failed to open editor");
    }
    
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    Ok(())
}

pub struct DiaryEntry {
    pub path: String,
    pub exists: bool,
}

impl DiaryEntry {
    fn new(path: String, exists: bool) -> DiaryEntry {
        DiaryEntry {
            path: path,
            exists: exists,
        }
    }
}

// Return the path for the date file and wether it actually exists or not.
pub fn get_entry_path(base_path: &str, date: &NaiveDate) -> DiaryEntry {
    let filename_org = format!("{}/{}.org", base_path, date.format("%Y-%m-%d"));
    let filename_md = format!("{}/{}.md", base_path, date.format("%Y-%m-%d"));

    if Path::new(&filename_org).exists() {
        DiaryEntry::new(filename_org, true)
    } else if Path::new(&filename_md).exists() {
        DiaryEntry::new(filename_md, true)
    } else {
        DiaryEntry::new(filename_md, false)
    }
}
