use clap::Parser;

mod args;
mod scanner;

fn main() {
    let args = args::Args::parse();
    let result = scanner::scan_files(&args.target, args.deep, args.all);
    // println!("{:#?}", result);
    println!("{}", scanner::FileInTree(&result));
    // println!("{}", scanner::FileWithName(&result));
}
