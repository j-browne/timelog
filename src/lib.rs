#![feature(custom_attribute)]
#[macro_use] extern crate serde_derive;

use chrono::{DateTime, Local};
use std::{cmp::Ordering, collections::BinaryHeap};
use structopt::StructOpt;

#[derive(Debug, Clone, Deserialize, Serialize, PartialOrd, Eq, PartialEq)]
pub struct Entry {
    #[serde(default)]
    start: Option<DateTime<Local>>,
    #[serde(default)]
    stop: Option<DateTime<Local>>,
    #[serde(default)]
    goal: String,
    #[serde(default)]
    result: String,
    #[serde(default)]
    notes: String,
}


impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.start, other.start) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => Ordering::Greater,
            (None, _) => Ordering::Less,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "timelog", author="")]
pub enum Opt {
    #[structopt(name = "print", author="", about = "Print all log entries")]
    Print {
    },
    #[structopt(name = "add", author="", about = "Add a log entry")]
    Add {
    },
    #[structopt(name = "start", author="", about = "Create a new log entry")]
    Start {
    },
    #[structopt(name = "stop", author="", about = "Complete the most recent log entry")]
    Stop {
    },
}
