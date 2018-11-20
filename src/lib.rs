#![feature(custom_attribute)]
#[macro_use]
extern crate serde_derive;

use chrono::{DateTime, Local, Duration};
use itertools::{EitherOrBoth, Itertools};
use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    fmt::{self, Display, Write},
    io,
    iter::once,
};

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialOrd, Eq, PartialEq)]
pub struct Entry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Local>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<DateTime<Local>>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub goal: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub result: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
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

fn fmt_str_title_pad(text: &str, title: &str, pad: usize) -> Result<String, fmt::Error> {
    let mut s = String::new();
    let it = once(title).zip_longest(text.lines()).enumerate();
    for (i, either_or_both) in it {
        if i != 0 {
            writeln!(s)?;
        }

        let (left, right) = match either_or_both {
            EitherOrBoth::Both(left, right) => (left, right),
            EitherOrBoth::Left(left) => (left, ""),
            EitherOrBoth::Right(right) => ("", right),
        };
        write!(s, "{l:<width$}{r}", l = left, width = pad, r = right)?;
    }

    Ok(s)
}

fn fmt_option_title_pad<T: Display>(
    text: &Option<T>,
    title: &str,
    pad: usize,
) -> Result<String, fmt::Error> {
    let mut s = String::new();
    write!(s, "{:<width$}", title, width = pad)?;
    if let Some(right) = text {
        write!(s, "{}", right)?;
    } else {
        write!(s, "--")?;
    }
    Ok(s)
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let duration = if let (Some(start), Some(stop)) = (self.start, self.stop) {
            Some(stop - start)
        } else {
            None
        };

        enum Data<'a> {
            OpDt(Option<DateTime<Local>>),
            OpSt(Option<String>),
            St(&'a str),
        };

        let duration = duration.map(|x| format_dur(x));
        let mut to_output = vec![
            ("Start Time:", Data::OpDt(self.start)),
            ("Stop Time:", Data::OpDt(self.stop)),
            ("Duration:", Data::OpSt(duration)),
            ("Goal:", Data::St(&self.goal)),
            ("Result:", Data::St(&self.result)),
        ];
        for note in &self.notes {
            to_output.push(("Note:", Data::St(note)));
        }

        let pad = to_output.iter().map(|x| x.0.len()).max().unwrap() + 1;

        for (i, (title, data)) in to_output.iter().enumerate() {
            if i != 0 {
                writeln!(f)?;
            }

            match data {
                Data::OpDt(x) => write!(f, "{}", fmt_option_title_pad(x, title, pad)?)?,
                Data::OpSt(x) => write!(f, "{}", fmt_option_title_pad(x, title, pad)?)?,
                Data::St(x) => write!(f, "{}", fmt_str_title_pad(x, title, pad)?)?,
            }
        }

        Ok(())
    }
}

pub fn read_entries<R: io::Read>(
    reader: Option<R>,
) -> Result<BinaryHeap<Entry>, serde_json::Error> {
    if let Some(reader) = reader {
        Ok(serde_json::from_reader(reader)?)
    } else {
        Ok(BinaryHeap::default())
    }
}

pub fn write_entries<W: io::Write>(
    writer: W,
    entries: BinaryHeap<Entry>,
) -> Result<(), serde_json::Error> {
    let entries = entries.into_sorted_vec();
    serde_json::to_writer_pretty(writer, &entries)?;
    Ok(())
}

pub fn format_dur(mut dur: Duration) -> String {
    let mut out = String::new();
    let d = dur.num_days();
    if d != 0 {
        out += &format!("{}d", d);
        dur = dur - Duration::days(d);
    }
    let h = dur.num_hours();
    if h != 0 {
        out += &format!("{}h", h);
        dur = dur - Duration::hours(h);
    }
    let m = dur.num_minutes();
    if m != 0 {
        out += &format!("{}m", m);
        dur = dur - Duration::minutes(m);
    }
    let s = dur.num_seconds();
    if s != 0 {
        out += &format!("{}s", s);
    }
    out
}
