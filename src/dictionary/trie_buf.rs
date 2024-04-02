use std::{
    borrow::Cow,
    cmp,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    io, iter,
    path::PathBuf,
};

use crate::zhuyin::{Syllable, SyllableSlice};

use super::{
    BuildDictionaryError, DictEntries, Dictionary, DictionaryBuilder, DictionaryInfo,
    DictionaryUpdateError, Phrase, TrieDictionary, TrieDictionaryBuilder,
};

#[derive(Debug)]
pub(crate) struct TrieBufDictionary {
    path: PathBuf,
    trie: Option<TrieDictionary>,
    btree: BTreeMap<PhraseKey, (u32, u64)>,
    graveyard: BTreeSet<PhraseKey>,
}

type PhraseKey = (Cow<'static, [Syllable]>, Cow<'static, str>);

const MIN_PHRASE: &str = "";
const MAX_PHRASE: &str = "\u{10FFFF}";

impl TrieBufDictionary {
    pub(crate) fn open<P: Into<PathBuf>>(path: P) -> io::Result<TrieBufDictionary> {
        let path = path.into();
        if !path.exists() {
            let info = DictionaryInfo {
                name: Some("我的詞庫".to_string()),
                copyright: Some("Unknown".to_string()),
                license: Some("Unknown".to_string()),
                version: Some("0.0.0".to_string()),
                software: Some(format!(
                    "{} {}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                )),
            };
            let mut builder = TrieDictionaryBuilder::new();
            builder
                .set_info(info)
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
            builder
                .build(&path)
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        }
        let trie = TrieDictionary::open(&path)?;
        Ok(TrieBufDictionary {
            path,
            trie: Some(trie),
            btree: BTreeMap::new(),
            graveyard: BTreeSet::new(),
        })
    }

    pub(crate) fn new_in_memory() -> TrieBufDictionary {
        TrieBufDictionary {
            path: PathBuf::new(),
            trie: None,
            btree: BTreeMap::new(),
            graveyard: BTreeSet::new(),
        }
    }

    pub(crate) fn entries_iter_for<'a>(
        &'a self,
        syllables: &'a dyn SyllableSlice,
    ) -> impl Iterator<Item = Phrase> + 'a {
        let syllable_key = Cow::from(syllables.as_slice().into_owned());
        let min_key = (syllable_key.clone(), Cow::from(MIN_PHRASE));
        let max_key = (syllable_key.clone(), Cow::from(MAX_PHRASE));
        let store_iter = self
            .trie
            .iter()
            .flat_map(move |trie| trie.lookup_all_phrases(syllables));
        let btree_iter = self
            .btree
            .range(min_key..max_key)
            .map(|(key, value)| Phrase {
                phrase: key.1.clone().into(),
                freq: value.0,
                last_used: Some(value.1),
            });

        store_iter.chain(btree_iter).filter(move |it| {
            !self
                .graveyard
                .contains(&(syllable_key.clone(), Cow::from(it.as_str())))
        })
    }

    pub(crate) fn entries_iter(&self) -> impl Iterator<Item = (Vec<Syllable>, Phrase)> + '_ {
        let mut trie_iter = self.trie.iter().flat_map(|trie| trie.entries()).peekable();
        let mut btree_iter = self
            .btree
            .iter()
            .map(|(key, value)| {
                (
                    key.0.clone().into_owned(),
                    Phrase {
                        phrase: key.1.clone().into(),
                        freq: value.0,
                        last_used: Some(value.1),
                    },
                )
            })
            .peekable();
        iter::from_fn(move || {
            let a = trie_iter.peek();
            let b = btree_iter.peek();
            match (a, b) {
                (None, Some(_)) => btree_iter.next(),
                (Some(_), None) => trie_iter.next(),
                (Some(x), Some(y)) => match (&x.0, x.1.as_str()).cmp(&(&y.0, y.1.as_str())) {
                    cmp::Ordering::Less => trie_iter.next(),
                    cmp::Ordering::Equal => match x.1.freq.cmp(&y.1.freq) {
                        cmp::Ordering::Less | cmp::Ordering::Equal => {
                            let _ = trie_iter.next();
                            btree_iter.next()
                        }
                        cmp::Ordering::Greater => {
                            let _ = btree_iter.next();
                            trie_iter.next()
                        }
                    },
                    cmp::Ordering::Greater => btree_iter.next(),
                },
                (None, None) => None,
            }
        })
        .filter(|it| {
            !self
                .graveyard
                .contains(&(Cow::from(it.0.as_slice()), Cow::from(it.1.as_str())))
        })
    }

    pub(crate) fn lookup_first_n_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        first: usize,
    ) -> Vec<Phrase> {
        let mut sort_map = BTreeMap::new();
        let mut phrases: Vec<Phrase> = Vec::new();

        for phrase in self.entries_iter_for(syllables) {
            match sort_map.entry(phrase.to_string()) {
                Entry::Occupied(entry) => {
                    let index = *entry.get();
                    phrases[index] = cmp::max(&phrase, &phrases[index]).clone();
                }
                Entry::Vacant(entry) => {
                    entry.insert(phrases.len());
                    phrases.push(phrase);
                }
            }
        }
        phrases.truncate(first);
        phrases
    }

    pub(crate) fn entries(&self) -> DictEntries<'_> {
        Box::new(self.entries_iter().into_iter())
    }

    pub(crate) fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_slice = syllables.as_slice();
        if self
            .entries_iter_for(&syllable_slice.as_ref())
            .any(|ph| ph.as_str() == phrase.as_str())
        {
            return Err(DictionaryUpdateError { source: None });
        }

        self.btree.insert(
            (
                Cow::from(syllable_slice.into_owned()),
                Cow::from(phrase.phrase.into_string()),
            ),
            (phrase.freq, phrase.last_used.unwrap_or_default()),
        );

        Ok(())
    }

    pub(crate) fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        self.btree.insert(
            (
                Cow::from(syllables.as_slice().into_owned()),
                Cow::from(phrase.phrase.into_string()),
            ),
            (user_freq, time),
        );

        Ok(())
    }

    pub(crate) fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_slice = Cow::from(syllables.as_slice().into_owned());
        self.btree
            .remove(&(syllable_slice.clone(), Cow::from(phrase_str.to_owned())));
        self.graveyard
            .insert((syllable_slice, phrase_str.to_owned().into()));
        Ok(())
    }
}

impl From<BuildDictionaryError> for DictionaryUpdateError {
    fn from(value: BuildDictionaryError) -> Self {
        DictionaryUpdateError {
            source: Some(Box::new(value)),
        }
    }
}

impl Dictionary for TrieBufDictionary {
    fn lookup_first_n_phrases(&self, syllables: &dyn SyllableSlice, first: usize) -> Vec<Phrase> {
        TrieBufDictionary::lookup_first_n_phrases(self, syllables, first)
    }

    fn entries(&self) -> DictEntries<'_> {
        TrieBufDictionary::entries(self)
    }

    fn about(&self) -> DictionaryInfo {
        self.trie
            .as_ref()
            .map_or(DictionaryInfo::default(), |trie| trie.about())
    }

    fn reopen(&mut self) -> Result<(), DictionaryUpdateError> {
        if !self.path.as_os_str().is_empty() {
            self.trie = Some(TrieDictionary::open(&self.path)?);
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), DictionaryUpdateError> {
        if self.path.as_os_str().is_empty() {
            return Ok(());
        }
        let mut builder = TrieDictionaryBuilder::new();
        builder.set_info(self.about())?;
        for (syllables, phrase) in self.entries() {
            builder.insert(&syllables, phrase)?;
        }
        builder.build(&self.path)?;
        self.btree.clear();
        self.graveyard.clear();
        self.reopen()
    }

    fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        TrieBufDictionary::add_phrase(self, syllables, phrase)
    }

    fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        TrieBufDictionary::update_phrase(self, syllables, phrase, user_freq, time)
    }

    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        TrieBufDictionary::remove_phrase(self, syllables, phrase_str)
    }
}

impl<const N: usize> From<[(Vec<Syllable>, Vec<Phrase>); N]> for TrieBufDictionary {
    fn from(value: [(Vec<Syllable>, Vec<Phrase>); N]) -> Self {
        let mut dict = TrieBufDictionary::new_in_memory();
        for (syllables, phrases) in value {
            for phrase in phrases {
                dict.add_phrase(&syllables, phrase).unwrap();
            }
        }
        dict
    }
}

impl Drop for TrieBufDictionary {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{dictionary::Phrase, syl, zhuyin::Bopomofo::*};

    use super::{Dictionary, TrieBufDictionary};

    #[test]
    fn create_new_dictionary_in_memory_and_query() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("user.dat");
        let mut dict = TrieBufDictionary::open(file_path)?;
        let info = dict.about();
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        assert_eq!(Some("Unknown".to_string()), info.copyright);
        assert_eq!(
            Some(("dict", 1, 2).into()),
            dict.lookup_first_phrase(&[syl![Z, TONE4], syl![D, I, AN, TONE3]])
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_and_query() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("user.dat");
        let mut dict = TrieBufDictionary::open(&file_path)?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        dict.flush()?;
        let dict = TrieBufDictionary::open(file_path)?;
        let info = dict.about();
        assert_eq!(Some("Unknown".to_string()), info.copyright);
        assert_eq!(
            Some(("dict", 1, 2).into()),
            dict.lookup_first_phrase(&[syl![Z, TONE4], syl![D, I, AN, TONE3]])
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_and_enumerate() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("user.dat");
        let mut dict = TrieBufDictionary::open(file_path)?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        dict.flush()?;
        assert_eq!(
            vec![(
                vec![syl![Z, TONE4], syl![D, I, AN, TONE3]],
                Phrase::from(("dict", 1, 2))
            )],
            dict.entries().collect::<Vec<_>>()
        );
        Ok(())
    }
}
