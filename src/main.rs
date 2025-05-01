pub mod constants;
mod helpers;
mod parser_vsd;
mod parser_vsdx;
pub mod utils;
mod  vsd_constants;

fn main() {

    std::process::exit(job());
}

fn job() -> i32 {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        println!("need more args");
        return 1;
    }
    let fname = std::path::Path::new(&*args[1]);
    let out_dir = std::path::Path::new(&*args[2]);

    if (&*args[1])
        .to_lowercase()
        .ends_with(String::from(".vsd").as_str())
    {
        parser_vsd::read_vsd::read_file(fname);
        return 0;
    }

    if (&*args[1])
        .to_lowercase()
        .ends_with(String::from(".vsdx").as_str())
    {
        parser_vsdx::read_vsdx::read_file(fname, out_dir);
        return 0;
    }

    println!("Unsapported file format");

    1
}
