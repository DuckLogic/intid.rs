#![allow(missing_docs)]
use intid_derive::{EnumId, IntegerId};

use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_test::{assert_tokens, Token};

use idmap::{enum_map, EnumMap};
use KnownState::*;

#[test]
fn remove() {
    let mut m = important_cities();
    assert_eq!(m.remove(NewMexico), None);
    for state in IMPORTANT_STATES {
        assert_eq!(Some(state.city()), m.remove(state), "{m:#?}");
    }
    assert_eq!(m.len(), 0);
    assert_eq!(m.remove(NewMexico), None);
    assert_eq!(m.remove(NorthDakota), None);
}

#[test]
fn eq() {
    let first = important_cities();
    let second = important_cities()
        .into_iter()
        .rev()
        .collect::<EnumMap<_, _>>();

    assert_eq!(first, second);
}

#[test]
fn from_iter() {
    let xs = [
        (California, "San Diego"),
        (NewYork, "New York"),
        (Arizona, "Phoenix"),
    ];

    let map: EnumMap<_, _> = xs.iter().copied().collect();

    for &(k, v) in &xs {
        assert_eq!(map.get(k), Some(&v));
    }
    check_missing(TINY_STATES, &map);
}

#[test]
fn clone() {
    let original = important_cities();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn index() {
    let map = important_cities();

    for state in IMPORTANT_STATES {
        assert_eq!(map[state], state.city());
    }
}

#[test]
fn declaration_order() {
    let map = important_cities();
    let actual_entries = map
        .iter()
        .map(|(state, &city)| (state, city))
        .collect::<Vec<_>>();
    let declared_entries = actual_entries.iter().copied().sorted().collect_vec();
    assert_eq!(actual_entries, declared_entries);
    let reversed_map = actual_entries
        .iter()
        .rev()
        .copied()
        .collect::<EnumMap<KnownState, &'static str>>();
    let reversed_entries = reversed_map
        .iter()
        .map(|(state, &city)| (state, city))
        .collect::<Vec<_>>();
    assert_eq!(reversed_entries, declared_entries);
}

#[test]
#[should_panic = "index out of bounds"]
#[allow(clippy::no_effect)] // It's supposed to panic
fn index_nonexistent() {
    let map = important_cities();

    map[NorthDakota];
}

#[test]
#[cfg(any())] // TODO: Support entry API?
fn entry_insert() {
    let mut map = important_cities();

    for &state in ALL_STATES {
        let value = *map.entry(state).or_insert_with(|| state.city());
        assert_eq!(value, state.city());
    }
    check_cities(ALL_STATES, &map);
}

#[test]
fn extend_ref() {
    let important = important_cities();
    let mut all = EnumMap::new();
    all.insert(NewMexico, "Albuquerque");
    all.insert(California, "San Diego");
    all.insert(NorthDakota, "Fargo");

    all.extend(&important);

    assert_eq!(all.len(), 5);
    // Updates must remain in declaration order
    assert_eq!(all.iter().nth(1).unwrap(), (California, &California.city()));
    assert_eq!(all.iter().nth(4).unwrap(), (NorthDakota, &"Fargo"));
    check_cities(ALL_STATES, &all);
}

#[test]
fn retain() {
    let mut map = important_cities();
    map.retain(|state, _| match state {
        NewYork => false, // New york city is too big!
        California | Arizona => true,
        _ => unreachable!(),
    });
    assert_eq!(map.len(), 2);
    check_cities(&[Arizona, California], &map);
    check_missing(TINY_STATES, &map);
}

/// List the biggest cities in each state except for `NewMexico` and `NorthDakota`,
/// intentionally excluding them to provide a better test case.
fn important_cities() -> EnumMap<KnownState, &'static str> {
    // NOTE: Intentionally out of declared order to try and mess things up
    enum_map! {
        Arizona => "Phoenix",
        NewYork => "New York City",
        California => "Los Angeles"
    }
}
#[derive(
    IntegerId, EnumId, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Eq,
)]
enum KnownState {
    Arizona,
    California,
    NewMexico,
    NewYork,
    NorthDakota,
}
fn check_missing(states: &[KnownState], target: &EnumMap<KnownState, &'static str>) {
    for state in states {
        state.check_missing(target);
    }
}
fn check_cities(states: &[KnownState], target: &EnumMap<KnownState, &'static str>) {
    for state in states {
        state.check_city(target);
    }
}
static ALL_STATES: &[KnownState] = &[Arizona, California, NewMexico, NewYork, NorthDakota];
// NOTE: Intentionally out of declared order to try and mess things up

static IMPORTANT_STATES: &[KnownState] = &[Arizona, NewYork, California];
static TINY_STATES: &[KnownState] = &[NorthDakota, NewMexico];
impl KnownState {
    fn city(self) -> &'static str {
        match self {
            Arizona => "Phoenix",
            California => "Los Angeles",
            NewMexico => "Albuquerque",
            NewYork => "New York City",
            NorthDakota => "Fargo",
        }
    }
    fn check_missing(self, target: &EnumMap<KnownState, &'static str>) {
        assert_eq!(target.get(self), None, "Expected no city for {self:?}");
    }
    fn check_city(self, target: &EnumMap<KnownState, &'static str>) {
        assert_eq!(
            target.get(self),
            Some(&self.city()),
            "Unexpected city for {self:?}"
        );
    }
}

#[test]
#[cfg(feature = "serde")]
fn serde() {
    macro_rules! state_tokens {
        ($len:expr, $($state:expr => $city:expr),*) => (&[
            Token::Map { len: Some($len) },
            $(
                Token::Enum { name: "KnownState" },
                Token::Str(stringify!($state)),
                Token::Unit,
                Token::BorrowedStr($city),
            )*
            Token::MapEnd
        ]);
    }
    // Remember, EnumMap is in _declaration order_
    const EXPECTED_TOKENS: &[Token] = state_tokens!(3,
        Arizona => "Phoenix",
        California => "Los Angeles",
        NewYork => "New York City"
    );
    assert_tokens(&important_cities(), EXPECTED_TOKENS);
}
