mod scanner;

fn main() {
    let result = scanner::scan_files("..", 1);
    println!("{}", scanner::FileInTree(&result));
}
