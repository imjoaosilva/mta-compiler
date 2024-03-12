use colored::*;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, Read, Write},
    path::Path,
    thread,
    time::Duration,
};
use std::fs;

struct Script {
    file_name: String,
    folder_path: String,
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
    }.trim().to_string();

    clearscreen::clear().expect("Failed to clear the screen");
    println!("{}", "Getting data...".cyan());

    println!("{}", format!("{}meta.xml",&custom_path));
    let hasmeta = Path::new(format!("{}meta.xml",&custom_path).as_str()).exists();
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
    
    let mut meta_content_str = fs::read_to_string(format!("{}meta.xml",&custom_path)).expect("Failed to read file")
    .trim()
    .replace("'", "\"");

    let file = OpenOptions::new()
        .read(true)
        .open(format!("{}meta.xml",&custom_path))
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
            let src = line.trim().split("src=").collect::<Vec<&str>>()[1];
            let filetype = src.split(".").collect::<Vec<&str>>()[1]
                .split("\"")
                .collect::<Vec<&str>>()[0];
            if filetype != "lua" {
                continue;
            }

            let script_src = src.split("\"").collect::<Vec<&str>>()[1];
            let path = Path::new(script_src);
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            let folder_path = path.parent().unwrap().to_str().unwrap();
            scripts.push(Script {
                file_name: file_name.to_string(),
                folder_path: folder_path.to_string(),
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

        let hasfile = Path::new(format!("{}{}",&custom_path, &script.script_src).as_str()).exists();
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
            .form(&[("luasource", format!("{}{}", &custom_path, &script.script_src))])
            .send();

        if response.is_err() {
            println!("{}", "Failed to connect to the server".red());
            continue;
        }

        let response = response.unwrap();

        let file = File::create(format!("{}{}{}.luac", custom_path, script.folder_path, script.file_name))
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

    println!("{}", "All files compiled".green());

    File::create(format!("{}meta.xml", &custom_path))
        .expect("Failed to create file")
        .write_all(meta_content_str.as_bytes())
        .expect("Failed to write file");
}
