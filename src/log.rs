use std::path::{PathBuf};
use git2::{Commit, Repository, Time};
use git2::{Error};
use std::str;
use chrono::prelude::*;

pub fn print_log(path: &PathBuf) -> Result<String, Error> {
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;

    revwalk.push_head()?;

    macro_rules! filter_try {
        ($e:expr) => {
            match $e {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            }
        };
    }
    let revwalk = revwalk
        .filter_map(|id| {
            let id = filter_try!(id);
            let commit = filter_try!(repo.find_commit(id));
            Some(Ok(commit))
        })
        .take(100);

    let mut log = String::new();
    // print!
    for commit in revwalk {
        let commit = commit?;
        log.push_str(format!("{}", print_commit(&commit)).as_str());
    }

    Ok(log)
}

fn print_commit(commit: &Commit) -> String {
    let mut commit_message = String::new();
    commit_message.push_str(format!("commit {}", commit.id()).as_str());

    if commit.parents().len() > 1 {
        commit_message.push_str(format!("\n").as_str());
        commit_message.push_str(format!("Merge:").as_str());
        for id in commit.parent_ids() {
            commit_message.push_str(format!(" {:.8}", id).as_str());
        }
        commit_message.push_str(format!("\n").as_str());
    } else {
        commit_message.push_str(format!("\n").as_str());
    }

    let author = commit.author();
    commit_message.push_str(format!("Author: {}", author).as_str());
    commit_message.push_str(format!("\n").as_str());
    commit_message.push_str(print_time(&author.when(), "Date:   ").as_str());
    commit_message.push_str(format!("\n").as_str());

    for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
        commit_message.push_str(format!("    {}", line).as_str());
        commit_message.push_str(format!("\n").as_str());
    }
    commit_message.push_str(format!("\n").as_str());
    return commit_message
}

fn print_time(time: &Time, prefix: &str) -> String {
    let (offset, sign) = match time.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);

    let naive = NaiveDateTime::from_timestamp(time.seconds() + (time.offset_minutes() as i64) * 60, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    let time = datetime.format("%Y-%m-%d %H:%M:%S");

    return format!(
        "{}{} {}{:02}{:02}",
        prefix,
        time.to_string(),
        sign,
        hours,
        minutes
    );
}
