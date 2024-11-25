#[derive(Debug, Clone, Copy)]
pub struct Gen {
    value: u64,
}

pub fn new() -> Gen {
    Gen { value: 0 }
}

impl Gen {
    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn generate(&mut self) -> u64 {
        let result = self.value;
        self.value += 1;
        result
    }
}
