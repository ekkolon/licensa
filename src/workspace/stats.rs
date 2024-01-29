pub struct ScanStats {
    pub skipped: usize,
}

impl ScanStats {
    pub fn skip(&mut self) {
        self.skipped += 1
    }

    pub fn new() -> Self {
        ScanStats { skipped: 0 }
    }
}
