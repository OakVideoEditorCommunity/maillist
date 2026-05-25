pub struct ModerationPolicy {
    pub high_risk_threshold: i32,
    pub medium_risk_threshold: i32,
}

impl ModerationPolicy {
    pub fn verdict(&self, score: i32) -> &'static str {
        if score >= self.high_risk_threshold {
            "flagged"
        } else if score >= self.medium_risk_threshold {
            "caution"
        } else {
            "clean"
        }
    }
}
