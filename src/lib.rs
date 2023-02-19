use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvError},
        Arc,
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
#[derive(Clone)]
pub struct Actor<M> {
    channel: mpsc::Sender<Option<M>>,
    should_die: Arc<AtomicBool>,
}

impl<M> Actor<M>
where
    M: 'static + Send,
{
    /// Creates and returns and actor that gracefully
    /// process all pending messages when asked to die
    pub fn graceful<I>(interpreter: I) -> Self
    where
        I: Interpreter<M> + Send + 'static,
    {
        let (channel, consumer) = mpsc::channel();
        let should_die = Arc::new(AtomicBool::new(false));

        let mut interpreter = interpreter;
        let thread_should_die = should_die.clone();
        thread::spawn(move || {
            // Main message handling loop
            loop {
                // Check if it has to die
                if thread_should_die.load(Ordering::SeqCst) {
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
            // Cleaning up
            while let Ok(message) = consumer.recv() {
                match message {
                    None => (), // Do nothing since message was a kill signal
                    Some(message) => interpreter.interpret(message),
                }
            }
        });

        Self {
            channel,
            should_die,
        }
    }

    /// Creates and returns and actor that disgracefully
    /// ignore all pending messages when asked to die
    pub fn disgraceful<I>(interpreter: I) -> Self
    where
        I: Interpreter<M> + Send + 'static,
    {
        let (channel, consumer) = mpsc::channel();
        let should_die = Arc::new(AtomicBool::new(false));

        let mut interpreter = interpreter;
        let thread_should_die = should_die.clone();
        thread::spawn(move || {
            // Main message handling loop
            loop {
                // Check if it has to die
                if thread_should_die.load(Ordering::SeqCst) {
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
            // Cleaning up
            while let Ok(message) = consumer.recv() {
                match message {
                    None => (), // Do nothing since message was a kill signal
                    Some(message) => interpreter.interpret(message),
                }
            }
        });

        Self {
            channel,
            should_die,
        }
    }

    /// Creates and returns and actor that gracefully
    /// process all pending messages when asked to die
    pub fn suicidal() -> Self {
        unimplemented!()
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
        self.should_die.store(true, Ordering::Relaxed);
        // Signal the actor that he has to die
        match self.channel.send(None) {
            Err(_) => (), // Actor already dead so do nothing
            Ok(_) => (),  // Sent a dummy message to process
        }
    }
}
