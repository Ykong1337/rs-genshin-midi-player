use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use chrono::Local;
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use portable_atomic::AtomicF64;
use rayon::prelude::*;
use rayon::ThreadPool;

use crate::maps::{gen, vr};
use crate::ui::Mode;

pub static SPEED: AtomicF64 = AtomicF64::new(1.0);
pub static IS_PLAY: AtomicBool = AtomicBool::new(false);
pub static PLAYING: AtomicBool = AtomicBool::new(false);
pub static PAUSE: AtomicBool = AtomicBool::new(false);

pub static SPACE: AtomicBool = AtomicBool::new(false);
pub static CTRL: AtomicBool = AtomicBool::new(false);
pub static BACK: AtomicBool = AtomicBool::new(false);

static MAP: &'static [i32] = &[
    24, 26, 28, 29, 31, 33, 35, 36, 38, 40, 41, 43, 45, 47, 48, 50, 52, 53, 55, 57, 59, 60, 62, 64,
    65, 67, 69, 71, 72, 74, 76, 77, 79, 81, 83, 84, 86, 88, 89, 91, 93, 95,
];

#[derive(Debug, Clone)]
pub struct Midi {
    pub file_name: Arc<Mutex<Option<PathBuf>>>,
    pub events: Arc<Mutex<Vec<Event>>>,
    pub pool: Arc<ThreadPool>,
}

impl Midi {
    #[inline]
    pub fn new() -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        Midi {
            file_name: Arc::new(Mutex::new(None)),
            events: Arc::new(Mutex::new(vec![])),
            pool: Arc::new(pool),
        }
    }

    #[inline]
    fn play<F: Fn(i32)>(&self, f: F) {
        let events = self.events.lock().unwrap();
        let mut start_time = Local::now().timestamp_millis();
        let mut input_time = 0.0;
        for e in events.iter(){
            if PAUSE.load(Ordering::Relaxed) {
                loop {
                    if !PAUSE.load(Ordering::Relaxed) {
                        input_time = e.delay;
                        start_time = Local::now().timestamp_millis();
                        break;
                    }
                }
            }
            input_time += e.delay / SPEED.load(Ordering::Relaxed);
            let playback_time = (Local::now().timestamp_millis() - start_time) as f64;
            let current_time = (input_time - playback_time) as u64;
            if current_time > 0 {
                sleep(Duration::from_millis(current_time));
            }
            match IS_PLAY.load(Ordering::Relaxed) {
                true => f(e.press),
                false => break,
            }
        }
    }

    pub fn init(&self) {
        let mid = self.clone();
        self.pool.spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("MIDI File", &["mid"])
                .pick_file()
            {
                *mid.file_name.lock().unwrap() = Some(path.clone());

                let file = std::fs::read(path).unwrap();
                let smf = Smf::parse(&file).unwrap();
                let fps = match smf.header.timing {
                    Timing::Metrical(fps) => fps.as_int() as f64,
                    _ => 480.0,
                };

                let mut raw_events = smf
                    .tracks
                    .into_iter()
                    .map(|track| {
                        let mut tick = 0.0;
                        track
                            .into_iter()
                            .map(|event| {
                                tick += event.delta.as_int() as f64;
                                RawEvent {
                                    event: event.kind,
                                    tick,
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                raw_events.par_sort_by_key(|e| e.tick as u64);

                let mut tick = 0.0;
                let mut tempo = 500000.0;
                *mid.events.lock().unwrap() = raw_events
                    .into_iter()
                    .filter_map(|event| match event.event {
                        TrackEventKind::Meta(MetaMessage::Tempo(t)) => {
                            tempo = t.as_int() as f64;
                            None
                        }
                        TrackEventKind::Midi {
                            message: MidiMessage::NoteOn { key, vel },
                            ..
                        } => {
                            if vel > 0 {
                                let time = (event.tick - tick) * (tempo / 1000.0 / fps);
                                tick = event.tick;
                                return Some(Event {
                                    press: key.as_int() as i32,
                                    delay: time,
                                });
                            }
                            None
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
            }
        });
    }

    pub fn playback(&self, tuned: bool, mode: Mode) {
        let mid = self.clone();
        self.pool.spawn(move || {
            PLAYING.store(true, Ordering::Relaxed);
            let mut shift = 0;
            if tuned {
                shift = tune(mid.events.clone());
            }
            let send = match mode {
                Mode::GenShin => gen,
                Mode::VRChat => vr,
            };
            mid.play(|key| {
                send(key + shift);
            });
            PLAYING.store(false, Ordering::Relaxed);
            IS_PLAY.store(false, Ordering::Relaxed);
        });
    }
}

struct RawEvent<'a> {
    event: TrackEventKind<'a>,
    tick: f64,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub press: i32,
    pub delay: f64,
}

fn tune(events: Arc<Mutex<Vec<Event>>>) -> i32 {
    let len = events.lock().unwrap().len() as f32;
    let mut up_hit = vec![];
    let mut down_hit = vec![];
    let mut up_max = 0.0;
    let mut down_max = 0.0;
    let mut up_shift = 0;
    let mut down_shift = 0;

    rayon::join(
        || {
            tune_offset(events.clone(), len, &mut up_hit, 0, true);
            for (i, x) in up_hit.into_iter().enumerate() {
                if x > up_max {
                    up_max = x;
                    up_shift = i as i32;
                }
            }
        },
        || {
            tune_offset(events.clone(), len, &mut down_hit, 0, false);
            for (i, x) in down_hit.into_iter().enumerate() {
                if x > down_max {
                    down_max = x;
                    down_shift = i as i32;
                }
            }
        },
    );

    if up_shift > down_shift {
        return up_shift;
    }
    -down_shift
}

fn tune_offset(
    events: Arc<Mutex<Vec<Event>>>,
    len: f32,
    hit_vec: &mut Vec<f32>,
    offset: i32,
    direction: bool,
) {
    let mut hit_count = 0.0;
    for msg in events.lock().unwrap().iter() {
        let key = msg.press + offset;
        if MAP.contains(&key) {
            hit_count += 1.0;
        }
    }
    let hit = hit_count / len;
    hit_vec.push(hit);
    match direction {
        true => {
            if offset > 21 {
                return;
            }
            tune_offset(events, len, hit_vec, offset + 1, true);
        }
        false => {
            if offset < -21 {
                return;
            }
            tune_offset(events, len, hit_vec, offset - 1, false);
        }
    }
}
