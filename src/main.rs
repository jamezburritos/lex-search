use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::PathBuf;
use std::process::ExitCode;

use xml::reader::{EventReader, XmlEvent};

use search::Tokenizer;

pub fn read_file_to_string(path: PathBuf) -> Result<String, ()> {
    let file = File::open(path.clone()).map_err(|err| {
        eprintln!("ERROR: Failed to open file {}: {err}", path.display())
    })?;

    let extension = path
        .as_path()
        .extension()
        .and_then(OsStr::to_str);

    match extension {
        Some("xml") | Some("xhtml") => {
            Ok(EventReader::new(file)
                .into_iter()
                .filter_map(|event| {
                    if let XmlEvent::Characters(data) = event.unwrap() {
                        Some(data)
                    } else {
                        None
                    }
                })
                .fold(String::new(), |acc, x| acc + &x + " "))
        },

        _ => Err(
            eprintln!("WARN: unrecognised filetype: {}", extension.unwrap_or(""))
        ),
    }
}

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;
type ScoreIndex<'a> = HashMap<&'a PathBuf, f32>;

fn index_dir(dir_path: &str) -> Result<TermFreqIndex, ()> {
    let mut tf_index = TermFreqIndex::new();

    let dir = fs::read_dir("/home/jimbo/Documents/docs.gl/gl4").map_err(|err| {
        eprintln!("ERROR: Failed to read directory {}: {err}", dir_path)
    })?;

    for entry in dir {
        let file = entry.map_err(|err| {
            eprintln!("ERROR: Could not read file at {} during indexing: {err}", dir_path)
        })?;

        let content = match read_file_to_string(file.path()) {
            Ok(content) => content.chars().collect::<Vec<_>>(),
            Err(_) => continue
        };

        let mut tf = TermFreq::new();

        eprintln!("Indexing file {}...", file.path().display());

        for token in Tokenizer::new(&content) {
            let term: String = token.iter()
                .map(|x| x.to_ascii_lowercase())
                .collect();

            if let Some(term) = tf.get_mut(&term) {
                *term += 1;
            } else {
                tf.insert(term, 1);
            }
        }

        tf_index.insert(file.path(), tf);
    }

    eprintln!("Indexed {} files successfully.", tf_index.len());

    Ok(tf_index)
}

fn save_index(tf_index: TermFreqIndex, path: &str) -> Result<(), ()> { 
    let index_file = File::create(path).map_err(|err| {
        eprintln!("ERROR: Failed to create index.json: {err}")
    })?;

    serde_json::to_writer(index_file, &tf_index).map_err(|err| {
        eprintln!("ERROR: Failed to write to json: {err}")
    })?;

    eprintln!("Written to {path}.");

    Ok(())
}

fn load_index(path: &str) -> Result<TermFreqIndex, ()> {
    let file = File::open(path).map_err(|err| {
        eprintln!("ERROR: could not open index at {path}: {err}");
    })?;

    let tf_index: TermFreqIndex = serde_json::from_reader(file)
        .expect("serde should work");

    eprintln!("Loaded {} files from {path}", tf_index.len());

    Ok(tf_index)
}

fn tf_idf_search<'a>(tf_index: &'a TermFreqIndex, query: &str) -> ScoreIndex<'a> {
    let n_docs = tf_index.len() as f32;

    let mut scores: ScoreIndex = tf_index.iter()
        .map(|(path, _)| (path, 0_f32))
        .collect();

    for term in query.split_whitespace() {
        let matching_docs: HashMap<&PathBuf, usize> = tf_index.iter()
            .filter_map(|(path, tf)| {
                if let Some(freq) = tf.get(term) {
                    Some((path, *freq))
                } else {
                    None
                }
            })
            .collect();

        let n_matches = matching_docs.len() as f32;

        for (path, score) in scores.iter_mut() {
            if let Some(freq) = matching_docs.get(path) {
                *score += *freq as f32 * (n_docs / n_matches).ln()
            }
        } 
    }

    scores.into_iter().filter(|(_, x)| *x != 0_f32).collect()
}

fn usage(program: &str) {
    eprintln!("Usage: {program} <subcommand> <options>");
    eprintln!("Subcommands:");
    eprintln!("\tindex <dir> [index]\t\tindex the <dir> and save the index (default is index.json)");
    eprintln!("\tsearch <query> [index]\t\tsearch for <query>. data is loaded from [index] (default is index.json)")
}

fn entry() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("argv[0] is always present");

    let subcommand = args.next().ok_or_else(|| {
        eprintln!("ERROR: no subcommand provided");
        usage(&program);
    })?;

    match subcommand.as_str() {
        "index" => {
            let dir_path = args.next().ok_or_else(|| {
                eprintln!("ERROR: missing dir");
                usage(&program);
            })?;

            let index_path = args.next().unwrap_or("index.json".into());

            let tf_index = index_dir(&dir_path)?;
            save_index(tf_index, &index_path)?;
        },

        #[allow(unused)]
        "search" => {
            let query = args.next().ok_or_else(|| {
                eprintln!("ERROR: missing query");
                usage(&program);
            })?;

            let index_path = args.next().unwrap_or("index.json".into());
            let tf_index = load_index(&index_path)?;

            let scores = tf_idf_search(&tf_index, &query);

            let mut results: Vec<_> = scores.iter().collect();
            results.sort_by_key(|(_, score)| **score as u32);
    
            println!("Found {} results: ", scores.len());
            results.iter()
                .rev()
                .take(5)
                .for_each(|(path, _)| eprintln!("-> {path:?}"));
        }

        _ => {
            eprintln!("ERROR: unknown subcommand");
            usage(&program);
        }
    }
    
    Ok(())
}

fn main() -> ExitCode {
     match entry() {
         Ok(()) => ExitCode::SUCCESS,
         Err(()) => ExitCode::FAILURE
     }
}
