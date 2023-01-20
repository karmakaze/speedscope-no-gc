use std::fs;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Shared {
    frames: Vec<Frame>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct Event {
    #[serde(rename = "type")]
    etype: EType,
    frame: usize,
    at:    usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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

    let mut out: Speedscope = Speedscope {
        exporter: speedscope.exporter,
        name: speedscope.name,
        active_profile_index: speedscope.active_profile_index,
        schema: speedscope.schema,
        shared: speedscope.shared.clone(),
        profiles: vec![],
    };

    for profile in speedscope.profiles {
        // println!("profile: {:?}..{:?}", profile.start_value, profile.end_value);

        let mut gc_intervals: IntervalSet<usize> = vec![].to_interval_set();
        let mut gc_intervals_by_open: BTreeMap<usize, usize> = BTreeMap::new();
        let mut gc_intervals_by_close: BTreeMap<usize, usize> = BTreeMap::new();

        fill_gc_intervals(&profile, &speedscope.shared.frames, &mut gc_intervals,
            &mut gc_intervals_by_open, &mut gc_intervals_by_close);

        let mut out_profile = Profile {
            ptype:       profile.ptype.clone(),
            name:        profile.name.clone(),
            unit:        profile.unit.clone(),
            start_value: profile.start_value,
            end_value:   profile.end_value,
            events:      vec![],
        };

        span_events(&mut out_profile, &profile, &speedscope.shared.frames, &gc_intervals,
            &gc_intervals_by_open, &gc_intervals_by_close);

        out.profiles.push(out_profile);
    }

    let json = serde_json::to_string(&out).unwrap();
    println!("{}", json);
}

fn span_events<'a>(out_profile: &'a mut Profile, profile: &Profile, frames: &Vec<Frame>,
    _gc_intervals: &IntervalSet<usize>, _gc_intervals_by_open: &BTreeMap<usize, usize>,
    _gc_intervals_by_close: &BTreeMap<usize, usize>,
) {
    let mut gc_names: HashSet<String> = HashSet::new();
    gc_names.insert("(garbage collection)".to_string());
    gc_names.insert("(marking)".to_string());
    gc_names.insert("(sweeping)".to_string());

    let mut events: Vec<Event> = Vec::new();

    let mut _close_event_by_frame_close: HashMap<(usize, usize), Event> = HashMap::new();
    for event in profile.events.iter() {
        match event.etype {
            EType::O => {
                let frame = &frames[event.frame];
                if !gc_names.contains(&frame.name) {
                    events.push(*event);
                }
            },
            EType::C => {
                let frame = &frames[event.frame];
                if !gc_names.contains(&frame.name) {
                    events.push(*event);
                }
            },
        }
    }

    for event in events {
        out_profile.events.push(event);
    }
}


fn fill_gc_intervals(profile: &Profile, frames: &Vec<Frame>, gc_intervals: &mut IntervalSet<usize>,
    gc_intervals_by_open: &mut BTreeMap<usize, usize>, gc_intervals_by_close: &mut BTreeMap<usize, usize>
) {
    let mut gc_names: HashSet<String> = HashSet::new();
    gc_names.insert("(garbage collection)".to_string());
    gc_names.insert("(marking)".to_string());
    gc_names.insert("(sweeping)".to_string());

    let mut open_events: HashMap<usize, Event> = HashMap::new();
    for event in profile.events.iter() {
        match event.etype {
            EType::O => {
                let frame = &frames[event.frame];
                if gc_names.contains(&frame.name) {
                    // println!("Insert event {:?}", &event);
                    open_events.insert(event.frame, *event);
                }
            },
            EType::C => {
                match open_events.remove(&event.frame) {
                    Some(open_event) => {
                        let interval = vec![(open_event.at, event.at)].to_interval_set();
                        *gc_intervals = gc_intervals.union(&interval);
                        gc_intervals_by_open.insert(open_event.at, event.at);
                        gc_intervals_by_close.insert(event.at, open_event.at);
                    },
                    _ => {
                        // println!("Did not find event 'O' for {:?}", event);
                        // println!("open_events {:?}", &open_events);
                    },
                }
            },
        }
    }
}
