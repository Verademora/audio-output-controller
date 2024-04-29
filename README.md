# Audio Output Controller
This is a work in progress application for helping me manage switching output devices in PulseAudio. This is the working proof of concept. There's probably a more nuanced way to call the PulseAudio server to do what this code does, but we instead rely on hard coded `grep` and `awk` commands from piped `pacmd` commands using the process crate in the standard library.
## How it works
In its current state, this program will attempt to collect the sinks and sink-inputs from PulseAudio and list them. It then takes user inputs to perform `pacmd move-sink-input`, not bothering to check that user input against the collected data or anything. I got very lazy towards the end of writing this first version.
## TODO
This program currently takes no arguments and this current version exists as a usable proof of concept that I intend to expand upon. Here is a generalized list of changes I intend to make:
1. Implement arguments for controlling how the program works.
2. Consider utilizing environment variables for user granular configuration.
3. Look more into PulseAudio and consider an alternative approach for how the program interacts with it.
4. A lot of cleanup.
## Contributions
This is a personal, and currently very developmental application. I am not currently seeking any contributions in this stage but you are welcome to fork what I have here and play around with the code for your own purposes.
