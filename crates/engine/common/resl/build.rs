fn main() {
    match lalrpop::process_root() {
        Ok(_) => {}
        Err(err) => {
            println!("rust:warning={:?}", err);
            panic!("{:?}", err)
        }
    }
}