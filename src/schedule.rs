use std::fmt::Display;

use itertools::Itertools;

use crate::data::{Day, SubPar, Subject};

pub type Choice<'s> = Option<SubPar<'s>>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Choices<'s>(pub [[Choice<'s>; 7]; 7]);

pub struct Schedule<'s> {
    subjects: &'s [Subject],
    choices: Choices<'s>,

    filter: &'s mut dyn FnMut(SubPar<'s>, Choices<'s>) -> bool,
    finalize: &'s mut dyn FnMut(Choices<'s>),
}

impl<'s> Schedule<'s> {
    pub fn choose(
        subjects: &'s [Subject],
        filter: &'s mut dyn FnMut(SubPar<'s>, Choices<'s>) -> bool,
        finalize: &'s mut dyn FnMut(Choices<'s>),
    ) {
        let mut this = Self {
            subjects,
            choices: Choices::default(),

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

        (self.finalize)(self.choices);
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
            let choice = &mut self.choices.0[l.day as usize][l.block as usize];

            if choice.is_some() {
                continue;
            } else {
                *choice = Some((subj, l))
            }

            if (self.filter)((subj, l), self.choices) {
                self.choose_lectures(iter.clone());
            }

            self.choices.0[l.day as usize][l.block as usize] = None;
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
            let choice = &mut self.choices.0[l.day as usize][l.block as usize];

            if choice.is_some() {
                continue;
            } else {
                *choice = Some((subj, l))
            }

            if (self.filter)((subj, l), self.choices) {
                self.choose_seminar(iter.clone());
            }

            self.choices.0[l.day as usize][l.block as usize] = None;
        }
    }

    fn choose_lab(&mut self, mut iter: impl Iterator<Item = &'s Subject> + Clone) {
        let Some(subj) = iter.next() else {
            self.finalize();

            return;
        };

        for l in &subj.labs {
            let choice = &mut self.choices.0[l.day as usize][l.block as usize];

            if choice.is_some() {
                continue;
            } else {
                *choice = Some((subj, l))
            }

            if (self.filter)((subj, l), self.choices) {
                self.choose_lab(iter.clone());
            }

            self.choices.0[l.day as usize][l.block as usize] = None;
        }
    }
}

impl<'p> Display for Choices<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ",7:30, 9:15,11:00,12:45,14:30,16:15,18:00")?;

        for day in 1..6 {
            write!(f, "{}", Day::try_from(day).unwrap())?;

            for block in self.0[day as usize] {
                write!(f, ",")?;

                if let Some((subj, par)) = block {
                    write!(f, "{subj} {par}")?;
                };
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
