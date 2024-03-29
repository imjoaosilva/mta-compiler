use colored::*;
use std::fs;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, Read, Write},
    path::Path,
    thread,
    time::Duration,
};

struct Script {
    script_src: String,
    line_src: String,
}

fn main() {
    let mut result = String::new();
    let mut compiled = false;

    while !compiled {
        compiled = true;
        println!("{}", "Is the project on this path? (y/n) :".cyan());
        io::stdin()
            .read_line(&mut result)
            .expect("Failed to read line");
        match result.trim() {
            "y" => {
                println!("{}", "Great! Let's compile the project".green());
                thread::sleep(Duration::from_secs(1));
                compile(result.as_str().trim());
                break;
            }
            "n" => {
                println!("{}", "Enter the path of the project:".cyan());
                let mut path = String::new();
                io::stdin()
                    .read_line(&mut path)
                    .expect("Failed to read line");

                println!("{}", "Great! Let's compile the project".green());
                thread::sleep(Duration::from_secs(1));
                compile(path.as_str().trim());
                break;
            }
            _ => {
                println!("{}", "Invalid input".red());
                continue;
            }
        }
    }
}

fn compile(result: &str) {
    let custom_path = if result == "y" {
        String::from("")
    } else {
        format!("{}\\", result)
    }
    .trim()
    .to_string();

    clearscreen::clear().expect("Failed to clear the screen");
    println!("{}", "Getting data...".cyan());

    println!("{}", format!("{}meta.xml", &custom_path));
    let hasmeta = Path::new(format!("{}meta.xml", &custom_path).as_str()).exists();
    if !hasmeta {
        clearscreen::clear().expect("Failed to clear the screen");
        println!(
            "{} {} ",
            "[ERROR] -> File `meta.xml` not found.".bright_red(),
            "The Path is correct?".red()
        );
        thread::sleep(Duration::from_secs(2));
        panic!();
    };

    let mut meta_content_str = fs::read_to_string(format!("{}meta.xml", &custom_path))
        .expect("Failed to read file")
        .trim()
        .replace("'", "\"");

    let file = OpenOptions::new()
        .read(true)
        .open(format!("{}meta.xml", &custom_path))
        .expect("Failed to open file");

    let mut reader: io::BufReader<&File> = io::BufReader::new(&file);
    let reader = reader.by_ref();

    let mut scripts: Vec<Script> = Vec::new();
    for line in reader.lines() {
        let line = line
            .expect("Failed to read line")
            .trim()
            .to_string()
            .replace("'", "\"");

        if line.contains("src") {
            
            let filtered_line = line.replace(" ", "");
            let src = filtered_line.trim().split("src=").collect::<Vec<&str>>()[1];
            let filetype = src.split(".").collect::<Vec<&str>>()[1]
                .split("\"")
                .collect::<Vec<&str>>()[0];
            if filetype != "lua" {
                continue;
            }

            let script_src = src.split("\"").collect::<Vec<&str>>()[1];

            scripts.push(Script {
                script_src: script_src.to_string(),
                line_src: line,
            });
        }
    }

    clearscreen::clear().expect("Failed to clear the screen");

    if scripts.len() == 0 {
        println!("{}", "No lua files found".red());
        thread::sleep(Duration::from_secs(2));
        panic!();
    }

    for script in &scripts {
        println!("{} {}", "Getting...".cyan(), script.script_src);

        let hasfile =
            Path::new(format!("{}{}", &custom_path, &script.script_src).as_str()).exists();
        if !hasfile {
            println!(
                "{} {}",
                "[ERROR] -> File not found.".bright_red(),
                "The Path is correct?".red()
            );
            continue;
        };

        let response = reqwest::blocking::Client::new()
            .post("https://luac.multitheftauto.com/?compile=1&debug=0&obfuscate=2")
            .multipart(
                reqwest::blocking::multipart::Form::new()
                    .file(
                        "luasource",
                        format!("{}{}", &custom_path, &script.script_src),
                    )
                    .unwrap(),
            )
            .send();

        if response.is_err() {
            println!("{}", "Failed to connect to the server".red());
            continue;
        }

        let response = response.unwrap();

        let file = File::create(format!(
            "{}{}",
            custom_path, script.script_src.replace("lua", "luac")
        ))
        .expect("Failed to create file");

        let mut writer = io::BufWriter::new(file);

        let content = response.text().unwrap();

        writer
            .write_all(content.as_bytes())
            .expect("Failed to write file");

        println!("{}", "Compiled".green());

        let new_line = script.line_src.replace(".lua", ".luac");
        meta_content_str = meta_content_str.replace(&script.line_src, &new_line);
    }

    clearscreen::clear().expect("Failed to clear the screen");
    println!("{}", "All files compiled".green());

    File::create(format!("{}meta.xml", &custom_path))
        .expect("Failed to create file")
        .write_all(meta_content_str.as_bytes())
        .expect("Failed to write file");

    thread::sleep(Duration::from_secs(2));
}
