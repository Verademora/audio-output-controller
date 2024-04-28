use std::error::Error;
use std::io;
use std::process::{Command, Stdio};

#[derive(Clone, Debug, Default)]
struct Sink {
    index: u32,
    name: String,
}

#[derive(Clone, Debug, Default)]
struct SinkInput {
    index: u32,
    sink_index: u32,
    media: String,
    app: String,
}

fn collect_sink_inputs() -> Result<Vec<SinkInput>, Box<dyn Error>> {
    let pacmd = Command::new("pacmd")
        .arg("list-sink-inputs")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Something went wrong");

    let pacmd_out = pacmd.stdout.unwrap();

    let grep = Command::new("grep")
        .arg("-e")
        .arg("index:")
        .arg("-e")
        .arg("sink:")
        .arg("-e")
        .arg("media.name")
        .arg("-e")
        .arg("application.process.binary")
        .stdin(Stdio::from(pacmd_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Something went wrong with grep");

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
        .spawn()
        .expect("Something went wrong with awk");

    let output = awk.wait_with_output().expect("Failed to wait");
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.trim().split('\n').collect();
    let mut iter = lines.chunks_exact(4);
    let mut sink_inputs: Vec<SinkInput> = Vec::new();
    for chunk in &mut iter {
        if let [index, sink_index, media, app] = chunk {
            let input = SinkInput {
                index: index.parse().expect("Could not parse sink input index"),
                sink_index: sink_index
                    .parse()
                    .expect("Could not parse sink input current sink index"),
                media: media.trim().trim_matches('\"').to_owned(),
                app: app.trim().trim_matches('\"').to_owned(),
            };
            sink_inputs.push(input);
        }
    }
    Ok(sink_inputs)
}

fn collect_sinks() -> Result<Vec<Sink>, Box<dyn Error>> {
    let pacmd = Command::new("pacmd")
        .arg("list-sinks")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Something went wrong");

    let pacmd_out = pacmd.stdout.unwrap();

    let grep = Command::new("grep")
        .arg("-e")
        .arg("index:")
        .arg("-e")
        .arg("device.description")
        .stdin(Stdio::from(pacmd_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Something went wrong with grep");

    let grep_out = grep.stdout.unwrap();

    let awk = Command::new("awk")
        .arg(
            "{
            if ($1 == \"index:\") {
                print $2
            } else {
                $1=$2=\"\";
                print $0
            }
            }",
        )
        .stdin(Stdio::from(grep_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Something went wrong with awk");

    let output = awk.wait_with_output().expect("Failed to wait");
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.trim().split('\n').collect();
    let mut iter = lines.chunks_exact(2);
    let mut sinks: Vec<Sink> = Vec::new();
    for chunk in &mut iter {
        if let [index, name] = chunk {
            let input = Sink {
                index: index.parse().expect("Could not parse sink input index"),
                name: name.trim().trim_matches('\"').to_owned(),
            };
            sinks.push(input);
        }
    }
    Ok(sinks)
}

fn move_sinks(stream: String, sink: String) {
    let pacmd = Command::new("pacmd")
        .arg("move-sink-input")
        .arg(stream.trim())
        .arg(sink.trim())
        .spawn()
        .expect("Something went wrong");
    let output = pacmd.wait_with_output().expect("Failed to wait");
}

fn main() {
    // Collect our output devices
    let sinks = collect_sinks().unwrap();

    // Collect the audio streams
    let sink_inputs = collect_sink_inputs().unwrap();

    // Attempt to infer the active sink
    let first = sink_inputs.first().unwrap();
    let index = first.sink_index;
    let active_sink = sinks
        .clone()
        .into_iter()
        .find(|x| x.index == index)
        .unwrap();

    println!("====== Sinks ======");

    for sink in sinks {
        let index = sink.index;
        let name = sink.name;
        println!("{index} - {name}");
    }
    println!();
    println!("====== Streams ======");
    for input in sink_inputs {
        let index = input.index;
        let media = input.media;
        let app = input.app;
        println!("{index} - {app}");
        println!("> {media}");
    }
    println!();
    println!("Chose stream");

    let mut target_input = String::new();
    io::stdin()
        .read_line(&mut target_input)
        .expect("failed to read");

    println!();
    println!("Chose output");
    let mut target_output = String::new();
    io::stdin()
        .read_line(&mut target_output)
        .expect("failed to read");

    move_sinks(target_input, target_output);
}
