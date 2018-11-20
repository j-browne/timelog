use chrono::{Local, Duration, Datelike};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, BufReader, BufWriter, Read},
};
use structopt::StructOpt;
use timelog::{read_entries, write_entries, Entry};

type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug, StructOpt)]
#[structopt(name = "timelog", author = "")]
struct Opt {
    #[structopt(short = "l", long = "log-file", default_value = "log.json")]
    log_file: String,
    #[structopt(subcommand)]
    sub_command: SubCommand,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    #[structopt(name = "print", author = "", about = "Print all log entries")]
    Print {},
    #[structopt(name = "summary", author = "", about = "Summarize time over certain time periods")]
    Summary {
        #[structopt(short = "y", long = "yearly")]
        yearly: bool,
        #[structopt(short = "w", long = "weekly")]
        weekly: bool,
        #[structopt(short = "d", long = "daily")]
        daily: bool,
    },
    #[structopt(name = "start", author = "", about = "Create a new log entry")]
    Start {},
    #[structopt(name = "stop", author = "", about = "Complete the latest log entry")]
    Stop {},
    #[structopt(
        name = "note",
        author = "",
        about = "Add a note to the latest log entry"
    )]
    Note {},
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let reader = get_file_reader(&opt.log_file)?;
    let mut entries = read_entries(reader)?;

    match opt.sub_command {
        SubCommand::Print {} => {
            let entries = entries.into_sorted_vec();
            for (i, e) in entries.iter().enumerate() {
                if i != 0 {
                    println!();
                }
                println!("{}", e);
            }
        }
        SubCommand::Summary {..} => {
            let mut yearly = HashMap::new();
            let mut monthly = HashMap::new();
            let mut weekly = HashMap::new();
            let mut daily = HashMap::new();

            for e in entries.iter() {
                if let (Some(start), Some(stop)) = (e.start, e.stop) {
                    let y = start.year();
                    let m = start.month();
                    let w = start.iso_week().week();
                    let d = start.ordinal();
                    let dur = stop - start;

                    let entry = yearly.entry(y).or_insert(Duration::zero());
                    *entry = *entry + dur;
                    let entry = monthly.entry((y, m)).or_insert(Duration::zero());
                    *entry = *entry + dur;
                    let entry = weekly.entry((y, w)).or_insert(Duration::zero());
                    *entry = *entry + dur;
                    let entry = daily.entry((y, d)).or_insert(Duration::zero());
                    *entry = *entry + dur;
                }
            }

            println!("{:?}", yearly);
            println!("{:?}", monthly);
            println!("{:?}", weekly);
            println!("{:?}", daily);
        }
        SubCommand::Start {} => {
            let start = Local::now();
            println!("Type a goal for this entry. Use EOF (Ctrl-D) to finish.");

            let goal = get_input()?;

            let new_entry = Entry {
                start: Some(start),
                goal,
                ..Entry::default()
            };
            entries.push(new_entry);
            let writer = get_file_writer(&opt.log_file)?;
            write_entries(writer, entries)?;
        }
        SubCommand::Stop {} => {
            let stop = Local::now();
            let mut last_entry = entries.pop().ok_or("NoneError")?;
            if last_entry.stop.is_none() {
                println!("{}", last_entry);
                println!();
                println!("Type a result for this entry. Use EOF (Ctrl-D) to finish.");

                let result = get_input()?;
                last_entry.stop = Some(stop);
                last_entry.result = result;
            } else {
                Err("last entry was already completed")?;
            }
            entries.push(last_entry);

            let writer = get_file_writer(&opt.log_file)?;
            write_entries(writer, entries)?;
        }
        SubCommand::Note {} => {
            let mut last_entry = entries.pop().ok_or("NoneError")?;
            println!("{}", last_entry);
            println!();
            println!("Type a note for this entry. Use EOF (Ctrl-D) to finish.");

            let note = get_input()?;
            last_entry.notes.push(note);
            entries.push(last_entry);

            let writer = get_file_writer(&opt.log_file)?;
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

fn get_input() -> Result<String> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}
