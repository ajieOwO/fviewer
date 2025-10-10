mod files;
fn main() {
    let result = files::scan_files("src");
    println!("{:#?}", result);
}
