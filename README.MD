# nablo
a immediate mode gui framework written in rust, inspired by [egui](https://crates.io/crates/egui).    
currently `nablo` is under developing and may have break changes any time.    

# A Simple Example
```rust
// lets just start with a simple counter demo

use nablo::prelude::*;

// we should have a struct to save our important datas. though nablo has own memeory, but it isnt designed for long-time storge.
#[derive(Default)]
struct Counter {
    // feel free to add stuff here
    counter: i32,
}

// a app trait for your application
impl App for Counter {
    fn app(&mut self, ui: &mut Ui) {
        // there would be where you add widgets. check build-in widgets in widgets module.
        if ui.add(Button::new("+")).is_clicked() {
            self.counter += 1
        }
        if ui.add(Button::new("-")).is_clicked() {
            self.counter -= 1
        }
        ui.add(Label::new(format!("counter: {}", self.counter)));
     }
} 

fn main() {
    // nablo build-in window manager
    Manager::new(Counter::default()).run();
}
```