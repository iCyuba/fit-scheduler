use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::data::{Block, Day, SubPar, Time};

#[derive(Debug, Clone, Default)]
pub struct ChoicesDay<'s>([Option<SubPar<'s>>; 7]);

#[derive(Debug, Clone, Default)]
pub struct Choices<'s>([ChoicesDay<'s>; 7]);

impl ChoicesDay<'_> {
    pub fn consecutive(&self, block: Block) -> u8 {
        let mut count = 0;

        for i in (0..block as usize).rev() {
            if self.0[i].is_none() {
                break;
            }

            count += 1;
        }

        for i in block as usize + 1..7 {
            if self.0[i].is_none() {
                break;
            }

            count += 1;
        }

        count
    }
}

impl Choices<'_> {
    pub fn consecutive(&self, time: Time) -> u8 {
        self[time.day].consecutive(time.block)
    }
}

impl<'s> Index<Block> for ChoicesDay<'s> {
    type Output = Option<SubPar<'s>>;

    fn index(&self, index: Block) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<'s> IndexMut<Block> for ChoicesDay<'s> {
    fn index_mut(&mut self, index: Block) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<'s> Index<Day> for Choices<'s> {
    type Output = ChoicesDay<'s>;

    fn index(&self, index: Day) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<'s> IndexMut<Day> for Choices<'s> {
    fn index_mut(&mut self, index: Day) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<'s> Index<Time> for Choices<'s> {
    type Output = Option<SubPar<'s>>;

    fn index(&self, index: Time) -> &Self::Output {
        &self[index.day][index.block]
    }
}

impl<'s> IndexMut<Time> for Choices<'s> {
    fn index_mut(&mut self, index: Time) -> &mut Self::Output {
        &mut self[index.day][index.block]
    }
}

impl<'p> Display for Choices<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ",7:30, 9:15,11:00,12:45,14:30,16:15,18:00")?;

        for day in 1..6 {
            let day = Day::try_from(day).unwrap();

            write!(f, "{}", day)?;

            for block in self[day].0 {
                write!(f, ",")?;

                if let Some((subj, time)) = block {
                    write!(f, "{subj} {time}")?;
                };
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
