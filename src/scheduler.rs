use itertools::Itertools;

use crate::{
    choices::Choices,
    data::{SubPar, Subject},
};

pub struct SchedulerCallbacks<'s> {
    pub filter: &'s dyn Fn(SubPar<'s>) -> bool,
    pub select: &'s dyn Fn(SubPar<'s>, &Choices<'s>) -> bool,
    pub callback: &'s mut dyn FnMut(&Choices<'s>),
}

pub struct Scheduler<'s> {
    pub callbacks: SchedulerCallbacks<'s>,
    choices: Box<Choices<'s>>,
}

impl<'s> Scheduler<'s> {
    pub fn new(cb: SchedulerCallbacks<'s>) -> Self {
        Self {
            choices: Box::new(Choices::default()),
            callbacks: cb,
        }
    }

    pub fn schedule(&mut self, subjects: &'s [Subject]) {
        let parallels = self.get_parallels(subjects);

        self.select(parallels.iter());
    }

    fn get_parallels(&self, subjects: &'s [Subject]) -> Vec<Vec<SubPar<'s>>> {
        subjects
            .iter()
            .flat_map(|s| {
                let fm = move |p| {
                    let sp = (s, p);

                    (self.callbacks.filter)(sp).then_some(sp)
                };

                [
                    s.lectures.iter().dedup().filter_map(fm).collect_vec(),
                    s.seminars.iter().dedup().filter_map(fm).collect_vec(),
                    s.labs.iter().dedup().filter_map(fm).collect_vec(),
                ]
            })
            .filter(|v| !v.is_empty())
            .sorted_unstable_by_key(|v| v.len())
            .collect_vec()
    }

    fn select<'a>(&mut self, mut iter: impl Iterator<Item = &'a Vec<SubPar<'s>>> + Clone)
    where
        's: 'a,
    {
        let Some(parallels) = iter.next() else {
            (self.callbacks.callback)(&self.choices);

            return;
        };

        for &sp in parallels {
            let (_, p) = sp;

            let odd = self.choices[false][p.time].is_some();
            if odd && self.choices[p.time].is_some() {
                continue;
            }

            let mut index_a = p.time;
            let mut index_b = p.time;

            if !p.time.biweekly {
                index_a.biweekly = true;
            } else if !odd {
                index_a.biweekly = false;
                index_b.biweekly = false;
            }

            self.choices[index_a] = Some(sp);
            self.choices[index_b] = Some(sp);

            if (self.callbacks.select)(sp, &self.choices) {
                self.select(iter.clone());
            }

            self.choices[index_a] = None;
            self.choices[index_b] = None;
        }
    }
}
