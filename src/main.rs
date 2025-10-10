mod scanner;
fn main() {
    let result = scanner::scan_files("src", 1);
    println!("{:#?}", result);
}
