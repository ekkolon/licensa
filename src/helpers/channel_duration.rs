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

    pub fn as_secs_rounded(&self) -> f32 {
        // Multiply the number by 100 to get two decimal places
        let mut rounded_number = self.get_duration().as_secs_f32() * 100.0;

        // Floor the number to the nearest integer
        rounded_number = f32::floor(rounded_number);

        // Divide the rounded number by 100 to get the rounded decimal part
        rounded_number /= 100.0;

        rounded_number
    }
}
