use evdev::{enumerate, EventType};
use std::io;

#[test]
fn test_raw_events() -> io::Result<()> {
    for (path, mut dev) in enumerate() {
        let name = dev.name().unwrap_or("").to_string();
        if name.contains("BY Tech") || name.contains("MAD 8K") {
            println!("Testing device {:?} ('{}')", path, name);
            dev.set_nonblocking(true)?;
            match dev.fetch_events() {
                Ok(_) => println!("Ok"),
                Err(e) => println!("Error kind: {:?}, raw: {}", e.kind(), e),
            }
        }
    }
    Ok(())
}
