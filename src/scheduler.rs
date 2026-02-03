use itertools::Itertools;

use data::{SubPar, Subject};

use crate::choices::Choices;

pub struct SchedulerCallbacks<'c, 's> {
    pub filter: &'c dyn Fn(SubPar<'s>) -> bool,
    pub select: &'c dyn Fn(SubPar<'s>, &Choices<'s>) -> bool,
    pub callback: &'c mut dyn FnMut(&Choices<'s>),
}

pub struct Scheduler<'c, 's> {
    pub callbacks: SchedulerCallbacks<'c, 's>,
    choices: Box<Choices<'s>>,
}

impl<'c, 's> Scheduler<'c, 's> {
    pub fn new(cb: SchedulerCallbacks<'c, 's>) -> Self {
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
                    let sp = SubPar(s, p);

                    (self.callbacks.filter)(sp).then_some(sp)
                };

                let lectures = s.lectures.iter().dedup().filter_map(fm).collect_vec();
                let seminars = s.seminars.iter().dedup().filter_map(fm).collect_vec();
                let labs = s.labs.iter().dedup().filter_map(fm).collect_vec();

                if lectures.is_empty() {
                    eprintln!("{s} lectures are empty!");
                }

                if seminars.is_empty() {
                    eprintln!("{s} seminars are empty!");
                }

                if labs.is_empty() {
                    eprintln!("{s} labs are empty!");
                }

                [lectures, seminars, labs]
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
            let SubPar(_, p) = sp;

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
