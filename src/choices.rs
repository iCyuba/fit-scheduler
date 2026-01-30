use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    ptr,
};

use strum::IntoEnumIterator;

use crate::data::{Block, Day, SubPar, Time};

#[derive(Debug, Clone, Default)]
pub struct ChoicesDay<'s>([Option<SubPar<'s>>; 7]);

#[derive(Debug, Clone, Default)]
pub struct ChoicesWeek<'s>([ChoicesDay<'s>; 7]);

#[derive(Debug, Clone, Default)]
pub struct Choices<'s>([ChoicesWeek<'s>; 2]);

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
        u8::max(
            self[false][time.day].consecutive(time.block),
            self[true][time.day].consecutive(time.block),
        )
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

impl<'s> Index<Time> for ChoicesWeek<'s> {
    type Output = Option<SubPar<'s>>;

    fn index(&self, index: Time) -> &Self::Output {
        &self[index.day][index.block]
    }
}

impl<'s> IndexMut<Time> for ChoicesWeek<'s> {
    fn index_mut(&mut self, index: Time) -> &mut Self::Output {
        &mut self[index.day][index.block]
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

impl<'s> Index<Time> for Choices<'s> {
    type Output = Option<SubPar<'s>>;

    fn index(&self, index: Time) -> &Self::Output {
        &self[index.biweekly][index.day][index.block]
    }
}

impl<'s> IndexMut<Time> for Choices<'s> {
    fn index_mut(&mut self, index: Time) -> &mut Self::Output {
        &mut self[index.biweekly][index.day][index.block]
    }
}

impl<'p> Display for Choices<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ",7:30, 9:15,11:00,12:45,14:30,16:15,18:00")?;

        for day in Day::iter().skip(1).take(5) {
            write!(f, "{}", day)?;

            let odd = &self[false][day];
            let even = &self[true][day];

            for block in Block::iter() {
                write!(f, ",")?;

                if let Some((subj, par)) = odd[block] {
                    write!(f, "{subj} {par}")?;

                    if let Some((subj_even, par_even)) = even[block]
                        && !(ptr::addr_eq(subj, subj_even) && par == par_even)
                    {
                        write!(f, " / {subj_even} {par_even}")?;
                    };
                };
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
