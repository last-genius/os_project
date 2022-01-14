use super::RustShellOutput;
use std::str::FromStr;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::env;

pub enum RustShellBuiltin {
    Echo,
    History,
    Cd,
    Pwd
}

impl FromStr for RustShellBuiltin {
    type Err = ();

    fn from_str(s : &str) -> Result<Self, ()> {
        match s {
            "echo" => Ok(RustShellBuiltin::Echo),
            "history" => Ok(RustShellBuiltin::History),
            "cd" => Ok(RustShellBuiltin::Cd),
            "pwd" => Ok(RustShellBuiltin::Pwd),
            _ => Err(()),
        }
    }
}

pub fn builtin_echo(args : &Vec<String>) -> Result<RustShellOutput, RustShellOutput> {
    Ok(RustShellOutput {
        code: Some(0),
        stdout: String::from(args.join(" ")).into_bytes(),
        stderr: String::from("").into_bytes(),
    })
}

// BUG: If rush_history doesn't exist yet, runtime panic.
pub fn builtin_history(_ : &Vec<String>) -> Result<RustShellOutput, RustShellOutput> {

    let f = File::open("rush_history").expect("rush_history not found");
    let reader = BufReader::new(f);
    let lines = reader.lines();

    let o = lines
        .enumerate()
        .map(|x| format!("{:3} {}", (x.0 + 1), x.1.unwrap()));

    println!("{:?}", o);

    Ok(RustShellOutput {
        code: Some(0),
        stdout: String::from("").into_bytes(),
        stderr: String::from("").into_bytes(),
    })
}

// FIXME: This doesn't actually do anything. Lol.
pub fn builtin_cd(args : &Vec<String>) -> Result<RustShellOutput, RustShellOutput> {
    warn!("Not yet implemented");
    // let new_dir = args.peekable().peek().map_or("/", |x| *x);
    let input = match args.get(0) {
        Some(i) => i,
        _ => "./"
    };

    let root = Path::new(input);
    match env::set_current_dir(&root) {
        Ok(_) => Ok(RustShellOutput {
            code: Some(0),
            stdout: String::from("").into_bytes(),
            stderr: String::from("").into_bytes(),
        }),
        Err(e) => Ok(RustShellOutput {
            code: Some(1),
            stdout: String::from("").into_bytes(),
            stderr: e.to_string().into_bytes(),
        })
    }
}

pub fn builtin_pwd(_ : &Vec<String>) -> Result<RustShellOutput, RustShellOutput> {
    match env::current_dir() {
        Ok(r) => Ok(RustShellOutput {
            code: Some(0),
            stdout: String::from(r.to_str().unwrap()).into_bytes(),
            stderr: String::from("").into_bytes(),
        }),
        Err(e) => Ok(
            RustShellOutput {
                code: Some(1),
                stdout: String::from("").into_bytes(),
                stderr: String::from(e.to_string()).into_bytes(),
            })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
