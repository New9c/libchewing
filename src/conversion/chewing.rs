use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Neg,
    rc::Rc,
};

use log::{trace, warn};

use crate::dictionary::{Dictionary, Phrase};

use super::{Break, Composition, ConversionEngine, Glue, Interval, Symbol};

/// TODO: doc
#[derive(Debug)]
pub struct ChewingEngine;

impl<C: Dictionary + ?Sized> ConversionEngine<C> for ChewingEngine {
    fn convert(&self, dict: &C, composition: &Composition) -> Vec<Interval> {
        if composition.buffer.is_empty() {
            return vec![];
        }
        let intervals = self.find_intervals(dict, composition);
        self.find_best_path(composition.buffer.len(), intervals)
            .into_iter()
            .map(|interval| interval.into())
            .fold(vec![], |acc, interval| glue_fn(composition, acc, interval))
    }

    fn convert_next(&self, dict: &C, composition: &Composition, next: usize) -> Vec<Interval> {
        if composition.buffer.is_empty() {
            return vec![];
        }
        let mut graph = Graph::default();
        let paths = self.find_all_paths(
            dict,
            &mut graph,
            composition,
            0,
            composition.buffer.len(),
            None,
        );
        if paths.is_empty() {
            warn!(
                "BUG! find_all_paths returned nothing from {:?}",
                composition
            );
            return vec![];
        }
        let mut trimmed_paths = self.trim_paths(paths);
        if trimmed_paths.is_empty() {
            warn!("BUG! trimmed paths is empty from");
            return vec![];
        }
        trimmed_paths.sort();
        trimmed_paths
            .into_iter()
            .rev()
            .cycle()
            .nth(next)
            .map(|p| p.intervals)
            .expect("should have path")
            .into_iter()
            .map(|it| it.into())
            .fold(vec![], |acc, interval| glue_fn(composition, acc, interval))
    }
}

fn glue_fn(com: &Composition, mut acc: Vec<Interval>, interval: Interval) -> Vec<Interval> {
    if acc.is_empty() {
        acc.push(interval);
        return acc;
    }
    let last = acc.last().expect("acc should have at least one item");
    if com.glues.contains(&Glue(last.end)) {
        let last = acc.pop().expect("acc should have at least one item");
        acc.push(Interval {
            start: last.start,
            end: interval.end,
            is_phrase: true,
            phrase: last.phrase + &interval.phrase,
        })
    } else {
        acc.push(interval);
    }
    acc
}

impl ChewingEngine {
    /// TODO: doc
    pub fn new() -> ChewingEngine {
        ChewingEngine
    }
}

impl ChewingEngine {
    fn find_best_phrase<D: Dictionary + ?Sized>(
        &self,
        dict: &D,
        start: usize,
        symbols: &[Symbol],
        selections: &[Interval],
        breaks: &[Break],
    ) -> Option<PossiblePhrase> {
        let end = start + symbols.len();

        for br in breaks.iter() {
            if br.0 > start && br.0 < end {
                // There exists a break point that forbids connecting these
                // syllables.
                return None;
            }
        }

        if symbols.len() == 1 && symbols[0].is_char() {
            return Some(symbols[0].into());
        }

        let syllables = symbols
            .iter()
            .take_while(|symbol| symbol.is_syllable())
            .map(|symbol| symbol.as_syllable())
            .collect::<Vec<_>>();
        if syllables.len() != symbols.len() {
            return None;
        }

        let mut max_freq = 0;
        let mut best_phrase = None;
        'next_phrase: for phrase in dict.lookup_all_phrases(&syllables) {
            // If there exists a user selected interval which is a
            // sub-interval of this phrase but the substring is
            // different then we can skip this phrase.
            for selection in selections.iter() {
                debug_assert!(!selection.phrase.is_empty());
                if start <= selection.start && end >= selection.end {
                    let offset = selection.start - start;
                    let len = selection.end - selection.start;
                    let substring: String =
                        phrase.as_str().chars().skip(offset).take(len).collect();
                    if substring != selection.phrase {
                        continue 'next_phrase;
                    }
                }
            }

            // If there are phrases that can satisfy all the constraints
            // then pick the one with highest frequency.
            if best_phrase.is_none() || phrase.freq() > max_freq {
                max_freq = phrase.freq();
                // TODO can we allocate less?
                best_phrase = Some(phrase.into());
            }
        }

        best_phrase
    }
    fn find_intervals<D: Dictionary + ?Sized>(
        &self,
        dict: &D,
        comp: &Composition,
    ) -> Vec<PossibleInterval> {
        let mut intervals = vec![];
        for begin in 0..comp.buffer.len() {
            for end in begin..=comp.buffer.len() {
                if let Some(phrase) = self.find_best_phrase(
                    dict,
                    begin,
                    &comp.buffer[begin..end],
                    &comp.selections,
                    &comp.breaks,
                ) {
                    intervals.push(PossibleInterval {
                        start: begin,
                        end,
                        phrase,
                    });
                }
            }
        }
        intervals
    }
    /// Calculate the best path with dynamic programming.
    ///
    /// Assume P(x,y) is the highest score phrasing result from x to y. The
    /// following is formula for P(x,y):
    ///
    /// P(x,y) = MAX( P(x,y-1)+P(y-1,y), P(x,y-2)+P(y-2,y), ... )
    ///
    /// While P(x,y-1) is stored in highest_score array, and P(y-1,y) is
    /// interval end at y. In this formula, x is always 0.
    ///
    /// The format of highest_score array is described as following:
    ///
    /// highest_score[0] = P(0,0)
    /// highest_score[1] = P(0,1)
    /// ...
    /// highest_score[y-1] = P(0,y-1)
    fn find_best_path(
        &self,
        len: usize,
        mut intervals: Vec<PossibleInterval>,
    ) -> Vec<PossibleInterval> {
        let mut highest_score = vec![PossiblePath::default(); len + 1];

        // The interval shall be sorted by the increase order of end.
        intervals.sort_by(|a, b| a.end.cmp(&b.end));

        for interval in intervals.into_iter() {
            let start = interval.start;
            let end = interval.end;

            let mut candidate_path = highest_score[start].clone();
            candidate_path.intervals.push(interval);

            if highest_score[end].score() < candidate_path.score() {
                highest_score[end] = candidate_path;
            }
        }

        highest_score
            .pop()
            .expect("highest_score has at least one element")
            .intervals
    }

    fn find_all_paths<'g, D: Dictionary + ?Sized>(
        &'g self,
        dict: &D,
        graph: &mut Graph<'g>,
        composition: &Composition,
        start: usize,
        target: usize,
        prefix: Option<PossiblePath>,
    ) -> Vec<PossiblePath> {
        if start == target {
            return vec![prefix.expect("should have prefix")];
        }
        let mut result = vec![];
        for end in (start + 1)..=target {
            let entry = graph.entry((start, end));
            if let Some(phrase) = entry.or_insert_with(|| {
                self.find_best_phrase(
                    dict,
                    start,
                    &composition.buffer[start..end],
                    &composition.selections,
                    &composition.breaks,
                )
            }) {
                let mut prefix = prefix.clone().unwrap_or_default();
                prefix.intervals.push(PossibleInterval {
                    start,
                    end,
                    phrase: phrase.clone(),
                });
                result.append(&mut self.find_all_paths(
                    dict,
                    graph,
                    composition,
                    end,
                    target,
                    Some(prefix),
                ));
            }
        }
        result
    }

    /// Trim some paths that were part of other paths
    ///
    /// Ported from original C implementation, but the original algorithm seems wrong.
    fn trim_paths(&self, paths: Vec<PossiblePath>) -> Vec<PossiblePath> {
        let mut trimmed_paths: Vec<PossiblePath> = vec![];
        for candidate in paths.into_iter() {
            trace!("Trim check {}", candidate);
            let mut drop_candidate = false;
            let mut keeper = vec![];
            for p in trimmed_paths.into_iter() {
                if drop_candidate || p.contains(&candidate) {
                    drop_candidate = true;
                    trace!("  Keep {}", p);
                    keeper.push(p);
                    continue;
                }
                if candidate.contains(&p) {
                    trace!("  Drop {}", p);
                    continue;
                }
                trace!("  Keep {}", p);
                keeper.push(p);
            }
            if !drop_candidate {
                trace!("  Keep {}", candidate);
                keeper.push(candidate);
            }
            trimmed_paths = keeper;
        }
        trimmed_paths
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PossiblePhrase {
    Symbol(Symbol),
    Phrase(Rc<Phrase>),
}

impl PossiblePhrase {
    fn to_string(&self) -> String {
        match self {
            PossiblePhrase::Symbol(sym) => sym.as_char().to_string(),
            PossiblePhrase::Phrase(phrase) => phrase.to_string(),
        }
    }
    fn freq(&self) -> u32 {
        match self {
            PossiblePhrase::Symbol(_) => 0,
            PossiblePhrase::Phrase(phrase) => phrase.freq(),
        }
    }
}

impl Display for PossiblePhrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl From<Phrase> for PossiblePhrase {
    fn from(value: Phrase) -> Self {
        PossiblePhrase::Phrase(value.into())
    }
}

impl From<Symbol> for PossiblePhrase {
    fn from(value: Symbol) -> Self {
        PossiblePhrase::Symbol(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PossibleInterval {
    start: usize,
    end: usize,
    phrase: PossiblePhrase,
}

impl PossibleInterval {
    fn contains(&self, other: &PossibleInterval) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl From<PossibleInterval> for Interval {
    fn from(value: PossibleInterval) -> Self {
        Interval {
            start: value.start,
            end: value.end,
            is_phrase: match value.phrase {
                PossiblePhrase::Symbol(_) => false,
                PossiblePhrase::Phrase(_) => true,
            },
            phrase: value.phrase.to_string(),
        }
    }
}

#[derive(Default, Clone, Eq)]
struct PossiblePath {
    intervals: Vec<PossibleInterval>,
}

impl Debug for PossiblePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PossiblePath")
            .field("score()", &self.score())
            .field("intervals", &self.intervals)
            .finish()
    }
}

impl PossiblePath {
    fn score(&self) -> i32 {
        let mut score = 0;
        score += 1000 * self.rule_largest_sum();
        score += 1000 * self.rule_largest_avgwordlen();
        score += 100 * self.rule_smallest_lenvariance();
        score += self.rule_largest_freqsum();
        score
    }

    /// Copied from IsRecContain to trim some paths
    fn contains(&self, other: &Self) -> bool {
        let mut big = 0;
        for sml in 0..other.intervals.len() {
            loop {
                if big < self.intervals.len()
                    && self.intervals[big].start < other.intervals[sml].end
                {
                    if self.intervals[big].contains(&other.intervals[sml]) {
                        break;
                    }
                } else {
                    return false;
                }
                big += 1;
            }
        }
        true
    }

    fn rule_largest_sum(&self) -> i32 {
        let mut score = 0;
        for interval in &self.intervals {
            score += interval.end - interval.start;
        }
        score as i32
    }

    fn rule_largest_avgwordlen(&self) -> i32 {
        if self.intervals.is_empty() {
            return 0;
        }
        // Constant factor 6=1*2*3, to keep value as integer
        6 * self.rule_largest_sum()
            / i32::try_from(self.intervals.len()).expect("number of intervals should be small")
    }

    fn rule_smallest_lenvariance(&self) -> i32 {
        let len = self.intervals.len();
        let mut score = 0;
        // kcwu: heuristic? why variance no square function?
        for i in 0..len {
            for j in i + 1..len {
                let interval_1 = &self.intervals[i];
                let interval_2 = &self.intervals[j];
                score += interval_1.len().abs_diff(interval_2.len());
            }
        }
        i32::try_from(score).expect("score should fit in i32").neg()
    }

    fn rule_largest_freqsum(&self) -> i32 {
        let mut score = 0;
        for interval in &self.intervals {
            let reduction_factor = if interval.len() == 1 { 512 } else { 1 };
            score += interval.phrase.freq() / reduction_factor;
        }
        i32::try_from(score).expect("score should fit in i32")
    }
}

impl PartialEq for PossiblePath {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl PartialOrd for PossiblePath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

impl Ord for PossiblePath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score().cmp(&other.score())
    }
}

impl Display for PossiblePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#PossiblePath({}", self.score())?;
        for interval in &self.intervals {
            write!(
                f,
                " ({} {} '{})",
                interval.start, interval.end, interval.phrase
            )?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

type Graph<'a> = HashMap<(usize, usize), Option<PossiblePhrase>>;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        conversion::{Break, Composition, ConversionEngine, Interval, Symbol},
        dictionary::{Dictionary, Phrase},
        syl,
        zhuyin::Bopomofo::*,
    };

    use super::{ChewingEngine, PossibleInterval, PossiblePath};

    fn test_dictionary() -> impl Dictionary {
        HashMap::from([
            (vec![syl![G, U, O, TONE2]], vec![("國", 1).into()]),
            (vec![syl![M, I, EN, TONE2]], vec![("民", 1).into()]),
            (vec![syl![D, A, TONE4]], vec![("大", 1).into()]),
            (vec![syl![H, U, EI, TONE4]], vec![("會", 1).into()]),
            (vec![syl![D, AI, TONE4]], vec![("代", 1).into()]),
            (
                vec![syl![B, I, AU, TONE3]],
                vec![("表", 1).into(), ("錶", 1).into()],
            ),
            (
                vec![syl![G, U, O, TONE2], syl![M, I, EN, TONE2]],
                vec![("國民", 200).into()],
            ),
            (
                vec![syl![D, A, TONE4], syl![H, U, EI, TONE4]],
                vec![("大會", 200).into()],
            ),
            (
                vec![syl![D, AI, TONE4], syl![B, I, AU, TONE3]],
                vec![("代表", 200).into(), ("戴錶", 100).into()],
            ),
            (vec![syl![X, I, EN]], vec![("心", 1).into()]),
            (
                vec![syl![K, U, TONE4], syl![I, EN]],
                vec![("庫音", 300).into()],
            ),
            (
                vec![syl![X, I, EN], syl![K, U, TONE4], syl![I, EN]],
                vec![("新酷音", 200).into()],
            ),
            (
                vec![syl![C, E, TONE4], syl![SH, TONE4], syl![I, TONE2]],
                vec![("測試儀", 42).into()],
            ),
            (
                vec![syl![C, E, TONE4], syl![SH, TONE4]],
                vec![("測試", 9318).into()],
            ),
            (
                vec![syl![I, TONE2], syl![X, I, A, TONE4]],
                vec![("一下", 10576).into()],
            ),
            (vec![syl![X, I, A, TONE4]], vec![("下", 10576).into()]),
        ])
    }

    #[test]
    fn convert_empty_composition() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition {
            buffer: vec![],
            selections: vec![],
            breaks: vec![],
            glues: vec![],
        };
        assert_eq!(Vec::<Interval>::new(), engine.convert(&dict, &composition));
    }

    #[test]
    fn convert_simple_chinese_composition() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition {
            buffer: vec![
                Symbol::Syllable(syl![G, U, O, TONE2]),
                Symbol::Syllable(syl![M, I, EN, TONE2]),
                Symbol::Syllable(syl![D, A, TONE4]),
                Symbol::Syllable(syl![H, U, EI, TONE4]),
                Symbol::Syllable(syl![D, AI, TONE4]),
                Symbol::Syllable(syl![B, I, AU, TONE3]),
            ],
            selections: vec![],
            breaks: vec![],
            glues: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    is_phrase: true,
                    phrase: "代表".to_string()
                },
            ],
            engine.convert(&dict, &composition)
        );
    }

    #[test]
    fn convert_chinese_composition_with_breaks() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition {
            buffer: vec![
                Symbol::Syllable(syl![G, U, O, TONE2]),
                Symbol::Syllable(syl![M, I, EN, TONE2]),
                Symbol::Syllable(syl![D, A, TONE4]),
                Symbol::Syllable(syl![H, U, EI, TONE4]),
                Symbol::Syllable(syl![D, AI, TONE4]),
                Symbol::Syllable(syl![B, I, AU, TONE3]),
            ],
            selections: vec![],
            breaks: vec![Break(1), Break(5)],
            glues: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 1,
                    is_phrase: true,
                    phrase: "國".to_string()
                },
                Interval {
                    start: 1,
                    end: 2,
                    is_phrase: true,
                    phrase: "民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 5,
                    is_phrase: true,
                    phrase: "代".to_string()
                },
                Interval {
                    start: 5,
                    end: 6,
                    is_phrase: true,
                    phrase: "表".to_string()
                },
            ],
            engine.convert(&dict, &composition)
        );
    }

    #[test]
    fn convert_chinese_composition_with_good_selection() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition {
            buffer: vec![
                Symbol::Syllable(syl![G, U, O, TONE2]),
                Symbol::Syllable(syl![M, I, EN, TONE2]),
                Symbol::Syllable(syl![D, A, TONE4]),
                Symbol::Syllable(syl![H, U, EI, TONE4]),
                Symbol::Syllable(syl![D, AI, TONE4]),
                Symbol::Syllable(syl![B, I, AU, TONE3]),
            ],
            selections: vec![Interval {
                start: 4,
                end: 6,
                is_phrase: true,
                phrase: "戴錶".to_string(),
            }],
            breaks: vec![],
            glues: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    is_phrase: true,
                    phrase: "戴錶".to_string()
                },
            ],
            engine.convert(&dict, &composition)
        );
    }

    #[test]
    fn convert_chinese_composition_with_substring_selection() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition {
            buffer: vec![
                Symbol::Syllable(syl![X, I, EN]),
                Symbol::Syllable(syl![K, U, TONE4]),
                Symbol::Syllable(syl![I, EN]),
            ],
            selections: vec![Interval {
                start: 1,
                end: 3,
                is_phrase: true,
                phrase: "酷音".to_string(),
            }],
            breaks: vec![],
            glues: vec![],
        };
        assert_eq!(
            vec![Interval {
                start: 0,
                end: 3,
                is_phrase: true,
                phrase: "新酷音".to_string()
            },],
            engine.convert(&dict, &composition)
        );
    }

    #[test]
    fn multiple_single_word_selection() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition {
            buffer: vec![
                Symbol::Syllable(syl![D, AI, TONE4]),
                Symbol::Syllable(syl![B, I, AU, TONE3]),
            ],
            selections: vec![
                Interval {
                    start: 0,
                    end: 1,
                    is_phrase: true,
                    phrase: "代".to_string(),
                },
                Interval {
                    start: 1,
                    end: 2,
                    is_phrase: true,
                    phrase: "錶".to_string(),
                },
            ],
            breaks: vec![],
            glues: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 1,
                    is_phrase: true,
                    phrase: "代".to_string()
                },
                Interval {
                    start: 1,
                    end: 2,
                    is_phrase: true,
                    phrase: "錶".to_string()
                }
            ],
            engine.convert(&dict, &composition)
        );
    }

    #[test]
    fn convert_cycle_alternatives() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition {
            buffer: vec![
                Symbol::Syllable(syl![C, E, TONE4]),
                Symbol::Syllable(syl![SH, TONE4]),
                Symbol::Syllable(syl![I, TONE2]),
                Symbol::Syllable(syl![X, I, A, TONE4]),
            ],
            selections: vec![],
            breaks: vec![],
            glues: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    phrase: "測試".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    phrase: "一下".to_string()
                }
            ],
            engine.convert_next(&dict, &composition, 0)
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 3,
                    is_phrase: true,
                    phrase: "測試儀".to_string()
                },
                Interval {
                    start: 3,
                    end: 4,
                    is_phrase: true,
                    phrase: "下".to_string()
                }
            ],
            engine.convert_next(&dict, &composition, 1)
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    phrase: "測試".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    phrase: "一下".to_string()
                }
            ],
            engine.convert_next(&dict, &composition, 2)
        );
    }

    #[test]
    fn possible_path_contains() {
        let path_1 = PossiblePath {
            intervals: vec![
                PossibleInterval {
                    start: 0,
                    end: 2,
                    phrase: Phrase::new("測試", 0).into(),
                },
                PossibleInterval {
                    start: 2,
                    end: 4,
                    phrase: Phrase::new("一下", 0).into(),
                },
            ],
        };
        let path_2 = PossiblePath {
            intervals: vec![
                PossibleInterval {
                    start: 0,
                    end: 2,
                    phrase: Phrase::new("測試", 0).into(),
                },
                PossibleInterval {
                    start: 2,
                    end: 3,
                    phrase: Phrase::new("遺", 0).into(),
                },
                PossibleInterval {
                    start: 3,
                    end: 4,
                    phrase: Phrase::new("下", 0).into(),
                },
            ],
        };
        assert!(path_1.contains(&path_2));
    }
}
