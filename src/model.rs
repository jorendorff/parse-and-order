use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Head(u32);

#[derive(Deserialize, Debug)]
pub struct Tail(u32);

#[derive(Deserialize, Debug)]
pub struct Leg {
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct Tongue;

#[derive(Deserialize, Debug)]
pub enum Animal {
    KimodoDragon,
    Aardvark(Tongue),
    WhiteTiger,
    Elephant,
    Kangaroo { legs: [Leg; 2], tail: Tail },
    Penguin,
    Langur,
    Giraffe,
    Springbok,
    Python(Head, Tail),
    Ruby,
    Flamingo,
    Panda,
}

#[derive(Deserialize, Debug)]
pub struct Zoo {
    animals: Vec<Animal>,
}
