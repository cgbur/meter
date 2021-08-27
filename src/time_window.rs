use crate::MIN_DB;
use cpal::StreamInstant;
use keyed_priority_queue::KeyedPriorityQueue;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::time::Duration;

pub(crate) struct TimeWindow {
    pub(crate) keep_secs: f32,
    keep_duration: Duration, // secs
    maxes: KeyedPriorityQueue<StreamInstant, MinNonNan>,
    times: VecDeque<StreamInstant>,
}

impl TimeWindow {
    pub fn new(keep_secs: f32) -> Self {
        Self {
            keep_duration: Duration::from_secs_f32(keep_secs),
            keep_secs,
            maxes: KeyedPriorityQueue::new(),
            times: Default::default(),
        }
    }

    pub fn push(&mut self, time: StreamInstant, value: f32) {
        let value = MinNonNan(-value);
        self.maxes.push(time, value);
        self.times.push_back(time);

        loop {
            if let Some(time) = self
                .times
                .back()
                .unwrap()
                .duration_since(self.times.front().unwrap())
            {
                if time > self.keep_duration {
                    self.maxes.remove(self.times.front().unwrap());
                    self.times.pop_front();
                } else {
                    break;
                }
            } else {
                break;
            }

            if self.times.len() <= 1 {
                break;
            }
        }
    }

    pub fn max(&self) -> f32 {
        -self.maxes.peek().map(|(_, db)| db.0).unwrap_or(-MIN_DB)
    }
}

#[derive(PartialEq)]
struct MinNonNan(f32);

impl Eq for MinNonNan {}

impl PartialOrd for MinNonNan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for MinNonNan {
    fn cmp(&self, other: &MinNonNan) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
