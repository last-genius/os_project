use super::RustShellCommand;
use super::RustShellOutput;
use std::io::{self,Write};
use super::builtins::*;
use std::str::FromStr;
use std::process::{Command};
use std::fs::{OpenOptions};
use std::env;

// TODO: Figure out an intentional error code
pub fn append_to_history(c: String) {
    let file = OpenOptions::new().append(true).create(true).open("rush_history");
    match file {
        Ok(_) => {writeln!(file.unwrap(), "{}", format!("{}", c))},
        _ => Err(io::Error::from_raw_os_error(1))
    };
}

pub fn tokenize_command(c : String) -> RustShellCommand {
    let mut command_split : Vec<String> = c.trim().split_whitespace().map(|s| s.to_string()).collect();
    debug!("Split input: {:?}", command_split);

    match command_split.len() {
        0 => RustShellCommand { command : "".to_owned(), args : Vec::new()  },
        _ => RustShellCommand { command : command_split.remove(0), args : command_split},
    }
}

// TODO: Read prompt from an environment variable
pub fn print_prompt() {
    print!("{0}$", env::current_dir().unwrap().to_str().unwrap());
    io::stdout().flush().unwrap();
}

pub fn process_command(c : &RustShellCommand) -> Result<RustShellOutput, RustShellOutput> {
    match RustShellBuiltin::from_str(&c.command) {
        Ok(RustShellBuiltin::Echo) => builtin_echo(&c.args),
        Ok(RustShellBuiltin::History) => builtin_history(&c.args),
        Ok(RustShellBuiltin::Cd) => builtin_cd(&c.args),
        Ok(RustShellBuiltin::Pwd) => builtin_pwd(&c.args),

        _ => {
            match c.command.is_empty() {
                true => {
                    debug!("Empty command. Possibly Ctrl+D pressed");
                    Err(RustShellOutput {
                        code: Some(1),
                        stdout: String::from("").into_bytes(),
                        stderr: String::from("").into_bytes(),
                    })
                },
                false => execute_binary(&c),
            }
        },
    }
}

fn execute_binary(c : &RustShellCommand) -> Result<RustShellOutput, RustShellOutput> {
    // TODO: Maybe pipe stdout and stderr so print() will handle all i/o
    // Figure out how to make vim continue working
    //  .stdout(Stdio::piped())
    //  .stderr(Stdio::piped())
    let child = Command::new(&c.command)
        .args(&c.args)
        .spawn();

    match child {
        Ok(process) => {
          let output = process.wait_with_output().unwrap();
          Ok(RustShellOutput{code: output.status.code(), stdout: output.stdout, stderr: output.stderr})
        },
        Err(e) => {
            eprintln!("DEBUG: Exit code: {:?}", e.raw_os_error());
            Err(RustShellOutput {
                code: e.raw_os_error(),
                stdout: String::from("").into_bytes(),
                stderr: String::from(format!("rush: {}: command not found", &c.command)).into_bytes(),
            })
        },
    }
}

