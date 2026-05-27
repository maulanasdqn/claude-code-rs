pub trait TokenEstimator: Send + Sync {
    fn estimate_tokens(&self, text: &str) -> usize;
}

pub struct SimpleEstimator;

impl SimpleEstimator {
    pub fn new() -> Self {
        Self
    }
}

impl TokenEstimator for SimpleEstimator {
    fn estimate_tokens(&self, text: &str) -> usize {

        (text.len() + 3) / 4
    }
}
