fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&oh_data::host_policy::schema()).unwrap()
    );
}
