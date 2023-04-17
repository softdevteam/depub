use getopts::Options;
use regex::RegexBuilder;
use std::{
    env,
    fs::{read_to_string, write},
    io::{stdout, Write},
    path::Path,
    process::{self, Command, Stdio},
};

enum PubKind {
    Pub,
    Crate,
    Super,
    Private,
}

fn process(oracle_cmd: &str, p: &Path) -> u64 {
    let pub_regex = RegexBuilder::new("pub(?:\\s*\\(\\s*(.*?)\\s*\\))?")
        .multi_line(true)
        .build()
        .unwrap();
    let mut cur_txt = read_to_string(p).unwrap();
    let mut i = 0;
    let mut cl = pub_regex.capture_locations();
    let mut num_changed = 0;
    while i < cur_txt.len() {
        print!(".");
        stdout().flush().ok();
        let old_txt = cur_txt.clone();
        let m = match pub_regex.captures_read_at(&mut cl, &old_txt, i) {
            Some(m) => m,
            None => break,
        };
        let mut kind = if let Some((start, end)) = cl.get(1) {
            match &cur_txt[start..end] {
                "crate" => PubKind::Crate,
                "super" => PubKind::Super,
                _ => {
                    // FIXME: this captures things we don't need to deal with (e.g. `pub(self)`),
                    // things we could deal with (e.g. `pub(in ...)`) and random strings we've
                    // accidentally picked up (e.g. `a pub(The Frog and Cow)`). We should probably
                    // do something better with the middle class of thing than simply ignoring it.
                    i = m.end();
                    continue;
                }
            }
        } else {
            PubKind::Pub
        };
        let mut next_txt = cur_txt.clone();
        let mut depubed = false;
        loop {
            let next_kind = match kind {
                PubKind::Pub => PubKind::Crate,
                PubKind::Crate => PubKind::Super,
                PubKind::Super => PubKind::Private,
                PubKind::Private => break,
            };
            let mut try_txt = cur_txt[..m.start()].to_string();
            let pub_txt = match next_kind {
                PubKind::Crate => "pub(crate) ",
                PubKind::Super => "pub(super) ",
                PubKind::Private => "",
                _ => unreachable!(),
            };
            try_txt.push_str(pub_txt);
            try_txt.push_str(&cur_txt[m.end()..]);
            write(p, &try_txt).unwrap();
            match Command::new("sh")
                .arg("-c")
                .arg(oracle_cmd)
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .status()
            {
                Ok(s) if s.success() => {
                    if !depubed {
                        num_changed += 1;
                        depubed = true;
                    }
                    next_txt = try_txt;
                }
                _ => {
                    if let PubKind::Super = next_kind {
                        // If we're depubing a root module, then `super` is invalid, causing
                        // the following (not entirely intuitive) error:
                        //   there are too many leading `super` keywords
                        // We still want to try Private visibility in such cases.
                    } else {
                        break;
                    }
                }
            }
            kind = next_kind;
        }
        cur_txt = next_txt;
        if cur_txt.len() >= old_txt.len() {
            i = m.end() + (cur_txt.len() - old_txt.len());
        } else {
            i = m.end() - (old_txt.len() - cur_txt.len());
        }
    }
    write(p, cur_txt).unwrap();
    num_changed
}

fn progname() -> String {
    match env::current_exe() {
        Ok(p) => p
            .file_name()
            .map(|x| x.to_str().unwrap_or("depub"))
            .unwrap_or("depub")
            .to_owned(),
        Err(_) => "depub".to_owned(),
    }
}

/// Print out program usage then exit. This function must not be called after daemonisation.
fn usage() -> ! {
    eprintln!("Usage: {} -c <command> file_1 [... file_n]", progname());
    process::exit(1)
}

fn main() {
    let matches = Options::new()
        .reqopt("c", "command", "Command to execute.", "string")
        .optflag("h", "help", "")
        .parse(env::args().skip(1))
        .unwrap_or_else(|_| usage());
    if matches.opt_present("h") || matches.free.is_empty() {
        usage();
    }

    let oracle_cmd = matches.opt_str("c").unwrap();
    let mut round = 1;
    loop {
        println!("===> Round {}", round);
        let mut changed = false;
        for p in &matches.free {
            print!("{}: ", p);
            stdout().flush().ok();
            let num_changed = process(oracle_cmd.as_str(), Path::new(&p));
            if num_changed > 0 {
                print!(" ({} items depub'ed)", num_changed);
                changed = true;
            }
            println!();
        }
        if !changed {
            break;
        }
        round += 1;
    }
}
