use clap::Parser;
use itertools::Itertools;
use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
use std::{fs, io::BufRead, result::Result, thread};
use walkdir::WalkDir;

#[allow(clippy::struct_excessive_bools)]
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

    #[arg(
        long,
        help = "Parser only checks the file for errors, without formatting it"
    )]
    check: bool,

    #[arg(
        long,
        help = "Allow parsing to be lossy, replacing invalid chars with �"
    )]
    lossy: bool,
}

fn main() {
    stderrlog::new()
        // .modules(vec!["ksp-cfg-formatter"])
        .verbosity(log::Level::Error)
        .init()
        .unwrap();
    // Read CLI arguments
    let args = Args::parse();

    // Read input from either a path or stdin if no path is provided
    if let Some(path) = &args.path {
        let paths = files_from_path(path);
        let mut workers = vec![];
        for path in paths {
            let args = args.clone();
            let worker = thread::spawn(move || {
                let mut res = vec![];
                let text = if args.lossy {
                    let raw = fs::read(&path).map_or_else(|err| panic!("{err}"), |t| t);
                    String::from_utf8_lossy(&raw).to_string()
                } else {
                    fs::read_to_string(&path)
                        .map_or_else(|_| panic!("Failed to read text from {path}"), |t| t)
                };
                if args.check {
                    match ksp_cfg_formatter::parse_to_ast(&text) {
                        Ok(doc) => match ksp_cfg_formatter::transformer::assignments_first(doc) {
                            Ok(_) => (),
                            Err(_err) => {
                                // res.push(format!("{path}\n{_err}"))
                            }
                        },
                        Err(errs) => {
                            for err in errs {
                                use ksp_cfg_formatter::parser::nom::Severity as sev;
                                if matches!(
                                    err.severity,
                                    sev::Error //| sev::Warning
                                ) {
                                    res.push(format!("{}\n{}", path, err));
                                }
                            }
                        }
                    };
                } else {
                    format_file(&args, &text, Some(path.clone()));
                }
                res
            });
            workers.push(worker);
        }
        let mut res = vec![];
        for worker in workers {
            res.append(&mut worker.join().expect("Thread failed to join"));
        }
        println!("{}", res.iter().format("\n\n\n"));
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

/// Generates a Vec of all the paths to ksp cfg files in a `GameData` folder
fn files_from_path(path: &String) -> Vec<String> {
    let mut paths = Vec::new();
    for path in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let name = path.path().to_owned();
        if let Some(extension) = name.extension() {
            if extension == "cfg"
                && name
                    .canonicalize()
                    .unwrap()
                    .ancestors()
                    .any(|n| n.ends_with("GameData"))
            {
                if let Some(name) = name.to_str() {
                    paths.push(name.to_owned());
                };
            }
        }
    }
    paths
}
