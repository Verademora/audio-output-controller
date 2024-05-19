use clap::Parser;
use std::error::Error;
use std::io;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    next: bool,

    #[arg(short, long)]
    debug: bool,
}

#[derive(Clone, Debug, Default)]
struct Sink {
    index: u32,
    name: String,
    description: String,
    is_default: bool,
}

#[derive(Clone, Debug, Default)]
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

fn move_sinks(sink_input: SinkInput, sink: Sink) {
    let pacmd = Command::new("pacmd")
        .arg("move-sink-input")
        .arg(sink_input.index.to_string())
        .arg(sink.index.to_string())
        .spawn()
        .expect("Something went wrong");
    let _output = pacmd.wait_with_output().expect("Failed to wait");
    set_default_sink(sink);
}

fn set_default_sink(sink: Sink) {
    let pacmd = Command::new("pacmd")
        .arg("set-default-sink")
        .arg(sink.index.to_string())
        .spawn()
        .expect("Something went wrong");
    let _output = pacmd.wait_with_output().expect("Failed to wait");
}

// TODO: This is a placeholder function. Consider cleaning this function
// for actual use or removing it when no longer needed
fn print_sinks(sinks: &Vec<Sink>) {
    println!("====== Sinks ======");
    for sink in sinks {
        let index = sink.index;
        let name = &sink.name;
        println!("{index} - {name}");
    }
    println!();
}

// TODO: This is a placeholder function. Consider cleaning this function
// for actual use or removing it when no longer needed
fn print_sink_inputs(sink_inputs: &Vec<SinkInput>) {
    println!("====== Streams ======");
    for input in sink_inputs {
        let index = input.index;
        let media = &input.media;
        let app = &input.app;
        println!("{index} - {app}");
        println!("> {media}");
    }
    println!();
}

fn cli_prompt(sinks: Vec<Sink>, sink_inputs: Vec<SinkInput>) {
    print_sink_inputs(&sink_inputs);
    print_sinks(&sinks);

    println!("Chose Sink Stream");
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

    move_sinks(sink_input, sink);
}

fn main() {
    let cli = Cli::parse();
    // Collect our output devices
    let sinks = collect_sinks().unwrap();

    // Collect the audio streams
    let sink_inputs = collect_sink_inputs().unwrap();

    let _default_sink = sinks.clone().into_iter().find(|x| x.is_default).unwrap();

    if cli.next {
        todo!();
    } else {
        cli_prompt(sinks, sink_inputs);
    }
}
