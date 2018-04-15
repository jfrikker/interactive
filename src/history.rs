use std::env::home_dir;
use std::ffi::OsStr;
use std::fs::{File, create_dir, metadata};
use std::io::{self, BufRead, BufReader, BufWriter, Lines, Write};
use std::path::{Path, PathBuf};

pub fn read_history(cmd: &OsStr) -> io::Result<HistoryIterator> {
    match prepare_history_path(cmd)? {
        None => Ok(HistoryIterator{iter: None}),
        Some(path) => {
            match File::open(&path) {
                Ok(f) => Ok(HistoryIterator{iter: Some(BufReader::new(f).lines())}),
                Err(e) => {
                    if e.kind() == io::ErrorKind::NotFound {
                        Ok(HistoryIterator{iter: None})
                    } else {
                        Err(e)
                    }
                }
            }
        }
    }
}

pub fn write_history<'a, I>(cmd: &OsStr, history: I) -> io::Result<()>
    where I: Iterator<Item = &'a str> {
    if let Some(path) = prepare_history_path(cmd)? {
        let mut f =
        BufWriter::new(File::create(&path)?);
        for line in history {
            writeln!(f, "{}", line)?;
        }
    };
    Ok(())
}

fn prepare_history_path(cmd: &OsStr) -> io::Result<Option<PathBuf>> {
    match home_dir() {
        None => Ok(None),
        Some(mut path) => {
            path.push(".interactive");
            ensure_dir(&path)?;
            path.push("history");
            ensure_dir(&path)?;
            path.push(cmd);
            Ok(Some(path))
        }
    }
}

fn ensure_dir(path: &Path) -> io::Result<()> {
    metadata(path)
        .map(|_| ())
        .or_else(|e| if e.kind() == io::ErrorKind::NotFound {
            create_dir(path)
        } else {
            Err(e)
        })
}

pub struct HistoryIterator {
    iter: Option<Lines<BufReader<File>>>
}

impl Iterator for HistoryIterator {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.as_mut()
            .and_then(|i| i.next())
    }
}