use std::sync;
use std::sync::mpsc;
use std::thread;

#[derive(Clone, Debug)]
pub enum ActorError<T> {
    Immortal,
    Unsendable(T),
}

pub trait Actor<T> {
    fn tell(&self, message: T) -> Result<(), ActorError<T>>;

    fn kill(self) -> Result<(), ActorError<T>>;
}

pub trait Interpreter<T> {
    fn process(&mut self, message: T);
}

pub trait SuicidalInterpreter<T> {
    fn process(&mut self, message: T) -> bool;
}

#[derive(Clone)]
pub struct BaseActor<T> {
    sender: mpsc::Sender<Option<T>>,
    should_die: sync::Arc<sync::Mutex<bool>>,
}

impl<T> Actor<T> for BaseActor<T> {
    fn tell(&self, message: T) -> Result<(), ActorError<T>> {
        match self.sender.send(Some(message)) {
            Err(mpsc::SendError(Some(message))) => Err(ActorError::Unsendable(message)),
            _ => Ok(()),
        }
    }

    fn kill(self) -> Result<(), ActorError<T>> {
        match self.should_die.lock() {
            Err(_) => Err(ActorError::Immortal),
            Ok(mut should_die) => {
                *should_die = true;
                match self.sender.send(None) {
                    Ok(()) => Ok(()),
                    _ => Err(ActorError::Immortal),
                }
            }
        }
    }
}

/// Actor that gracefully interpretes all pending messages before letting itself be killed
#[derive(Clone)]
pub struct GracefulActor<T>(BaseActor<T>);

impl<T> Actor<T> for GracefulActor<T> {
    fn tell(&self, message: T) -> Result<(), ActorError<T>> {
        self.0.tell(message)
    }

    fn kill(self) -> Result<(), ActorError<T>> {
        self.0.kill()
    }
}

#[allow(dead_code)]
impl<T> GracefulActor<T>
where
    T: Send + 'static,
{
    pub fn new<I>(mut interpreter: I) -> GracefulActor<T>
    where
        I: Interpreter<T>,
        I: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let should_die = sync::Arc::new(sync::Mutex::new(false));

        let local_should_die = should_die.clone();
        thread::spawn(move || loop {
            let next = receiver.recv().unwrap();
            let should_die = local_should_die.lock().unwrap();
            match next {
                None => break,
                Some(message) => interpreter.process(message),
            }
            if *should_die {
                // Die gracefull
                loop {
                    match receiver.try_recv() {
                        Ok(Some(message)) => interpreter.process(message),
                        Err(mpsc::TryRecvError::Empty) => break,
                        _ => (),
                    }
                }
                break;
            }
        });

        Self(BaseActor { sender, should_die })
    }
}

/// Actor that disgracefully ignores all pending messages before letting itself be killed
#[derive(Clone)]
pub struct DisgracefulActor<T>(BaseActor<T>);

impl<T> Actor<T> for DisgracefulActor<T> {
    fn tell(&self, message: T) -> Result<(), ActorError<T>> {
        self.0.tell(message)
    }

    fn kill(self) -> Result<(), ActorError<T>> {
        self.0.kill()
    }
}

#[allow(dead_code)]
impl<T> DisgracefulActor<T>
where
    T: Send + 'static,
{
    pub fn new<I>(mut interpreter: I) -> DisgracefulActor<T>
    where
        I: Interpreter<T>,
        I: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let should_die = sync::Arc::new(sync::Mutex::new(false));

        let local_should_die = should_die.clone();
        thread::spawn(move || loop {
            let next = receiver.recv().unwrap();
            let should_die = local_should_die.lock().unwrap();
            match next {
                None => break,
                Some(message) => interpreter.process(message),
            }
            if *should_die {
                // Die disgracefull
                break;
            }
        });

        Self(BaseActor { sender, should_die })
    }
}

/// Actor that disgracefully ignores all pending messages before letting itself be
/// killed or killing itself
#[derive(Clone)]
pub struct SuicidalActor<T>(BaseActor<T>);

impl<T> Actor<T> for SuicidalActor<T> {
    fn tell(&self, message: T) -> Result<(), ActorError<T>> {
        self.0.tell(message)
    }

    fn kill(self) -> Result<(), ActorError<T>> {
        self.0.kill()
    }
}

#[allow(dead_code)]
impl<T> SuicidalActor<T>
where
    T: Send + 'static,
{
    pub fn new<I>(mut interpreter: I) -> SuicidalActor<T>
    where
        I: SuicidalInterpreter<T>,
        I: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let should_die = sync::Arc::new(sync::Mutex::new(false));

        let local_should_die = should_die.clone();
        thread::spawn(move || loop {
            let next = receiver.recv().unwrap();
            let should_die = local_should_die.lock().unwrap();
            match next {
                None => break,
                Some(message) => {
                    if interpreter.process(message) {
                        break;
                    }
                }
            }
            if *should_die {
                // Die disgracefull
                break;
            }
        });

        Self(BaseActor { sender, should_die })
    }
}
