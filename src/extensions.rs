// Copyright (C) 2024 The OpenTimestamps developers

use ots::{
    attestation::Attestation,
    op::Op,
    timestamp::{Step, StepData},
    Timestamp,
};
use std::collections::HashMap;

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

pub trait StepExtension {
    fn merge(&mut self, b: Timestamp);
    fn roots(&self) -> Vec<&Step>;
    fn cat(&mut self, b: Step);
    fn cat_new(&mut self, op: ots::op::Op);
    fn all_attestations(&self) -> HashMap<Vec<u8>, Attestation>;
}
impl StepExtension for Step {
    fn merge(&mut self, b: Timestamp) {
        for n in self.next.iter_mut() {
            n.merge(b.clone());
        }
        if self.output == b.start_digest {
            if let Some(next) = self.next.first() {
                match next.data {
                    StepData::Fork => {}
                    StepData::Op(_) => {}
                    StepData::Attestation(_) => self.next = vec![],
                }
            }
            self.next.push(b.first_step.clone());
        }
    }
    fn cat(&mut self, b: Step) {
        for n in self.next.iter_mut() {
            n.cat(b.clone());
        }
        if self.next.is_empty() {
            self.next.push(b);
        }
    }
    fn cat_new(&mut self, op: Op) {
        let next = Step {
            data: StepData::Op(op.clone()),
            output: op.execute(&self.output),
            next: vec![],
        };
        self.cat(next)
    }
    fn roots(&self) -> Vec<&Step> {
        if self.next.is_empty() {
            return vec![&self];
        }
        let mut res = vec![];
        for n in self.next.iter() {
            res.append(&mut n.roots())
        }
        res
    }
    fn all_attestations(&self) -> HashMap<Vec<u8>, Attestation> {
        let mut attestations = HashMap::default();
        match &self.data {
            StepData::Attestation(attestation) => {
                attestations.insert(self.output.clone(), attestation.clone())
            }
            StepData::Op(_) => None,
            StepData::Fork => None,
        };
        for step in self.next.iter() {
            attestations.extend(step.all_attestations())
        }
        attestations
    }
}
