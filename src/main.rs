use clap::Parser;
use std::error::Error;
use std::io;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    move_all: Option<String>,
    #[arg(short, long)]
    print: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Sink {
    index: u32,
    name: String,
    description: String,
    is_default: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SinkInput {
    index: u32,
    state: String,
    sink_index: u32,
    media: String,
    app: String,
}

// This function uses pacmd provided by PulseAudio to list the sink inputs to the stdout.
// It then pipes that output to grep and pipes that output to awk to clean the output.
// When finished, we collect all the inputs we were able to parse into a vector and return that.
fn collect_sink_inputs() -> Result<Vec<SinkInput>, Box<dyn Error>> {
    // TODO: We start and execute multiple commands and pipe stdin to stdout until we have a clean
    // enough output to parse. Consider replacing some commands by reading through and parsing the
    // output of pacmd ourselves.
    let pacmd = Command::new("pacmd")
        .arg("list-sink-inputs")
        .stdout(Stdio::piped())
        .spawn()?;

    let pacmd_out = pacmd.stdout.unwrap();

    let grep = Command::new("grep")
        .arg("-e")
        .arg("index:")
        .arg("-e")
        .arg("state:")
        .arg("-e")
        .arg("sink:")
        .arg("-e")
        .arg("media.name")
        .arg("-e")
        .arg("application.process.binary")
        .stdin(Stdio::from(pacmd_out))
        .stdout(Stdio::piped())
        .spawn()?;

    let grep_out = grep.stdout.unwrap();

    let awk = Command::new("awk")
        .arg(
            "{
            if ($2 == \"=\") {
                $1=$2=\"\";
                print $0
            } else {
                print $2
            }
        }",
        )
        .stdin(Stdio::from(grep_out))
        .stdout(Stdio::piped())
        .spawn()?;

    let output = awk.wait_with_output().expect("Failed to wait");
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.trim().split('\n').collect();
    let mut iter = lines.chunks_exact(5);
    let mut sink_inputs: Vec<SinkInput> = Vec::new();
    for chunk in &mut iter {
        if let [index, state, sink_info, media, app] = chunk {
            let input = SinkInput {
                index: index
                    .trim()
                    .parse()
                    .expect("Could not parse sink input index"),
                sink_index: sink_info
                    .trim()
                    .parse()
                    .expect("Could not parse active sink index"),
                state: state.trim().parse().expect("Could not parse state"),
                media: media.trim().trim_matches('\"').to_owned(),
                app: app.trim().trim_matches('\"').to_owned(),
            };
            sink_inputs.push(input);
        }
    }
    Ok(sink_inputs)
}

// This function uses pacmd provided by PulseAudio to list the sinks to the stdout.
// It then pipes that output to grep and pipes that output to awk to clean the output.
// When finished, we collect all the Sinks we were able to parse into a vector and return that.
fn collect_sinks() -> Result<Vec<Sink>, Box<dyn Error>> {
    // TODO: We start and execute multiple commands and pipe stdin to stdout until we have a clean
    // enough output to parse. Consider replacing some commands by reading through and parsing the
    // output of pacmd ourselves.
    let pacmd = Command::new("pacmd")
        .arg("list-sinks")
        .stdout(Stdio::piped())
        .spawn()?;

    let pacmd_out = pacmd.stdout.unwrap();

    let grep = Command::new("grep")
        .arg("-e")
        .arg("index:")
        .arg("-e")
        .arg("name:")
        .arg("-e")
        .arg("device.description")
        .stdin(Stdio::from(pacmd_out))
        .stdout(Stdio::piped())
        .spawn()?;

    let grep_out = grep.stdout.unwrap();

    let awk = Command::new("awk")
        .arg(
            "{
            if ($2 == \"index:\") {
                $2=\"\";
                print $0
            } else if ($3) {
                $1=$2=\"\";
                print $0
            } else {
                print $2
            }
            }",
        )
        .stdin(Stdio::from(grep_out))
        .stdout(Stdio::piped())
        .spawn()?;

    let output = awk.wait_with_output().expect("Failed to wait");
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.trim().split('\n').collect();
    let mut iter = lines.chunks_exact(3);
    let mut sinks: Vec<Sink> = Vec::new();
    for chunk in &mut iter {
        if let [index_raw, name, description] = chunk {
            let mut index_split: Vec<&str> = index_raw.trim().split(' ').collect();
            let mut is_default = false;
            let index: u32 = index_split
                .pop()
                .unwrap()
                .parse()
                .expect("Could not parse index");
            if !index_split.is_empty() {
                is_default = true;
            }
            let input = Sink {
                index,
                is_default,
                name: name
                    .trim()
                    .trim_matches(|c| c == '<' || c == '>')
                    .to_owned(),
                description: description.trim().trim_matches('\"').to_owned(),
            };
            sinks.push(input);
        }
    }
    Ok(sinks)
}

fn move_sinks(sink_input: &SinkInput, sink: &Sink) {
    let pacmd = Command::new("pacmd")
        .arg("move-sink-input")
        .arg(sink_input.index.to_string())
        .arg(sink.index.to_string())
        .spawn()
        .expect("Something went wrong");
    let _output = pacmd.wait_with_output().expect("Failed to wait");
}

fn set_default_sink(sink: &Sink) {
    let pacmd = Command::new("pacmd")
        .arg("set-default-sink")
        .arg(sink.index.to_string())
        .spawn()
        .expect("Something went wrong");
    let _output = pacmd.wait_with_output().expect("Failed to wait");
}

// TODO: This is a placeholder function. Consider cleaning this function
// for actual use or removing it when no longer needed
fn print_sinks(sinks: &[Sink]) {
    println!("====== Sinks ======");
    for sink in sinks {
        let index = sink.index;
        let name = &sink.name;
        let description = &sink.description;
        let is_default = sink.is_default;
        if is_default {
            println!("* Index: {index}");
            println!("    {description} - <{name}>");
        } else {
            println!("  Index: {index}");
            println!("    {description} - <{name}>");
        }
    }
    println!();
}

// TODO: This is a placeholder function. Consider cleaning this function
// for actual use or removing it when no longer needed
fn print_sink_inputs(sink_inputs: &[SinkInput]) {
    println!("====== Inputs ======");
    let mut apps: Vec<String> = Vec::new();
    let mut inputs_clone = sink_inputs.to_owned();
    inputs_clone.sort_by(|a, b| a.app.partial_cmp(&b.app).unwrap());
    for input in inputs_clone {
        let index = input.index;
        let media = &input.media;
        let app = &input.app;
        if let Some(_app) = apps.last() {
            if _app != app {
                apps.push(app.to_owned());
                println!();
                println!("{app}");
            }
        } else {
            apps.push(app.to_owned());
            println!("{app}");
        }
        println!("  {index} -> \"{media}\"");
    }
    println!();
}

fn move_all_next() {
    let sinks = collect_sinks().unwrap();
    let sink_inputs = collect_sink_inputs().unwrap();
    let mut sinks_iter = sinks.iter();
    let _default = sinks_iter.find(|x| x.is_default).unwrap();
    match sinks_iter.next() {
        Some(next) => {
            sink_inputs.iter().for_each(|x| move_sinks(x, next));
            set_default_sink(next);
        }
        None => {
            let first = sinks.first().unwrap();
            sink_inputs.iter().for_each(|x| move_sinks(x, first));
            set_default_sink(first);
        }
    }
}

fn move_all_default() {
    let sinks = collect_sinks().unwrap();
    let sink_inputs = collect_sink_inputs().unwrap();
    let mut sinks_iter = sinks.iter();
    let default = sinks_iter.find(|x| x.is_default).unwrap();
    sink_inputs.iter().for_each(|x| move_sinks(x, default));
}

fn cli_prompt() {
    let sinks = collect_sinks().unwrap();
    let sink_inputs = collect_sink_inputs().unwrap();
    print_sink_inputs(&sink_inputs);
    print_sinks(&sinks);

    println!("Chose Sink Input");
    let mut user_sink_input_raw = String::new();
    io::stdin()
        .read_line(&mut user_sink_input_raw)
        .expect("failed to read");
    let user_sink_input = user_sink_input_raw
        .trim()
        .parse()
        .expect("Could not parse user input");
    let sink_input = sink_inputs
        .clone()
        .into_iter()
        .find(|x| x.index == user_sink_input)
        .expect("Could not find the referenced sink input");
    println!();

    println!("Chose Sink");
    let mut user_sink_raw = String::new();
    io::stdin()
        .read_line(&mut user_sink_raw)
        .expect("failed to read");
    let user_sink = user_sink_raw
        .trim()
        .parse()
        .expect("Could not parse user input");
    let sink = sinks
        .clone()
        .into_iter()
        .find(|x| x.index == user_sink)
        .expect("Could not find the referenced sink input");

    move_sinks(&sink_input, &sink);
}

fn main() {
    let cli = Cli::parse();
    let sinks = collect_sinks().unwrap();
    if sinks.is_empty() {
        println!("No audio devices detected");
        return;
    }

    let mut no_command = true;

    if let Some(opt) = cli.print {
        no_command = false;
        if &opt == "default" || &opt == "d" {
            let default = sinks.iter().find(|x| x.is_default).unwrap();
            println!("{}", default.description);
        }
    }
    if let Some(opt) = cli.move_all {
        no_command = false;
        if &opt == "next" || &opt == "n" {
            move_all_next();
        } else if &opt == "default" || &opt == "d" {
            move_all_default();
        }
    }
    if no_command {
        cli_prompt();
    }
}
