mod time_window;
mod ui;

use crate::time_window::TimeWindow;
use crate::ui::run;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{InputCallbackInfo, Sample, SampleFormat, StreamError};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::Duration;

const MAX_AMPLITUDE_F32: f32 = (u16::MAX / 2) as f32; // i16 max value
const ZERO_AMPLITUDE: u16 = 0;
const MIN_DB: f32 = -96.0;
const FPS: u64 = 15;
const TERMINAL_WIDTH: f32 = 0.8f32;
const DRAW_SLEEP_TIME: Duration = Duration::from_millis(1000 / FPS);

lazy_static! {
    pub(crate) static ref TIME_WINDOWS: Mutex<Vec<TimeWindow>> = Mutex::new(Vec::new());
}

fn db_fs(data: &[f32]) -> f32 {
    let max = data
        .iter()
        .map(|f| Sample::to_i16(f).unsigned_abs() as u16)
        .max()
        .unwrap_or(ZERO_AMPLITUDE);

    (20.0f32 * (max as f32 / MAX_AMPLITUDE_F32).log10()).clamp(MIN_DB, 0.0)
}

fn data_callback(data: &[f32], info: &InputCallbackInfo) {
    let db = db_fs(data);
    let time = info.timestamp().capture;
    let mut windows = TIME_WINDOWS.lock().unwrap();

    for window in windows.iter_mut() {
        window.push(time, db);
    }
}

fn error(e: StreamError) {
    panic!("Error in input stream {:?}", e);
}

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("unable to get default device");

    let config = device
        .default_input_config()
        .expect("unable to get default input config");

    let stream = match config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(&config.into(), data_callback, error),
        _ => panic!("bad format"),
    }
    .expect("unable to build stream");

    let windows = vec![0.2f32, 1.0, 3.0];

    for time in windows {
        TIME_WINDOWS.lock().unwrap().push(TimeWindow::new(time));
    }

    stream.play().expect("unable to play stream");

    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(db_fs(&[0.13f32, 0.1, -0.4]), -7.9586673)
    }

    #[test]
    fn very_high() {
        assert_eq!(db_fs(&[0.13f32, 0.99, -0.4]), -0.0873844)
    }

    #[test]
    fn max_amp() {
        assert_eq!(db_fs(&[0.13f32, 1.0, -0.4]), 0.0)
    }

    #[test]
    fn min_amp() {
        assert_eq!(db_fs(&[0.13f32, -1.0, -0.4]), 0.0)
    }

    #[test]
    fn no_data() {
        assert_eq!(db_fs(&[]), -96.0)
    }
}
