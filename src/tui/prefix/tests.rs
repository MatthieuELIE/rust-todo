use super::*;

#[test]
fn each_prefix_gives_the_key_that_starts_its_command() {
    let keys = [Prefix::D, Prefix::G, Prefix::P, Prefix::Y, Prefix::Z].map(Prefix::key);

    assert_eq!(keys, ['d', 'g', 'p', 'y', 'z']);
}
