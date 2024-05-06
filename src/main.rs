#[macro_use]
mod macros;

use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{Clear, ClearType},
};
use serde::Deserialize;
use std::{
    fs::File,
    io::Write,
    process::{Command, Output},
};

#[derive(Deserialize)]
struct Config {
    phone_path: String,
    pc_path: String,
}

enum Destination {
    ToPC,
    ToPhone,
}

pub fn clear_terminal() {
    match execute!(
        std::io::stdout(),
        Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    ) {
        Ok(()) => (),
        Err(err) => eprintln!("Could not clear terminal. Error: {}", err),
    };
}

fn main() {
    let config: Config = settings();
    clear_terminal();
    check_conn();
    'outer: loop {
        println!(
            "{}[1] Phone -> PC\n[2] PC -> Phone\n[0] Exit",
            "Select Source -> Destination.\n".blue(),
        );
        print!("{}", "Type selection: ".blue());
        let input = scanln!();
        let destination = match input[..1].parse::<usize>() {
            Ok(destination) => match destination {
                0 => {
                    clear_terminal();
                    std::process::exit(0x0)
                }
                1 => Destination::ToPC,
                2 => Destination::ToPhone,
                _ => {
                    print_try_again!();
                    continue 'outer;
                }
            },
            Err(_) => {
                print_try_again!();
                continue 'outer;
            }
        };
        let list = match destination {
            Destination::ToPC => from_phone_to_pc(&config),
            Destination::ToPhone => from_pc_to_phone(&config),
        };
        let stdout = String::from_utf8_lossy(&list.stdout);
        let vec: Vec<&str> = stdout.lines().collect();
        clear_terminal();
        'middle: loop {
            println!("{}", "Select file to copy.".blue());
            vec.iter()
                .enumerate()
                .for_each(|(i, line)| println!("[{}] {}", i + 1, line));
            println!("[0] Back");
            print!("{}", "Type selection: ".blue());
            let input = scanln!();
            let selection: usize = match input.parse() {
                Ok(sel) => match sel {
                    0 => {
                        clear_terminal();
                        continue 'outer;
                    }
                    n if (1..=vec.len()).contains(&n) => sel,
                    _ => {
                        print_try_again!();
                        continue 'middle;
                    }
                },
                Err(_) => {
                    print_try_again!();
                    continue 'middle;
                }
            };
            let selected_save: &str = vec[selection - 1];
            clear_terminal();
            'inner: loop {
                let dest_str = match destination {
                    Destination::ToPhone => "Phone",
                    Destination::ToPC => "PC",
                };
                println!(
                    "Are you {} you want to copy file {} to {}?",
                    "sure".italic().bold().red(),
                    selected_save.yellow(),
                    dest_str.yellow()
                );

                print!("[1] Yes\n[2] No\n{}", "Type Selection: ".blue());
                let input = scanln!();
                match input.trim()[..1].parse::<usize>() {
                    Ok(choice) => match choice {
                        1 => {
                            copy_file(selected_save, &destination);
                            break 'middle;
                        }
                        2 => {
                            clear_terminal();
                            continue 'middle;
                        }
                        _ => {
                            print_try_again!();
                            continue 'inner;
                        }
                    },
                    Err(_) => {
                        print_try_again!();
                        continue 'inner;
                    }
                };
            }
        }
    }
}

fn from_phone_to_pc(data: &Config) -> Output {
    let output: Output = Command::new("adb.exe")
        .args(["shell", "ls", data.phone_path.as_str()])
        .output()
        .expect("Failed to execute command");
    output
}

fn from_pc_to_phone(data: &Config) -> Output {
    let arg = format!("ls -n '{}'", data.pc_path);
    let output: Output = Command::new("powershell.exe")
        .arg(arg)
        .output()
        .expect("Failed to execute command");
    output
}

fn copy_file(file: &str, opt: &Destination) {
    let data: Config = settings();
    match opt {
        Destination::ToPC => {
            let from_path: String = format!("{}{}", data.phone_path, file);
            let output: std::process::Output = Command::new("adb.exe")
                .args(["pull", &from_path, data.pc_path.as_str()])
                .output()
                .expect("Could not Copy file");
            print_output(output);
        }
        Destination::ToPhone => {
            let from_path: String = format!("{}{}", data.pc_path, file);
            let output: std::process::Output = Command::new("adb.exe")
                .args(["push", &from_path, data.phone_path.as_str()])
                .output()
                .expect("Could not Copy file");
            print_output(output);
        }
    }
}

fn print_output(output: std::process::Output) {
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        clear_terminal();
        print!("{}", stdout.green());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        clear_terminal();
        print!("{}", stdout.red());
        if !output.stderr.is_empty() {
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}

fn check_conn() {
    let output = Command::new("adb.exe")
        .arg("shell")
        .output()
        .expect("Could not check for phone");
    if output.status.success() {
    } else {
        println!("{}", "No phone connected. Terminating program...".red());
        std::process::exit(0x1);
    }
}

fn settings() -> Config {
    let file_path = format!(
        "{}/.config/savswapds/config.json",
        dirs::home_dir().unwrap().to_str().unwrap()
    );
    let file_dir = format!(
        "{}/.config/savswapds/",
        dirs::home_dir().unwrap().to_str().unwrap()
    );
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        let output: std::process::Output = Command::new("mkdir")
            .arg(file_dir)
            .output()
            .expect("Couldn't create config dir");
        if output.status.success() {
            let mut f = File::create(&file_path).expect("Could not create config file");
            print!("Type in the phone path (end with '/'): ");
            let input1 = scanln!();
            print!("Type in the PC path (end with '\\\\'): ");
            let input2 = scanln!();
            let config = format!(
                "{{\"phone_path\": \"{}\",\"pc_path\": \"{}\"}}",
                input1, input2
            );
            f.write_all(config.as_bytes())
                .expect("could not write standard config to file");
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            clear_terminal();
            print!("{}", stdout.red());
            if !output.stderr.is_empty() {
                eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }
    let f = std::fs::File::open(path).expect("couldn't open config");
    let rdr = std::io::BufReader::new(f);
    let data: Config = serde_json::from_reader(rdr).expect("couldn't read json");
    data
}
