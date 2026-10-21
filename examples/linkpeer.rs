//! A Link peer for testing Diaphus against a session that misbehaves.
//!
//!   linkpeer jump <beats> [secs]   join, move the session's beat by <beats>
//!                                  (negative is backwards), stay [secs]
//!   linkpeer tempo <bpm> [secs]    join, set the session tempo, stay [secs]
//!   linkpeer drag <bpm,bpm,...> <ms>  join and set each tempo in turn,
//!                                  <ms> apart: a hand on Live's tempo field
//!   linkpeer join [secs]           join and stay [secs]: what a new app does
//!
//! `jump` uses Link's force_beat_at_time, which re-maps the beat for every
//! peer: rude, and exactly the discontinuity a peer joining can cause.
use rusty_link::{AblLink, SessionState};
use std::{thread, time::Duration};

const QUANTUM: f64 = 4.0;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verb = args.first().map(String::as_str).unwrap_or("join");
    let num = |i: usize, d: f64| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let link = AblLink::new(120.0);
    link.enable(true);
    for _ in 0..50 {
        if link.num_peers() > 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    // let the session settle on the other peers' timeline before acting
    thread::sleep(Duration::from_millis(500));
    let mut state = SessionState::new();
    link.capture_app_session_state(&mut state);
    let now = link.clock_micros();
    println!("peers {}  tempo {:.2}  beat {:.3}", link.num_peers(), state.tempo(), state.beat_at_time(now, QUANTUM));
    let stay = match verb {
        "jump" => {
            let by = num(1, -2.0);
            let beat = state.beat_at_time(now, QUANTUM);
            state.force_beat_at_time(beat + by, now, QUANTUM);
            link.commit_app_session_state(&state);
            println!("moved the beat by {}", by);
            num(2, 3.0)
        }
        "tempo" => {
            state.set_tempo(num(1, 120.0), now);
            link.commit_app_session_state(&state);
            println!("tempo set to {}", num(1, 120.0));
            num(2, 3.0)
        }
        "drag" => {
            let steps: Vec<f64> = args.get(1).map(|s| s.split(',').filter_map(|t| t.parse().ok()).collect()).unwrap_or_default();
            let gap = Duration::from_millis(num(2, 250.0) as u64);
            for bpm in steps {
                link.capture_app_session_state(&mut state);
                state.set_tempo(bpm, link.clock_micros());
                link.commit_app_session_state(&state);
                thread::sleep(gap);
            }
            println!("dragged");
            1.0
        }
        _ => num(1, 3.0),
    };
    thread::sleep(Duration::from_secs_f64(stay));
    link.capture_app_session_state(&mut state);
    println!("leaving: tempo {:.2}  beat {:.3}", state.tempo(), state.beat_at_time(link.clock_micros(), QUANTUM));
}
