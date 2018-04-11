extern crate clap;
use clap::{Arg, App};

extern crate crossbeam;

use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

extern crate ansi_term;
use ansi_term::Colour::{Yellow, Green};

use std::collections::HashMap;

extern crate chrono;
use chrono::NaiveDate;

use std::path::Path;

extern crate url;
use url::Url;


#[derive(Debug)]
struct Node {
    key: String,
    value: String,
    line: i32,
    optional: bool
}

type HNode = HashMap<String, Node>;


fn read_ini<'a>(inp: &str, vrb: bool, sections: &'a mut Vec<String>) -> (Vec<Vec<HNode>>, &'a mut Vec<String>)
{
    let mut entries = Vec::new();
    let f = File::open(inp).expect("Cannot open file!");
    let mut file = BufReader::new(&f);
    let mut level = 0;
    let mut last_sec = String::new();
    let mut line1 = String::new();
    let mut line_num = 1;
    let mut entry_index = [0; 100];
    while file.read_line(&mut line1).unwrap() > 0 {
        //println!("READ {}", line);
        if line1.find('#').is_none() {
            let line = line1.trim();
            if line.starts_with('[') {
                let sec = line[1..line.len() - 1].trim();
                if sec.starts_with(&last_sec) && sec.len() > last_sec.len() {
                    if vrb {
                        println!("INCREASING level to {} ({} -> {})", level + 1, last_sec, sec);
                    }
                    level += 1;
                } else if sec != last_sec {
                    if vrb {
                        println!("DECREASING level to {} ({} -> {})", level, last_sec, sec);
                    }
                    level -= 1;
                }
                if vrb {
                    println!("Section {}, level {}", sec, level);
                }
                let num_points = sec.to_string().into_bytes().iter()
                    .filter(|&c| *c == b'.').count();
                if num_points != level - 1 {
                    eprintln!("{}", Yellow.paint(format!("• Section '{}' should be at level {} but is at level {}",
                                                      sec, level, num_points + 1)));
                }
                if entries.len() <= level - 1 {
                    entries.push(Vec::new());
                    sections.push(sec.to_string());
                    if vrb {
                        println!("ADDING a new level: {}", level);
                    }
                }
                entries[level - 1].push(HashMap::new());
                if vrb {
                    println!("NEW hashmap at level {}, length = {}", level, entries[level - 1].len());
                }
                entry_index[level - 1] += 1;
                last_sec = sec.to_string();
            } else if line.find('=').is_some() {
                let kv: Vec<&str> = line.split('=').map(|e| e.trim()).collect();
                if vrb {
                    println!("READING '{:?}'", kv);
                }
                let (k, opt) = if kv[0].ends_with('*') {
                    (&kv[0][0..kv[0].len() - 1], true)
                } else {
                    (kv[0], false)
                };
                debug_assert!(entries.len() > level - 1, format!("Entries size = {} <= {} = level - 1",
                                                              entries.len(), level - 1));
                debug_assert!(entry_index.len() > level - 1, format!("Entry indices size = {} <= {} = level - 1",
                                                                  entry_index.len(), level - 1));
                debug_assert!(entries[level - 1].len() > entry_index[level - 1] - 1,
                              format!("Entries size at level {} = {} <= entry index - 1 at level {} = level",
                                      level, entries[level - 1].len() >= entry_index[level - 1] - 1, level));
                entries[level - 1][entry_index[level - 1] - 1].insert(k.to_string(),
                                            Node { key: k.to_string(), value: kv[1].to_string(),
                                                line: line_num, optional: opt });
            }
        }
        line1.clear();
        line_num += 1
    };

    (entries, sections)
}

fn check_list(list: &str, type_: &str) -> Result<bool, String>
{
    if !list.starts_with('[') {
        return Err(format!("'{}' is not a well-formed list: missing starting brace", list))
    }
    if !list.ends_with(']') {
        return Err(format!("'{}' is not a well-formed list: missing ending brace", list))
    }
    if type_ == "Int" {
        for (num, e) in list[1..list.len() - 1].split(',').enumerate() {
            match e.parse::<i32>() {
                Ok(_) =>
                    return Ok(true),
                Err(_) =>
                    return Err(format!("the {} element in list '{}', '{}', is not a valid integer number",
                                       card(num + 1), list, e))
            }
        }
    }
    else if type_ == "Float" {
        for (num, e) in list[1..list.len() - 1].split(',').enumerate() {
            match e.parse::<f32>() {
                Ok(_) =>
                    return Ok(true),
                Err(_) =>
                    return Err(format!("the {} element in list '{}', '{}', is not a valid floating point number",
                                       card(num + 1), e, list))
            }
        }
    }

    Ok(true)
}

fn check_type(val: &str, type_: &str) -> Result<bool, String>
{
    match type_ {
        "Date" =>
            match NaiveDate::parse_from_str(val, "%-d/%-m/%Y") {
                Ok(_) =>
                    Ok(true),
                Err(_) =>
                    Err(format!("'{}' is not a valid date", val))
            },
        "Float" =>
            match val.parse::<f32>() {
                Ok(_) =>
                    Ok(true),
                Err(_) =>
                    Err(format!("'{}' is not a valid floating point number", val))
            },
        "Int" =>
            match val.parse::<i32>() {
                Ok(_) =>
                    Ok(true),
                Err(_) =>
                    Err(format!("'{}' is not a valid integer number", val))
            },
        "List[String]" =>
            check_list(val, "String"),
        "List[Int]" =>
            check_list(val, "Int"),
        "List[Float]" =>
            check_list(val, "Float"),
        "File" =>
            if Path::new(val).is_file() {
                Ok(true)
            }
            else {
                Err(format!("file '{}' does not exist", val))
            },
        "Directory" =>
            if Path::new(val).is_dir() {
                Ok(true)
            }
            else {
                Err(format!("directory '{}' does not exist", val))
            },
        "URL" => {
            match Url::parse(val) {
                Ok(_) =>
                    Ok(true),
                Err(_err) =>
                    Err(format!("URL '{}' is malformed", val))
            }
        },
        _ =>
            Ok(true)
    }
}

#[inline]
fn card(num: usize) -> String
{
    if num % 10 == 1 {
        "1st".to_string()
    }
    else if num % 10 == 2 {
        format!("{}nd", num)
    }
    else if num % 10 == 3 {
        format!("{}rd", num)
    }
    else {
        format!("{}th", num)
    }
}

fn check_typed_entries(sec: &str, vrb: bool, tmpl_entries: &HNode, entries: &[HNode]) -> Result<bool, Vec<String>>
{
    let mut errors = Vec::new();

    for (num, entries) in entries.into_iter().enumerate() {
        if vrb {
            println!("  ★ {} section '{}'", card(num + 1), sec);
        }
        for (tmpl_key, tmpl_value) in tmpl_entries {
            match entries.get(tmpl_key) {
                Some(value) => {
                    if vrb {
                        println!("    {}: {} = {:?} at line {}", tmpl_key, tmpl_value.value, value.value, value.line);
                    }
                    let chk = check_type(&value.value, &tmpl_value.value).map(|_|Ok(true))
                        .unwrap_or_else(|err| Err(Yellow.paint(format!("• Type error at line {}: {}!", value.line, err)).to_string()));
                    if chk.is_err() {
                        errors.push(chk.unwrap_err())
                    }
                },
                None if !tmpl_value.optional =>
                    errors.push(Yellow.paint(format!("• Missing key '{}' in the {} section '{}' (see line {} in the template)!",
                                                 tmpl_key, card(num + 1), sec, tmpl_value.line)).to_string()),
                None =>
                    ()
            }
        }
    }

    if errors.is_empty() {
        Ok(true)
    }
    else {
        Err(errors)
    }
}

fn check_entries(sec: &str, vrb: bool, tmpl_entries: &HNode, entries: &[HNode]) -> Result<bool, Vec<String>>
{
    let mut errors = Vec::new();

    for (num, entries1) in entries.into_iter().enumerate() {
        if vrb {
            println!("  ★ {} section '{}'", card(num + 1), sec);
        }
        for (key, value) in entries1 {
            match tmpl_entries.get(key) {
                Some(value) => {
                    if vrb {
                        println!("    {} = {:?} at line {}", key, value.value, value.line);
                    }
                    ()
                },
                None =>
                    errors.push(Yellow.paint(format!("• Spurious key '{}' at line {} in the {} section named '{}'!",
                                                 key, value.line, card(num + 1), sec)).to_string())
            }
        }
    };

    if errors.is_empty() {
        Ok(true)
    }
    else {
        Err(errors)
    }
}

fn real_main() -> i32
{
    let matches = App::new("INI validator")
        .version("0.1")
        .author("Author: M. Falda")
        .about("Validate an INI file according to a template")
        .arg(Arg::with_name("template")
            .short("t")
            .long("template")
            .value_name("TEMPLATE_INPUT")
            .required(true)
            .help("Sets the template file to use")
            .takes_value(true))
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .value_name("INPUT_FILE")
            .required(true)
            .help("Sets the INI file to validate")
            .takes_value(true))
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("Be verbose"))
        .get_matches();

    let vrb = matches.occurrences_of("verbose") > 0;

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    let tmpl = matches.value_of("template").expect("Cannot find the template file!");
    println!("\n*** Reading template file: {} ***", tmpl);

    let mut sections: Vec<String> = Vec::with_capacity(100);
    let (tmpl_entries, sections) = read_ini(tmpl, vrb, &mut sections);

    let inp = matches.value_of("input").expect("Cannot find the INI file!");
    println!("\n*** Reading input file: {} ***", inp);

    let (entries, sections) = read_ini(inp, vrb, sections);

    // check if all mandatory template entries are in the INI file and possibly their type

    println!();

    let mut num_errors = 0;

    crossbeam::scope(|scope| {
        for (s, (te, e)) in sections.iter().zip(tmpl_entries.iter().zip(&entries)) {
            println!("Checking section '{}'", s);
            let handle1: crossbeam::ScopedJoinHandle<Result<bool, Vec<String>>> = scope.spawn(move || {
                check_typed_entries(s, vrb, &te[0], e)
            });
            let handle2: crossbeam::ScopedJoinHandle<Result<bool, Vec<String>>> = scope.spawn(move || {
                check_entries(s, vrb, &te[0], e)
            });
            let res1: Result<bool, Vec<String>> = handle1.join();
            let res2: Result<bool, Vec<String>> = handle2.join();

            match res1 {
                Ok(_) =>
                    (),
                Err(err) => {
                    eprintln!("    {}", err.join("\n    "));
                    num_errors += err.len()
                }
            }

            match res2 {
                Ok(_) =>
                    println!("    {}", Green.paint("• the section is well-formed")),
                Err(err) => {
                    eprintln!("    {}", err.join("\n    "));
                    num_errors += err.len()
                }
            }
        }
    });

    if num_errors == 0 {
        println!("\nThe INI file is well-formed.\n");
        0
    }
    else {
        eprintln!("\nThere are {} errors!\n", num_errors);
        1
    }
}

fn main()
{
    let exit_code = real_main();
    std::process::exit(exit_code);
}
