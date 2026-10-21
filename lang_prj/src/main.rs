use std::io;
fn main() {
    println!("Put text here.");
    let mut input:String = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    println!("You typed: {}", input);
}