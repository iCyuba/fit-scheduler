use itertools::Itertools;
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};
use strum::IntoEnumIterator;

use data::{Block, Day, Subject, Time, Type};

#[derive(Debug, Clone)]
pub struct Choice<'s> {
    pub subject: &'s Subject,
    pub kind: Type,
    pub time: &'s Time,
}

impl<'s> Display for Choice<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.subject.code, self.kind,)
    }
}

impl<'s> Choices<'s> {
    pub fn get_ordered_choices(&self, even: bool, day: Day) -> Vec<&Choice<'s>> {
        self[even][day]
            .items
            .iter()
            .sorted_unstable_by_key(|c| c.time.block.offset)
            .collect_vec()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChoicesDay<'s> {
    pub items: Vec<Choice<'s>>,
    pub occupied: u128,
}

impl<'s> ChoicesDay<'s> {
    pub fn consecutive(&self) -> u8 {
        let mut count = 0;
        let mut val = self.occupied;

        while val != 0 {
            val = val & (val << 1);
            count += 1;
        }

        count
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChoicesWeek<'s>([ChoicesDay<'s>; 7]);

#[derive(Debug, Clone, Default)]
pub struct Choices<'s>([ChoicesWeek<'s>; 2]);

impl<'s> Choices<'s> {
    pub fn add_choice(&mut self, choice: Choice<'s>, even: bool) {
        let day = &mut self[even][choice.time.day];

        day.occupied |= u128::from(choice.time);
        day.items.push(choice);
    }

    pub fn remove_choice(&mut self, day: Day, even: bool) {
        let day = &mut self[even][day];

        if let Some(choice) = day.items.pop() {
            day.occupied &= !u128::from(choice.time);
        }
    }

    pub fn consecutive(&self, time: Time) -> u8 {
        u8::max(
            self[false][time.day].consecutive(),
            self[true][time.day].consecutive(),
        )
    }
}

impl<'s> Index<Day> for ChoicesWeek<'s> {
    type Output = ChoicesDay<'s>;

    fn index(&self, index: Day) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<'s> IndexMut<Day> for ChoicesWeek<'s> {
    fn index_mut(&mut self, index: Day) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<'s> Index<bool> for Choices<'s> {
    type Output = ChoicesWeek<'s>;

    fn index(&self, index: bool) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<'s> IndexMut<bool> for Choices<'s> {
    fn index_mut(&mut self, index: bool) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<'p> Display for Choices<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in 0..12 * 4 {
            write!(f, ",{}", Block { offset: 30 + b })?;
        }
        writeln!(f)?;

        for day in Day::iter().skip(1).take(5) {
            write!(f, "{},", day)?;
            let mut last = 30;
            for choice in self.get_ordered_choices(false, day) {
                for _ in last..choice.time.block.offset {
                    write!(f, ",")?;
                }
                last = choice.time.block.offset + choice.time.duration;
                for _ in 0..choice.time.duration {
                    write!(f, "{},", choice)?;
                }
            }
            writeln!(f)?;

            write!(f, "{},", day)?;
            let mut last = 30;
            for choice in self.get_ordered_choices(true, day) {
                for _ in last..choice.time.block.offset {
                    write!(f, ",")?;
                }
                last = choice.time.block.offset + choice.time.duration;
                for _ in 0..choice.time.duration {
                    write!(f, "{},", choice)?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
