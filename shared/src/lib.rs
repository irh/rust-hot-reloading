pub trait Rng {
    fn gen_range(&mut self, low: i32, high: i32) -> i32;
}

pub struct State {
    pub rng: Box<dyn Rng>,
    pub items: Vec<i32>,
}

impl State {
    pub fn new(rng: Box<dyn Rng>) -> State {
        State {
            rng,
            items: Default::default(),
        }
    }
}
