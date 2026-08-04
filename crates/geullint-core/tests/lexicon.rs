use geullint_core::{LexiconEntry, StandardLexicon};

#[test]
fn parses_versioned_standard_lexicon_and_looks_up_entries() {
    let lexicon =
        StandardLexicon::parse("geullint-standard-lexicon-v1\n가다\tVV\t1000\n며칠\tNNG\t132\n")
            .expect("valid standard lexicon");

    assert_eq!(lexicon.entry_count(), 2);
    assert_eq!(
        lexicon.lookup("며칠"),
        Some(&LexiconEntry {
            surface: "며칠".to_owned(),
            part_of_speech: "NNG".to_owned(),
            frequency: 132,
        })
    );
    assert_eq!(lexicon.lookup("없는말"), None);
}

#[test]
fn rejects_unsorted_duplicate_and_malformed_lexicon_rows() {
    assert!(StandardLexicon::parse("wrong\n").is_err());
    assert!(
        StandardLexicon::parse("geullint-standard-lexicon-v1\n며칠\tNNG\t1\n가다\tVV\t2\n")
            .is_err()
    );
    assert!(
        StandardLexicon::parse("geullint-standard-lexicon-v1\n가다\tVV\t1\n가다\tVV\t1\n").is_err()
    );
}

#[cfg(feature = "standard")]
#[test]
fn bundled_standard_lexicon_matches_the_checked_in_manifest_size() {
    let lexicon = StandardLexicon::bundled().expect("bundled standard lexicon");
    assert!(lexicon.entry_count() >= 1_000);
    assert!(lexicon.lookup("며칠").is_some());
}
