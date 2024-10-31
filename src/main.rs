use pourover_rs::template;

fn main() {
    template::Template::build("test").unwrap();
    println!("Hello, World!");
}
