use std::{thread, time::Duration};

use theatre::{Actor, Echo};

#[inline]
fn sleep_ms(count: u64) {
    thread::sleep(Duration::from_millis(count));
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
    sleep_ms(200);
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

#[test]
fn test_death_wait() {
    let actor: Actor<i32> = Actor::graceful(Echo);

    let cloned_actor = actor.clone();
    thread::spawn(move || {
        sleep_ms(200);
        for k in 1..=5 {
            cloned_actor.tell(4 * k - 3).unwrap();
            sleep_ms(150);
        }
        cloned_actor.kill();
    });

    println!("Waiting for actor to die...");
    actor.wait();
}
