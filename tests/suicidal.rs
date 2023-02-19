use std::thread;
use std::time::Duration;

use theatre::Actor;
use theatre::SuicidalActor;
use theatre::SuicidalInterpreter;

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

#[test]
#[should_panic]
fn disgracefully_close() {
    let counter = CoutingIntepreter(0);
    let actor = SuicidalActor::new(counter);
    actor.tell(3).unwrap();
    actor.tell(5).unwrap();
    // Actor will kill himself after receiving an even number
    actor.tell(2).unwrap();
    thread::sleep(Duration::from_millis(500));
    // Actor should be dead so this should fail
    actor.tell(7).unwrap();
}
