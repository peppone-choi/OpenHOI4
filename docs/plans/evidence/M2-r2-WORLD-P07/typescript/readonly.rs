use std::io::Write; fn main() { std::io::stdout().write_all(oh_proto::typescript().as_bytes()).unwrap(); }
