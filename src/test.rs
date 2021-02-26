use std::thread;
use std::time::Duration;

mod lib;
use lib::Actor;
use lib::SuicidalActor;
use lib::SuicidalInterpreter;

pub struct CoutingIntepreter(u32);

impl SuicidalInterpreter<u32> for CoutingIntepreter {
    fn process(&mut self, message: u32) -> bool {
        println!(
            "It's the nb. {}, message I've received! \n  => {}",
            self.0, message
        );
        self.0 += 1;
        self.0 % 2 == 0
    }
}

fn main() {
    let counter = CoutingIntepreter(0);
    let actor = SuicidalActor::new(counter);
    actor.tell(2).unwrap();
    actor.tell(13).unwrap();
    actor.tell(7).unwrap();
    actor.tell(57).unwrap();
    thread::sleep(Duration::from_millis(3000));
}
