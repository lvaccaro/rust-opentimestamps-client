use ots::{
    attestation::Attestation,
    timestamp::Step,
    Timestamp,
};
use std::collections::HashMap;
use step_extension::StepExtension;

pub trait TimestampExtension {
    fn merge(&mut self, b: Timestamp);
    fn roots(&self) -> Vec<&Step>;
    fn cat(&mut self, b: Step);
    fn all_attestations(&self) -> HashMap<Vec<u8>, Attestation>;
}
impl TimestampExtension for Timestamp {
    fn merge(&mut self, b: Timestamp) {
        self.first_step.merge(b)
    }
    fn roots(&self) -> Vec<&Step> {
        self.first_step.roots()
    }
    fn cat(&mut self, b: Step) {
        self.first_step.cat(b)
    }
    fn all_attestations(&self) -> HashMap<Vec<u8>, Attestation> {
        self.first_step.all_attestations()
    }
}
