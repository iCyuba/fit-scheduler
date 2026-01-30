use itertools::Itertools;

use crate::{
    choices::Choices,
    data::{SubPar, Subject},
};

pub struct Scheduler<'s> {
    subjects: &'s [Subject],
    choices: Box<Choices<'s>>,

    filter: &'s mut dyn FnMut(SubPar<'s>, &Choices<'s>) -> bool,
    finalize: &'s mut dyn FnMut(&Choices<'s>),
}

impl<'s> Scheduler<'s> {
    pub fn choose(
        subjects: &'s [Subject],
        filter: &'s mut dyn FnMut(SubPar<'s>, &Choices<'s>) -> bool,
        finalize: &'s mut dyn FnMut(&Choices<'s>),
    ) {
        let mut this = Self {
            subjects,
            choices: Box::new(Choices::default()),

            filter,
            finalize,
        };

        this.choose_lectures(
            this.subjects
                .iter()
                .filter(|s| !s.lectures.is_empty())
                .sorted_unstable_by_key(|s| s.lectures.len()),
        );
    }

    pub fn finalize(&mut self) {
        // println!("{}", self.choices);

        (self.finalize)(&self.choices);
    }

    fn choose_lectures(&mut self, mut iter: impl Iterator<Item = &'s Subject> + Clone) {
        let Some(subj) = iter.next() else {
            self.choose_seminar(
                self.subjects
                    .iter()
                    .filter(|s| !s.seminars.is_empty())
                    .sorted_unstable_by_key(|s| s.seminars.len()),
            );

            return;
        };

        for l in &subj.lectures {
            let choice = &mut self.choices[l.time];

            if choice.is_some() {
                continue;
            } else {
                *choice = Some((subj, l))
            }

            if (self.filter)((subj, l), &self.choices) {
                self.choose_lectures(iter.clone());
            }

            self.choices[l.time] = None;
        }
    }

    fn choose_seminar(&mut self, mut iter: impl Iterator<Item = &'s Subject> + Clone) {
        let Some(subj) = iter.next() else {
            self.choose_lab(
                self.subjects
                    .iter()
                    .filter(|s| !s.labs.is_empty())
                    .sorted_unstable_by_key(|s| s.labs.len()),
            );

            return;
        };

        for l in &subj.seminars {
            let choice = &mut self.choices[l.time];

            if choice.is_some() {
                continue;
            } else {
                *choice = Some((subj, l))
            }

            if (self.filter)((subj, l), &self.choices) {
                self.choose_seminar(iter.clone());
            }

            self.choices[l.time] = None;
        }
    }

    fn choose_lab(&mut self, mut iter: impl Iterator<Item = &'s Subject> + Clone) {
        let Some(subj) = iter.next() else {
            self.finalize();

            return;
        };

        for l in &subj.labs {
            let choice = &mut self.choices[l.time];

            if choice.is_some() {
                continue;
            } else {
                *choice = Some((subj, l))
            }

            if (self.filter)((subj, l), &self.choices) {
                self.choose_lab(iter.clone());
            }

            self.choices[l.time] = None;
        }
    }
}
