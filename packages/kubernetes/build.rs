fn main() {
    if let Err(e) = buildsys::build_package() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
