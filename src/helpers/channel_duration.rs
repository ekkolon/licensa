use std::time::{Duration, SystemTime};

pub struct ChannelDuration {
    creation_time: SystemTime,
    dropped_time: Option<SystemTime>,
}

impl ChannelDuration {
    pub fn new() -> Self {
        ChannelDuration {
            creation_time: SystemTime::now(),
            dropped_time: None,
        }
    }

    pub fn drop_channel(&mut self) {
        self.dropped_time = Some(SystemTime::now());
    }

    pub fn get_duration(&self) -> Duration {
        match &self.dropped_time {
            None => SystemTime::now()
                .duration_since(self.creation_time)
                .unwrap(),
            Some(dropped_time) => dropped_time.duration_since(self.creation_time).unwrap(),
        }
    }
}
