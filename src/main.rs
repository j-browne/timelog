use chrono::Local;
use std::{
    error::Error,
    fs::File,
    io::{self, BufReader, BufWriter, Read},
};
use structopt::StructOpt;
use timelog::{read_entries, write_entries, Entry};

type Result<T> = std::result::Result<T, Box<Error>>;
const LOGNAME: &str = "log.json";

#[derive(Debug, StructOpt)]
#[structopt(name = "timelog", author = "")]
pub enum Opt {
    #[structopt(name = "print", author = "", about = "Print all log entries")]
    Print {},
    #[structopt(name = "start", author = "", about = "Create a new log entry")]
    Start {},
    #[structopt(name = "stop", author = "", about = "Complete the latest log entry")]
    Stop {},
    #[structopt(name = "note", author = "", about = "Add a note to the latest log entry")]
    Note {},
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let reader = get_file_reader(LOGNAME)?;
    let mut entries = read_entries(reader)?;

    match opt {
        Opt::Print {} => {
            let entries = entries.into_sorted_vec();
            for (i, e) in entries.iter().enumerate() {
                if i != 0 {
                    println!();
                }
                println!("{}", e);
            }
        }
        Opt::Start {} => {
            let start = Local::now();
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            let mut buf = Vec::new();
            stdin.read_to_end(&mut buf)?;
            let goal = String::from_utf8(buf)?;

            let new_entry = Entry {
                start: Some(start),
                goal,
                ..Entry::default()
            };
            entries.push(new_entry);
            let writer = get_file_writer(LOGNAME)?;
            write_entries(writer, entries)?;
        }
        Opt::Stop {} => {
            let stop = Local::now();
            let mut last_entry = entries.pop().ok_or("NoneError")?;
            if last_entry.stop.is_none() {
                let stdin = io::stdin();
                let mut stdin = stdin.lock();
                let mut buf = Vec::new();
                stdin.read_to_end(&mut buf)?;
                let result = String::from_utf8(buf)?;

                last_entry.stop = Some(stop);
                last_entry.result = result;
            } else {
                Err("last entry was already completed")?;
            }
            entries.push(last_entry);

            let writer = get_file_writer(LOGNAME)?;
            write_entries(writer, entries)?;
        }
        Opt::Note {} => {
            let mut last_entry = entries.pop().ok_or("NoneError")?;
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            let mut buf = Vec::new();
            stdin.read_to_end(&mut buf)?;
            let note = String::from_utf8(buf)?;

            last_entry.notes.push(note);
            entries.push(last_entry);

            let writer = get_file_writer(LOGNAME)?;
            write_entries(writer, entries)?;
        }
    }

    Ok(())
}

fn get_file_reader(filename: &str) -> Result<Option<BufReader<File>>> {
    let reader = File::open(filename);

    if let Err(e) = reader {
        if e.kind() == std::io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(e)?
        }
    } else {
        Ok(Some(BufReader::new(reader?)))
    }
}

fn get_file_writer(filename: &str) -> Result<BufWriter<File>> {
    let writer = File::create(filename);
    Ok(BufWriter::new(writer?))
}
