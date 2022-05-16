extern crate shared;

#[no_mangle]
pub fn view_state(state: &shared::State) {
    for &item in state.items.iter() {
        print!("{0} ", item);
    }
    println!();
}
