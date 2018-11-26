use chrono::{Datelike, Duration, Local};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    hash::Hash,
    io::{self, BufReader, BufWriter, Read},
};
use structopt::{
    clap::{AppSettings, ArgGroup},
    StructOpt,
};
use timelog::{format_dur, read_entries, write_entries, Entry};

type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "timelog",
    author = "",
    raw(setting = "AppSettings::DeriveDisplayOrder")
)]
struct Opt {
    #[structopt(
        short = "l",
        long = "log-file",
        default_value = "log.json",
        help = "The log file to use",
    )]
    log_file: String,
    #[structopt(subcommand)]
    sub_command: SubCommand,
}

fn time_arg_group() -> ArgGroup<'static> {
    ArgGroup::with_name("time").required(true).multiple(true)
}

#[derive(Debug, StructOpt)]
enum SubCommand {
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
    #[structopt(name = "print", author = "", about = "Print all log entries")]
    Print {},
    #[structopt(
        name = "summary",
        author = "",
        about = "Summarize time over certain time periods",
        raw(
            group = "time_arg_group()",
            setting = "AppSettings::DeriveDisplayOrder",
        ),
    )]
    Summary {
        #[structopt(
            short = "y",
            long = "yearly",
            group = "time",
            help = "Prints yearly summaries",
        )]
        yearly: bool,
        #[structopt(
            short = "m",
            long = "monthly",
            group = "time",
            help = "Prints monthly summaries",
        )]
        monthly: bool,
        #[structopt(
            short = "w",
            long = "weekly",
            group = "time",
            help = "Prints weekly summaries",
        )]
        weekly: bool,
        #[structopt(
            short = "d",
            long = "daily",
            group = "time",
            help = "Prints daily summaries",
        )]
        daily: bool,
    },
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
        SubCommand::Summary {
            yearly,
            monthly,
            weekly,
            daily,
        } => {
            let mut years = HashMap::new();
            let mut months = HashMap::new();
            let mut weeks = HashMap::new();
            let mut days = HashMap::new();

            for e in entries.iter() {
                if let (Some(start), Some(stop)) = (e.start, e.stop) {
                    let date = start.date();
                    let dur = stop - start;

                    if yearly {
                        let y = date
                            .with_ordinal0(0)
                            .expect("with_ordinal0(0) caused an error");
                        let entry = years.entry(y).or_insert(Duration::zero());
                        *entry = *entry + dur;
                    }
                    if monthly {
                        let m = date.with_day0(0).expect("with_day0(0) caused an error");
                        let entry = months.entry(m).or_insert(Duration::zero());
                        *entry = *entry + dur;
                    }
                    if weekly {
                        let y = start.year();
                        let w = start.iso_week().week();
                        let entry = weeks.entry((y, w)).or_insert(Duration::zero());
                        *entry = *entry + dur;
                    }
                    if daily {
                        let entry = days.entry(date).or_insert(Duration::zero());
                        *entry = *entry + dur;
                    }
                }
            }

            if yearly {
                for (y, dur) in sort_hash_map(years) {
                    println!("{}: {}", y.format("%Y"), format_dur(dur));
                }
                if monthly || weekly || daily {
                    println!();
                }
            }
            if monthly {
                for (m, dur) in sort_hash_map(months) {
                    println!("{}: {}", m.format("%B %Y"), format_dur(dur));
                }
                if weekly || daily {
                    println!();
                }
            }
            if weekly {
                for ((y, w), dur) in sort_hash_map(weeks) {
                    println!("{}, Week {}: {}", y, w, format_dur(dur));
                }
                if daily {
                    println!();
                }
            }
            if daily {
                for (d, dur) in sort_hash_map(days) {
                    println!("{}: {}", d.format("%v"), format_dur(dur));
                }
            }
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

fn sort_hash_map<K, V>(mut m: HashMap<K, V>) -> Vec<(K, V)> 
    where K: Eq + Hash + Ord + Copy {
    let mut v: Vec<(K, V)> = m.drain().collect();
    v.sort_by_key(|x: &(K, V)| x.0);
    v
}
