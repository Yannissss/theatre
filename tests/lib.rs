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
    actor.kill().unwrap();
    actor.wait();
}

#[test]
fn test_graceful() {
    let actor: Actor<i32> = Actor::graceful(Echo);
    actor.tell(0).unwrap();
    actor.kill().unwrap();
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
    actor.kill().unwrap();
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
    actor.kill().unwrap();
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
        cloned_actor.kill().unwrap();
    });

    println!("Waiting for actor to die...");
    actor.wait();
}

#[test]
fn test_instant_death_graceful() {
    let actor: Actor<i32> = Actor::graceful(Echo);
    let cloned = actor.clone();
    thread::spawn(move || {
        cloned.tell(0).unwrap();
    });
    actor.wait();
}

#[test]
fn test_instant_death_disgraceful() {
    // This should instanly return
    let actor: Actor<i32> = Actor::disgraceful(Echo);
    actor.wait();
}

#[test]
#[should_panic]
fn test_weak_cloning() {
    let actor: Actor<i32> = Actor::graceful(Echo);
    let cloned = actor.weak();
    cloned.tell(0).unwrap();
}
