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

fn process(cmd: &str, p: &Path) {
    let pub_regex = RegexBuilder::new("pub(?:\\s*\\(\\s*(.*?)\\s*\\))?")
        .multi_line(true)
        .build()
        .unwrap();
    let mut cur_txt = read_to_string(p).unwrap();
    let mut i = 0;
    let mut cl = pub_regex.capture_locations();
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
                _ => todo!(),
            }
        } else {
            PubKind::Pub
        };
        let mut next_txt = cur_txt.clone();
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
            try_txt.push_str(&pub_txt);
            try_txt.push_str(&cur_txt[m.end()..]);
            write(p, &try_txt).unwrap();
            match Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .status()
            {
                Ok(s) if s.success() => {
                    next_txt = try_txt;
                }
                _ => break,
            }
            kind = next_kind;
        }
        cur_txt = next_txt;
        if cur_txt.len() >= old_txt.len() {
            i += m.end() + (cur_txt.len() - old_txt.len());
        } else {
            i += m.end() - (old_txt.len() - cur_txt.len());
        }
    }
    write(p, cur_txt).unwrap();
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
    eprintln!(
        "Usage: {} -c <command> file_1 [... file_n]",
        progname = progname()
    );
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

    let cmd_str = matches.opt_str("c").unwrap();
    for p in matches.free {
        print!("{}: ", p);
        stdout().flush().ok();
        process(cmd_str.as_str(), &Path::new(&p));
        println!("");
    }
}
