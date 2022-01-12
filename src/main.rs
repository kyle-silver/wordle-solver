use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::io::{stdin, stdout, Read, Write};

const CORPUS: &str = include_str!("res/words.txt");

/// The relative frequency of each letter per-position across all five-letter
/// words. Pre-computation is pretty rad, we should use lookup tables more
/// often.
const FREQS: [[f64; 26]; 5] = [
    [
        0.07735058309037901,
        0.07083637026239067,
        0.06887755102040816,
        0.05138483965014577,
        0.031568877551020405,
        0.038584183673469385,
        0.04395954810495627,
        0.03844752186588921,
        0.02154701166180758,
        0.020317055393586005,
        0.035942055393586005,
        0.04764941690962099,
        0.05580357142857143,
        0.029473396501457725,
        0.02423469387755102,
        0.05393586005830904,
        0.004236516034985423,
        0.04318513119533528,
        0.10049198250728864,
        0.0575801749271137,
        0.01913265306122449,
        0.018403790087463557,
        0.027150145772594753,
        0.0016399416909620992,
        0.01024963556851312,
        0.008017492711370262,
    ],
    [
        0.18690779883381925,
        0.00783527696793003,
        0.015397230320699708,
        0.011160714285714286,
        0.13042091836734693,
        0.0033254373177842565,
        0.006331997084548105,
        0.042228498542274055,
        0.10199526239067055,
        0.0014577259475218659,
        0.006240889212827988,
        0.054345845481049565,
        0.014896137026239067,
        0.030566690962099127,
        0.13980502915451895,
        0.015306122448979591,
        0.0011388483965014578,
        0.07028972303206997,
        0.015533892128279884,
        0.018358236151603497,
        0.08309037900874636,
        0.005739795918367347,
        0.010295189504373178,
        0.0037809766763848398,
        0.020863702623906705,
        0.0026876822157434403,
    ],
    [
        0.08924016034985423,
        0.030247813411078718,
        0.03193330903790088,
        0.035258746355685135,
        0.06473214285714286,
        0.012345116618075802,
        0.02806122448979592,
        0.015215014577259475,
        0.07598396501457726,
        0.003689868804664723,
        0.018494897959183673,
        0.07302295918367346,
        0.040315233236151604,
        0.08003826530612244,
        0.06495991253644315,
        0.025510204081632654,
        0.0014577259475218659,
        0.10026421282798834,
        0.046510568513119535,
        0.04774052478134111,
        0.04509839650145773,
        0.018995991253644314,
        0.01589832361516035,
        0.007744169096209913,
        0.017356049562682215,
        0.009885204081632654,
    ],
    [
        0.1008564139941691,
        0.019269314868804666,
        0.035076530612244895,
        0.037764212827988336,
        0.14814139941690962,
        0.01266399416909621,
        0.027879008746355686,
        0.019269314868804666,
        0.08942237609329447,
        0.002778790087463557,
        0.02860787172011662,
        0.05899234693877551,
        0.027879008746355686,
        0.06336552478134111,
        0.05840014577259475,
        0.02396137026239067,
        0.00022776967930029153,
        0.05580357142857143,
        0.043048469387755105,
        0.06309220116618076,
        0.04095298833819242,
        0.011798469387755101,
        0.008746355685131196,
        0.0011844023323615161,
        0.012345116618075802,
        0.008473032069970845,
    ],
    [
        0.10759839650145772,
        0.005876457725947522,
        0.013666180758017493,
        0.04177295918367347,
        0.12203899416909621,
        0.006605320699708455,
        0.01188957725947522,
        0.029792274052478133,
        0.04145408163265306,
        0.00045553935860058307,
        0.022868075801749273,
        0.04423287172011662,
        0.018494897959183673,
        0.06933309037900874,
        0.03903972303206997,
        0.012982871720116617,
        0.0004099854227405248,
        0.054209183673469385,
        0.17160167638483964,
        0.058491253644314865,
        0.011251822157434402,
        0.0018677113702623906,
        0.005557580174927114,
        0.006559766763848397,
        0.09639212827988339,
        0.005557580174927114,
    ],
];

fn score(word: &str) -> f64 {
    let mut uniques = [false; 26];
    let mut raw_score = 0f64;
    for (i, c) in word.chars().enumerate() {
        let index = (c as u32 - 'a' as u32) as usize;
        uniques[index] = true;
        raw_score += FREQS[i][(c as u32 - 'a' as u32) as usize]
    }
    // if we just used the highest-scoring word, we'd get a lot of repeat
    // letters. This is a really primitive way of incentivizing words with many
    // unique letters.
    let unique_letters = uniques.into_iter().filter(|x| *x).count();
    raw_score * unique_letters as f64
}

#[derive(Debug, Clone)]
enum Hint {
    Unused,
    NotIn(Vec<usize>),
    In(Vec<usize>),
}

#[derive(Debug, Default)]
struct Hints(HashMap<char, Hint>);

impl Hints {
    fn add_unused(&mut self, c: char) {
        self.0.insert(c, Hint::Unused);
    }

    fn add_used_at(&mut self, c: char, index: usize) {
        match self.0.entry(c) {
            Entry::Occupied(mut occupied) => match occupied.get_mut() {
                Hint::In(positions) => positions.push(index),
                _ => {}
            },
            Entry::Vacant(vacant) => {
                vacant.insert(Hint::In(vec![index]));
            }
        }
    }

    fn add_not_used_at(&mut self, c: char, index: usize) {
        match self.0.entry(c) {
            Entry::Occupied(mut occupied) => match occupied.get_mut() {
                Hint::NotIn(positions) => positions.push(index),
                _ => {}
            },
            Entry::Vacant(vacant) => {
                vacant.insert(Hint::NotIn(vec![index]));
            }
        }
    }

    fn valid(&self, candidate: &str) -> bool {
        for (i, c) in candidate.chars().enumerate() {
            if let Some(hint) = self.0.get(&c) {
                match hint {
                    Hint::Unused => return false,
                    Hint::NotIn(positions) => {
                        if positions.contains(&i) {
                            return false;
                        }
                    }
                    Hint::In(positions) => {
                        if !positions.contains(&i) {
                            return false;
                        }
                    }
                }
            }
        }
        return true;
    }

    fn merge(&mut self, others: Hints) {
        for (c, hint) in others.0 {
            match self.0.entry(c) {
                Entry::Occupied(mut occupied) => match occupied.get_mut() {
                    Hint::NotIn(positions) => match hint {
                        Hint::NotIn(other_positions) => {
                            // types match, combine them
                            positions.extend(other_positions.iter());
                        }
                        Hint::In(positions) => {
                            // the "in" hint takes precedence over "not in"
                            *occupied.get_mut() = Hint::In(positions);
                        }
                        Hint::Unused => unreachable!("cannot be both used and unused"),
                    },
                    Hint::In(positions) => match hint {
                        Hint::In(other_positions) => {
                            // types match, combine them
                            positions.extend(other_positions.iter());
                        }
                        Hint::NotIn(_) => { /* I'm not sure what to do with this case */ }
                        Hint::Unused => unreachable!("cannot be both used and unused"),
                    },
                    Hint::Unused => {}
                },
                Entry::Vacant(vacant_entry) => {
                    // the incoming hint is new information
                    vacant_entry.insert(hint);
                }
            }
        }
    }
}

#[derive(Debug)]
struct Guesser<'a> {
    candidates: HashSet<&'a str>,
    hints: Hints,
}

impl<'a> Guesser<'a> {
    fn new(lines: impl Iterator<Item = &'a str>) -> Self {
        Guesser {
            hints: Hints::default(),
            candidates: lines.collect(),
        }
    }

    fn suggest(&self) -> &str {
        *self
            .candidates
            .iter()
            .max_by(|a, b| score(a).partial_cmp(&score(b)).unwrap())
            .unwrap()
    }

    fn update(&mut self, hints: Hints) {
        // consume all of the new hints
        self.hints.merge(hints);
        let invalid: HashSet<&'a str> = self
            .candidates
            .iter()
            .filter_map(|word| match self.hints.valid(word) {
                true => None,
                false => Some(*word),
            })
            .collect();
        for invalid_word in invalid.into_iter() {
            self.candidates.remove(invalid_word);
        }
    }
}

fn main() {
    let mut guesser = Guesser::new(CORPUS.lines());
    for i in 1..=6 {
        let guess = guesser.suggest();
        println!(
            "Guess #{}: {} ('g' = green / 'y' = yellow / 'b' = black)",
            i, guess
        );
        let mut hints = Hints::default();
        for (i, c) in guess.chars().enumerate() {
            print!("{}: ", c);
            let _ = stdout().flush();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            match input.chars().next() {
                Some(user_char) => match user_char.to_ascii_lowercase() {
                    'g' => hints.add_used_at(c, i),
                    'y' => hints.add_not_used_at(c, i),
                    _ => hints.add_unused(c),
                },
                None => hints.add_unused(c),
            };
            // println!();
        }
        guesser.update(hints);
    }
}
