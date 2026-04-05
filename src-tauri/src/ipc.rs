pub mod client {
    use serde::Serialize;

    pub fn send<T: Serialize>(value: T) {
        match serde_json::to_string(&value) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Serialization error: {}", e),
        }
    }
}