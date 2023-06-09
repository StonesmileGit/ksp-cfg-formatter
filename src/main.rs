use clap::Parser;
use ksp_cfg_formatter::{ast_formatter::Formatter, Indentation, LineReturn};
use std::{fs, io::BufRead, result::Result, thread};
use walkdir::WalkDir;

#[derive(Parser, Debug, Clone)]
#[command(author, about, long_about = None)]
struct Args {
    #[arg(
        long,
        help = "file to format. If a folder is provided, all containing files are formatted.
If no path is provided, text is read from stdin."
    )]
    path: Option<String>,

    #[arg(
        long,
        help = "Collapses blocks that only take up one line and are short enough"
    )]
    inline: bool,

    #[arg(
        long,
        help = "Number of spaces used for indentation. Tabs are used if not set"
    )]
    indentation: Option<usize>,

    #[arg(
        long,
        help = "Prints output to stdout instead of writing back to file when reading from path"
    )]
    stdout: bool,
}

fn main() {
    // Read CLI arguments
    let args = Args::parse();

    // Read input from either a path or stdin if no path is provided
    if let Some(path) = &args.path {
        let paths = files_from_path(path);
        let mut workers = vec![];
        for path in paths {
            let args = args.clone();
            let worker = thread::spawn(move || {
                let text = fs::read_to_string(&path)
                    .map_or_else(|_| format!("Failed to read text from {path}"), |t| t);
                format_file(&args, &text, Some(path.clone()));
            });
            workers.push(worker);
        }
        for worker in workers {
            worker.join().expect("Thread failed to join");
        }
    } else {
        let mut text: String = String::new();
        // Collect multi-line input from stdin
        let input = std::io::stdin().lock().lines().flatten();
        for line in input {
            text.push_str(&line);
            text.push('\n');
        }
        format_file(&args, &text, args.path.clone());
    }
}

fn format_file(args: &Args, text: &str, path: Option<String>) {
    // Set up formatter and use it to format the text
    let indentaion = Indentation::from(args.indentation);
    let formatter = Formatter::new(indentaion, args.inline, LineReturn::Identify);
    let output = formatter.format_text(text);

    // write output to path or stdout
    match (args.stdout, path) {
        (false, Some(path)) => {
            let _res = fs::write(path, output);
        }
        _ => {
            print!("{output}");
        }
    }
}

fn files_from_path(path: &String) -> Vec<String> {
    let mut paths = Vec::new();
    for path in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let name = path.path().to_owned();
        if let Some(extension) = name.extension() {
            if extension == "cfg" {
                if let Some(name) = name.to_str() {
                    paths.push(name.to_owned());
                };
            }
        }
    }
    paths
}
