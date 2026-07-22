#[cfg(any(feature = "wpa2-personal", feature = "wpa3-personal"))]
use hisi_rf::ws63::{SelectedProfile, Storage};

#[cfg(any(feature = "wpa2-personal", feature = "wpa3-personal"))]
fn main() {
    let storage = Storage::<SelectedProfile, 4>::new();
    let mut output = String::new();
    storage
        .report()
        .write_json(&mut output)
        .expect("String writes are infallible");
    println!("{output}");
}

#[cfg(not(any(feature = "wpa2-personal", feature = "wpa3-personal")))]
fn main() {
    panic!("select one named WS63 Wi-Fi profile to emit its resource report");
}
