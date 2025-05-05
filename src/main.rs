use parser_vsd::read_vsd::check_is_vsd;

pub mod constants;
mod VSDInternalStream;
mod parser_emf;
mod parser_vsd;
mod parser_vsdx;
mod VSDParser;

pub mod utils;

mod vsd_constants;

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

    let extention = (&*args[1]).to_lowercase();

    if (&extention).ends_with(String::from(".vsd").as_str()) && check_is_vsd(fname) {
        parser_vsd::read_vsd::read_file(fname);
        return 0;
    }

    if extention.ends_with(String::from(".vsdx").as_str()) {
        parser_vsdx::read_vsdx::read_file(fname, out_dir);
        return 0;
    }

    if extention.ends_with(String::from(".emf").as_str()) {
        parser_emf::read_emf::read_file(fname, out_dir);
        return 0;
    }

    println!("Unsupported file format");

    1
}
