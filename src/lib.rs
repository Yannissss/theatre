use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvError, Sender},
        Arc, Condvar, Mutex,
    },
    thread,
};
use thiserror::Error;

/// Possible errors when dealing with actors
#[derive(Error, Debug)]
pub enum ActingErr {
    #[error("Actor that was contacted is dead!")]
    DeadActor,
}

/// Trait that describe that a stateful interpreter
/// of messages
pub trait Interpreter<M> {
    fn interpret(&mut self, message: M);
}

/// Dummy interpreter that simply echoes the messages it receives
#[derive(Clone, Copy, Debug, Default)]
pub struct Echo;

impl<M> Interpreter<M> for Echo
where
    M: Display,
{
    fn interpret(&mut self, message: M) {
        println!("Echo: {}", message);
    }
}

/// Blank implementation for all functions
/// Similear to Fn(M) -> ()
impl<M, F> Interpreter<M> for F
where
    F: Fn(M),
{
    fn interpret(&mut self, message: M) {
        self(message)
    }
}

/// An actor which process messages of type M
/// Internally it actually forks an OS thread
/// that listens to incoming message and process
/// them
pub struct Actor<M> {
    channel: Sender<Option<M>>,
    should_die: Arc<AtomicBool>,
    till_death: Arc<Condvar>,
    is_dead: Arc<Mutex<bool>>,
}

impl<M> Clone for Actor<M> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
            should_die: self.should_die.clone(),
            till_death: self.till_death.clone(),
            is_dead: self.is_dead.clone(),
        }
    }
}

impl<M> Actor<M>
where
    M: 'static + Send,
{
    /// Consumes all notification until it is notifier to die
    fn loop_until_killed<I>(&self, interpreter: &mut I, consumer: &Receiver<Option<M>>)
    where
        I: Interpreter<M>,
    {
        loop {
            // Check if it has to die
            if self.should_die.load(Ordering::SeqCst) {
                break;
            }
            // Wait for an incoming message
            match consumer.recv() {
                // All channels were closed
                Err(RecvError) => break,
                // Someone request that actor dies
                Ok(None) => break,
                // Process the message
                Ok(Some(message)) => {
                    // Otherwise process the message
                    interpreter.interpret(message);
                }
            }
        }
    }

    /// Cleanup all pending messages
    fn cleanup<I>(&self, interpreter: &mut I, consumer: &Receiver<Option<M>>)
    where
        I: Interpreter<M>,
    {
        while let Ok(message) = consumer.try_recv() {
            match message {
                None => (), // Do nothing since message was a kill signal
                Some(message) => interpreter.interpret(message),
            }
        }
    }

    /// Dies & notify all waiters
    fn die(self) {
        let till_death = self.till_death.clone();
        let is_dead = self.is_dead.clone();
        drop(self);

        let mut guard = is_dead.lock().unwrap();
        *guard = true;
        till_death.notify_all();
    }

    /// Creates and returns and actor that gracefully
    /// process all pending messages when asked to die
    pub fn graceful<I>(interpreter: I) -> Self
    where
        I: Interpreter<M> + Send + 'static,
    {
        let (channel, consumer) = mpsc::channel();
        let actor = Self {
            channel,
            should_die: Arc::new(AtomicBool::new(false)),
            till_death: Arc::new(Condvar::default()),
            is_dead: Arc::new(Mutex::new(false)),
        };
        let cloned_actor = actor.clone();

        let mut interpreter = interpreter;
        thread::spawn(move || {
            // Main message handling loop
            actor.loop_until_killed(&mut interpreter, &consumer);
            // Cleaning up
            actor.cleanup(&mut interpreter, &consumer);
            // Dies & notify all waiters
            actor.die();
        });

        cloned_actor
    }

    /// Creates and returns and actor that disgracefully
    /// ignore all pending messages when asked to die
    pub fn disgraceful<I>(interpreter: I) -> Self
    where
        I: Interpreter<M> + Send + 'static,
    {
        let (channel, consumer) = mpsc::channel();
        let actor = Self {
            channel,
            should_die: Arc::new(AtomicBool::new(false)),
            till_death: Arc::new(Condvar::default()),
            is_dead: Arc::new(Mutex::new(false)),
        };
        let cloned_actor = actor.clone();

        let mut interpreter = interpreter;
        thread::spawn(move || {
            // Main message handling loop
            actor.loop_until_killed(&mut interpreter, &consumer);
            // Cleaning up
            // No cleaning up since we are disgraceful
            // Dies & notify all waiters
            actor.die();
        });

        cloned_actor
    }

    /// Creates and returns and actor that gracefully
    /// process all pending messages when asked to die
    pub fn suicidal() -> Self {
        unimplemented!()
    }

    /// Waits indefinitely until the actor is declared dead
    /// Renounce on its ability to send messages
    /// Will deadlock if actor is alread dead
    pub fn wait(self) {
        let till_death = self.till_death.clone();
        let is_dead = self.is_dead.clone();
        drop(self);

        // Check is actor is not already dead
        // and wait for the notification
        let mut guard = is_dead.lock().unwrap();
        while !*guard {
            guard = till_death.wait(guard).unwrap();
        }
    }

    /// Send a message to an actor
    pub fn tell(&self, message: M) -> Result<(), ActingErr> {
        self.channel
            .send(Some(message))
            .map_err(|_| ActingErr::DeadActor)
    }

    /// Kills an actor
    /// This function returns `Ok(())` if it succesfully kills it
    /// And error explaining the reason it could not do if not
    /// Trying to kill a dead actor does not do anything
    pub fn kill(&self) {
        self.should_die.store(true, Ordering::SeqCst);
        // Signal the actor that he has to die
        match self.channel.send(None) {
            Err(_) => (), // Actor already dead so do nothing
            Ok(_) => (),  // Sent a dummy message to process
        }
    }
}
