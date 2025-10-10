mod files;
fn main() {
    let result = files::scan_files("src", 1);
    println!("{:#?}", result);
}
