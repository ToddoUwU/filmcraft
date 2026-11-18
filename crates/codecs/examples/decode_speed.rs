//! How fast FilmCraft's own decoders read a file, frame by frame in display order, the way playback
//! asks for frames. Prints frames per second for one reader and for several readers sharing the
//! frames (playback can decode ahead on more than one thread).
//!
//! cargo run --release -p filmcraft-codecs --example decode_speed -- <file> <fps num> <fps den> <frames> [threads]

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use filmcraft_media::FrameRequest;
use filmcraft_time::{Tick, TICKS_PER_SECOND};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, path, num, den, frames, rest @ ..] = args.as_slice() else {
        eprintln!("usage: decode_speed <file> <fps num> <fps den> <frames> [threads]");
        std::process::exit(2);
    };
    let (num, den, frames): (i64, i64, i64) = (num.parse().unwrap(), den.parse().unwrap(), frames.parse().unwrap());
    let threads: i64 = rest.first().map_or(4, |t| t.parse().unwrap());
    let frame_ticks = TICKS_PER_SECOND * den / num;
    let at = |n: i64| FrameRequest::full(Tick(n * frame_ticks + frame_ticks / 2));

    let source = filmcraft_media::open_path(Path::new(path), &filmcraft_codecs::openers()).unwrap();
    let first = source.video_frame(at(0)).unwrap();
    println!("opened: {}x{}, color {:?}", first.width, first.height, first.color);

    let started = Instant::now();
    for n in 0..frames {
        source.video_frame(at(n)).unwrap();
    }
    println!("one reader: {:.1} fps", frames as f64 / started.elapsed().as_secs_f64());

    let shared = Arc::new(filmcraft_media::open_path(Path::new(path), &filmcraft_codecs::openers()).unwrap());
    let started = Instant::now();
    std::thread::scope(|scope| {
        for t in 0..threads {
            let shared = Arc::clone(&shared);
            scope.spawn(move || {
                let mut n = t;
                while n < frames {
                    shared.video_frame(at(n)).unwrap();
                    n += threads;
                }
            });
        }
    });
    println!("{threads} readers on one source: {:.1} fps", frames as f64 / started.elapsed().as_secs_f64());
}
