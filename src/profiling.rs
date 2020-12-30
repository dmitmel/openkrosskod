use prelude_plus::*;

#[derive(Debug)]
pub struct AverageTimeSampler {
  samples: Vec<Duration>,
  current_sample_index: usize,
}

impl AverageTimeSampler {
  pub fn new(sample_count: usize) -> Self {
    Self { samples: Vec::with_capacity(sample_count), current_sample_index: 0 }
  }

  #[inline(always)]
  pub fn samples(&self) -> &[Duration] { &self.samples }

  pub fn push(&mut self, duration: Duration) {
    if self.samples.len() < self.samples.capacity() {
      self.samples.push(duration);
    } else {
      if self.current_sample_index >= self.samples.len() {
        self.current_sample_index = 0;
      }
      self.samples[self.current_sample_index] = duration;
    }
    self.current_sample_index += 1;
  }

  pub fn measure<T>(&mut self, block: impl FnOnce() -> T) -> T {
    let start_time = Instant::now();
    let result = block();
    self.push(start_time.elapsed());
    result
  }

  // In the following functions a maximum value with 1 is taken to avoid
  // division by zero. When the samples list is empty the `sum` function
  // returns 0 anyway, so the result of `0 / 1` will still be zero.

  pub fn average_secs(&self) -> u64 {
    self.samples.iter().sum::<Duration>().as_secs() / self.samples.len().max(1) as u64
  }

  pub fn average_millis(&self) -> u128 {
    self.samples.iter().sum::<Duration>().as_millis() / self.samples.len().max(1) as u128
  }

  pub fn average_micros(&self) -> u128 {
    self.samples.iter().sum::<Duration>().as_micros() / self.samples.len().max(1) as u128
  }

  pub fn average_nanos(&self) -> u128 {
    self.samples.iter().sum::<Duration>().as_nanos() / self.samples.len().max(1) as u128
  }
}
