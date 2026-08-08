// Emits the esp-idf build environment (link args, include paths) to cargo.
fn main() {
    embuild::espidf::sysenv::output();
}
