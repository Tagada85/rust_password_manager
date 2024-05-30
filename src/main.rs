use clap::{Parser, Subcommand};
use colored::Colorize;
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use passwords::analyzer;
use passwords::scorer;
use passwords::PasswordGenerator;
use std::io;
use std::io::Write;
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Add {
        #[arg(short, long)]
        service: String,
        #[arg(short, long)]
        username: String,
        #[arg(short, long, default_missing_value("true"))]
        clipboard: bool, // copy from clipboard
        #[arg(short, long, default_missing_value("true"))]
        generate: bool, // generate new password
        #[arg(short, long, default_missing_value("true"))]
        write: bool, // type new password
    },
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    match args.cmd {
        Commands::List => display_passwords()?,
        Commands::Add {
            service,
            username,
            generate,
            clipboard,
            write,
        } => handle_new_password(service, username, generate, clipboard, write),
    }
    Ok(())
}

fn handle_new_password(
    service: String,
    username: String,
    generate: bool,
    clipboard: bool,
    write: bool,
) {
    if generate {
        let password = generate_password();
        let _ = add_new_password(&service, &username, &password);
    }
    if clipboard {
        let mut ctx = ClipboardContext::new().unwrap();
        println!("Let's get what you have in the clipboard");
        let password_in_clipboard = ctx.get_contents().unwrap();
        if should_save_password(&password_in_clipboard) {
            let _ = add_new_password(&service, &username, &password_in_clipboard);
        }
    }
    if write {
        let password = handle_write_password().ok().unwrap();
        if should_save_password(&password) {
            let _ = add_new_password(&service, &username, &password);
        }
    }
}

fn generate_password() -> String {
    PasswordGenerator::new()
        .length(16)
        .numbers(true)
        .uppercase_letters(true)
        .symbols(true)
        .spaces(true)
        .exclude_similar_characters(true)
        .strict(true)
        .generate_one()
        .unwrap()
}

fn should_save_password(password: &str) -> bool {
    if is_password_weak(password) {
        println!(
            "{} {}",
            password.red(),
            "is a weak password. Press Enter to continue anyway. Press Q to abort and try again!"
                .red()
        );
        if let Ok(choice) = read_next_char() {
            return choice == WeakPasswordChoice::CONTINUE;
        } else {
            return false;
        }
    } else {
        return true;
    }
}

fn is_password_weak(password: &str) -> bool {
    let score = scorer::score(&analyzer::analyze(password));
    return score < 80.0;
}

fn handle_write_password() -> io::Result<String> {
    println!("Write it down!");
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => return Ok(input),
        Err(error) => Err(error),
    }
}

#[derive(PartialEq)]
enum WeakPasswordChoice {
    ABORT,
    CONTINUE,
}

fn read_next_char() -> io::Result<WeakPasswordChoice> {
    enable_raw_mode()?;
    let result = loop {
        match read()? {
            Event::Key(event) => {
                // Handle key press
                match event.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        break Ok(WeakPasswordChoice::ABORT);
                    }
                    KeyCode::Enter => {
                        break Ok(WeakPasswordChoice::CONTINUE);
                    }
                    _ => {
                        println!("You pressed: {:?}", event.code);
                    }
                }
            }
            _ => {}
        }
    };

    disable_raw_mode()?;
    result
}

fn display_passwords() -> std::io::Result<()> {
    let path = Path::new("./passwords.txt");
    let contents = fs::read_to_string(path).expect("Could not read the passwords file");

    println!("{}", contents);
    Ok(())
}

fn add_new_password(service: &str, username: &str, password: &str) -> std::io::Result<()> {
    let path = Path::new("./passwords.txt");
    let password_infos = format!("{}|{}|{}\n", service, username, password);

    let mut file = OpenOptions::new().append(true).open(path)?;

    file.write_all(password_infos.as_bytes())?;

    Ok(())
}
