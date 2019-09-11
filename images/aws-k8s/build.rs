fn main() {
    if let Err(e) = buildsys::build_image() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
