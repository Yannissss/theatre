use std::thread;
use std::time::Duration;

use theatre::{Actor, Echo};

fn sleep_millis(count: u64) {
    thread::sleep(Duration::from_millis(count))
}

#[test]
fn test_single_send() {
    let actor: Actor<&'static str> = Actor::graceful(Echo);
    actor.tell("Hello, World!").unwrap();
    actor.kill();
    actor.wait();
}

#[test]
fn test_graceful() {
    let actor: Actor<i32> = Actor::graceful(Echo);
    actor.tell(0).unwrap();
    actor.kill();
    for k in 1..=10 {
        actor.tell(k).unwrap();
    }
    actor.wait();
}

#[test]
fn test_disgraceful() {
    let actor: Actor<i32> = Actor::disgraceful(Echo);
    actor.tell(0).unwrap();
    sleep_millis(100);
    actor.kill();
    for k in 1..=10 {
        actor.tell(k).unwrap();
    }
    actor.wait();
}

#[test]
fn test_blanket_interpreters() {
    let actor: Actor<i32> = Actor::graceful(|x| {
        println!("Got x = {}", x);
    });
    for k in 1..=10 {
        actor.tell(k).unwrap();
    }
    actor.kill();
    actor.wait();
}
