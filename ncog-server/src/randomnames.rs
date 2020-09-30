use rand::{seq::SliceRandom, thread_rng};

const ADJECTIVES: [&'static str; 2] = ["Silly", "Crazy"];
const NOUNS: [&'static str; 2] = ["Dog", "Cat"];

pub fn random_name() -> String {
    let mut rng = thread_rng();
    format!(
        "{}{}",
        ADJECTIVES.choose(&mut rng).unwrap(),
        NOUNS.choose(&mut rng).unwrap()
    )
}
