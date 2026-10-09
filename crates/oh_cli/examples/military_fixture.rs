//! Generate a temporary real military pack for native/server acceptance.
#[path = "../tests/support/military.rs"]
mod military;
fn main() {
    println!("{}", military::pack().display());
}
