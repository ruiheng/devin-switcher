//! dsw — headless CLI for the devin-switch vault. Same backend as the GUI;
//! usable over SSH on machines without a desktop.
//!
//!   dsw status                  current sign-in + quota + running devin procs
//!   dsw list                    all profiles, active marked with *
//!   dsw save [name]             save the current sign-in as a profile
//!   dsw use <name>              switch CLI/Desktop to a profile
//!   dsw rename <from> <to>      rename a profile
//!   dsw delete <name>           remove a profile's saved credentials
//!   dsw refresh <name>          re-fetch a profile's quota
//!   dsw login [name]            add an account: prompts for a session token
//!   dsw add-token <token> [name]  same, non-interactive (token as argument)
//!   dsw parallel <name> [cwd]   print a command for a parallel isolated session

use std::io::Write;
use std::process::ExitCode;

use devin_switch_lib::model::ProfileInfo;
use devin_switch_lib::ops;
use devin_switch_lib::store;
use devin_switch_lib::usage::Usage;

fn bar(pct: f64) -> String {
    let n = ((pct / 100.0) * 10.0).round().clamp(0.0, 10.0) as usize;
    format!("{}{}", "█".repeat(n), "░".repeat(10 - n))
}

fn fmt_date(unix: i64) -> String {
    let (y, m, d) = devin_switch_lib::model::civil_from_days(unix.div_euclid(86400));
    format!("{y:04}-{m:02}-{d:02}")
}

fn quota_lines(u: &Usage, indent: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(p) = u.daily_left {
        let mut s = format!("{indent}daily   {} {p:.0}%", bar(p));
        if let Some(t) = u.daily_reset_unix {
            s += &format!("  resets {}", fmt_date(t));
        }
        out.push(s);
    }
    if let Some(p) = u.weekly_left {
        let mut s = format!("{indent}weekly  {} {p:.0}%", bar(p));
        if let Some(t) = u.weekly_reset_unix {
            s += &format!("  resets {}", fmt_date(t));
        }
        out.push(s);
    }
    if let Some(limit) = u.acu_limit {
        if limit > 0.0 {
            let used = u.acu_used.unwrap_or(0.0);
            let pct = (limit - used) / limit * 100.0;
            out.push(format!(
                "{indent}acu     {} {used:.2}/{limit:.2} used",
                bar(pct)
            ));
        }
    }
    if let Some(m) = u.overage_micros {
        if m > 0.0 {
            out.push(format!("{indent}overage ${:.2}", m / 1e6));
        }
    }
    if let Some(end) = &u.plan_end {
        out.push(format!("{indent}plan ends {}", &end[..10.min(end.len())]));
    }
    out
}

fn show_profile(p: &ProfileInfo) {
    let mark = if p.is_active { "*" } else { " " };
    let id = [p.meta.email.as_str(), p.meta.display_name.as_str()]
        .into_iter()
        .find(|s| !s.is_empty())
        .unwrap_or("unknown account");
    let plan = if p.meta.plan.is_empty() {
        String::new()
    } else {
        format!(" ({})", p.meta.plan)
    };
    println!("{mark} {:<20} {id}{plan}", p.name);
    if let Some(u) = &p.meta.usage {
        for l in quota_lines(u, "      ") {
            println!("{l}");
        }
    }
    if !p.meta.note.is_empty() {
        println!("      note: {}", p.meta.note);
    }
}

fn usage() -> &'static str {
    "usage: dsw <command>\n\
     \x20 status                current sign-in, quota, running devin processes\n\
     \x20 list                  profiles (* = active)\n\
     \x20 save [name]           save the current sign-in (name auto-derived)\n\
     \x20 use <name>            switch CLI/Desktop to a profile\n\
     \x20 rename <from> <to>    rename a profile\n\
     \x20 delete <name>         remove a profile\n\
     \x20 refresh <name>        re-fetch a profile's quota\n\
     \x20 login [name]            add an account (prompts for a session token)\n\
     \x20 add-token <token> [name]  same, non-interactive\n\
     \x20 parallel <name> [cwd] print a command to run a parallel session"
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let r = match cmd {
        "status" => cmd_status(),
        "list" | "ls" => cmd_list(),
        "save" => ops::save_current(args.get(1).map(String::as_str).unwrap_or(""), "").map(|p| {
            println!("saved as \"{}\"", p.name);
            show_profile(&p);
        }),
        "use" => args
            .get(1)
            .ok_or_else(|| "use needs a profile name".to_string())
            .and_then(|n| ops::use_profile(n))
            .map(|r| {
                let who = if r.auth.logged_in {
                    r.auth.name
                } else {
                    "account".into()
                };
                println!("switched to {who}");
                if !r.restart_needed.is_empty() {
                    println!("restart to apply: {}", r.restart_needed.join(", "));
                }
            }),
        "rename" | "mv" => match (args.get(1), args.get(2)) {
            (Some(f), Some(t)) => store::rename(f, t).map(|p| println!("renamed to {}", p.name)),
            _ => Err("rename needs <from> <to>".into()),
        },
        "delete" | "rm" => args
            .get(1)
            .ok_or_else(|| "delete needs a profile name".to_string())
            .and_then(|n| store::remove(n))
            .map(|_| println!("deleted")),
        "refresh" => args
            .get(1)
            .ok_or_else(|| "refresh needs a profile name".to_string())
            .and_then(|n| ops::refresh_usage(n))
            .map(|p| show_profile(&p)),
        "add-token" => {
            let Some(tok) = args.get(1) else {
                return done(Err("add-token needs a token".into()));
            };
            ops::add_token(args.get(2).map(String::as_str).unwrap_or(""), tok)
                .map(|p| show_profile(&p))
        }
        "login" | "add" => cmd_login(args.get(1).map(String::as_str).unwrap_or("")),
        "parallel" | "par" => args
            .get(1)
            .ok_or_else(|| "parallel needs a profile name".to_string())
            .and_then(|n| ops::launch_parallel(n, args.get(2).map(String::as_str)))
            .map(|r| {
                if r.launched {
                    println!("launched");
                } else {
                    println!("{}", r.command);
                }
            }),
        _ => {
            eprintln!("{}", usage());
            return ExitCode::from(if cmd.is_empty() { 0 } else { 2 });
        }
    };
    done(r)
}

fn done(r: Result<(), String>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_status() -> Result<(), String> {
    let s = ops::status();
    if s.auth.logged_in {
        println!(
            "signed in: {} <{}>{}",
            s.auth.name,
            s.auth.email,
            if s.auth.plan.is_empty() {
                String::new()
            } else {
                format!(" [{}]", s.auth.plan)
            }
        );
    } else {
        println!("not signed in");
    }
    if let Some(u) = &s.usage {
        for l in quota_lines(u, "  ") {
            println!("{l}");
        }
    }
    if !s.running.is_empty() {
        println!("running: {}", s.running.join(", "));
    }
    println!("credentials: {}", s.credentials_path);
    Ok(())
}

fn cmd_list() -> Result<(), String> {
    let profiles = store::list();
    if profiles.is_empty() {
        println!("no profiles — `dsw save` the current sign-in or `dsw login`");
    }
    for p in &profiles {
        show_profile(p);
    }
    Ok(())
}

/// Interactive login = paste a session token (hidden input). On unix the
/// terminal echo is disabled while reading; on other platforms just reads.
fn cmd_login(name: &str) -> Result<(), String> {
    eprintln!("paste a devin-session-token$… (input hidden)");
    eprintln!("get it from the sign-in page of a logged-in devin.ai session\n");
    let mut tok = String::new();
    #[cfg(unix)]
    {
        let _ = std::process::Command::new("stty").arg("-echo").status();
        std::io::stdin()
            .read_line(&mut tok)
            .map_err(|e| e.to_string())?;
        let _ = std::process::Command::new("stty").arg("echo").status();
        eprintln!();
    }
    #[cfg(not(unix))]
    {
        std::io::stdin()
            .read_line(&mut tok)
            .map_err(|e| e.to_string())?;
    }
    let _ = std::io::stdout().flush();
    ops::add_token(name, tok.trim()).map(|p| {
        println!("saved");
        show_profile(&p);
    })
}
