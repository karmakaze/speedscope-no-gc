use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;
use serde::Deserialize;
use serde::Serialize;
use gcollections::ops::*;
extern crate interval;
use crate::interval::interval_set::*;

#[derive(Debug, Deserialize, Serialize)]
struct Speedscope {
    exporter: String,
    name: String,
    #[serde(rename = "activeProfileIndex")]
    active_profile_index: usize,
    #[serde(rename = "$schema")]
    schema: String,
    shared: Shared,
    profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Shared {
    frames: Vec<Frame>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Frame {
    name: String,
    file: String,
    line: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Profile {
    #[serde(rename = "type")]
    ptype:       String,
    name:        String,
    unit:        String,
    #[serde(rename = "startValue")]
    start_value: usize,
    #[serde(rename = "endValue")]
    end_value:   usize,
    events:      Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Event {
    #[serde(rename = "type")]
    etype: EType,
    frame: usize,
    at:    usize,
}

#[derive(Debug, Deserialize, Serialize)]
enum EType {
    O,
    C,
}

fn main() {
    let input_filename = "20220329_095344_wall_e7518f1d_4629_4867_852e_284beb63ef9c_web_7d69bf8f49_ls5xl.speedscope.json";

    let content = fs::read_to_string(input_filename)
        .expect(format!("Unable to read the file \"{input_filename}\"").as_str());
    let input = content.as_str();

    let speedscope: Speedscope = serde_json::from_str(&input).unwrap();
    // println!("deserialized = {:?}", speedscope);

    let mut gc_names: HashSet<String> = HashSet::new();
    gc_names.insert("(garbage collection)".to_string());
    gc_names.insert("(marking)".to_string());
    gc_names.insert("(sweeping)".to_string());

    for profile in speedscope.profiles {
        println!("profile: {:?}..{:?}", profile.start_value, profile.end_value);

        let mut profile_intervals: IntervalSet<usize> = vec![(0, 1)].to_interval_set();

        let mut open_events: HashMap<usize, Event> = HashMap::new();
        for event in profile.events {
            match event.etype {
                EType::O => {
                    let frame = &speedscope.shared.frames[event.frame];
                    if !gc_names.contains(&frame.name) {
                        // println!("Insert event {:?}", &event);
                        open_events.insert(event.frame, event);
                    }
                },
                EType::C => {
                    match open_events.remove(&event.frame) {
                        Some(open_event) => {
                            let interval = vec![(open_event.at, event.at)].to_interval_set();
                            profile_intervals = profile_intervals.union(&interval);
                        },
                        _ => {
                            // println!("Did not find event 'O' for {:?}", event);
                            // println!("open_events {:?}", &open_events);
                        },
                    }
                },
            }
        }
        println!("intervals: {:?}", &profile_intervals);
    }
}
