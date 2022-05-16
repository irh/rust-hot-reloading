extern crate shared;
use shared::State;

#[no_mangle]
pub fn update_state(state: &mut State) {
    let n = match state.items.last() {
        Some(x) => x + state.rng.gen_range(-100, 101),
        None => 0,
    };

    state.items.push(n);

    while state.items.len() > 10 {
        state.items.remove(0);
    }
}
