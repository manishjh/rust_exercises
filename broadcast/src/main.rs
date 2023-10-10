use std::{
    sync::{
        mpsc::{self, channel},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

//we create a broadcast channel.
//assume we have a single sender, and multiple receivers.
//to achieve it we create a vector of mpsc channels, each catering to one receiver.
// we can clone this channel accross threads, so this needs to be wrapped in Arc.
pub struct InnerBroadcastChannel<T> {
    senders: Vec<mpsc::Sender<T>>,
}

pub struct BroadcastChannel<T> {
    channel: Arc<Mutex<InnerBroadcastChannel<T>>>,
}

//when a sends sends a message, it is sent to each mpsc in the vector thus to every receiver.
//T must be clone for us to send arbitrary types. to be used independently in recv threads
impl<T: Clone> BroadcastChannel<T> {
    pub fn new() -> Self {
        Self {
            channel: Arc::new(Mutex::new(InnerBroadcastChannel { senders: vec![] })),
        }
    }

    pub fn broadcast(&self, msg: T) {
        self.channel.lock().unwrap().broadcast(msg);
    }

    pub fn create_receiver(&self) -> mpsc::Receiver<T> {
        self.channel.lock().unwrap().create_receiver()
    }
}

impl<T: Clone> InnerBroadcastChannel<T> {
    pub fn new() -> Self {
        Self { senders: vec![] }
    }

    pub fn broadcast(&self, msg: T) {
        for sender in self.senders.iter() {
            sender.send(msg.clone()).unwrap();
        }
    }

    pub fn create_receiver(&mut self) -> mpsc::Receiver<T> {
        let (sender, receiver) = channel::<T>();
        self.senders.push(sender);
        receiver
    }
}

// we later expand it to multiple senders.
// what changes is, that broadcastChannel is cloned, and the rest should stay the same.
// the vector of mpsc channels is not thread safe.
fn main() {
    let bc = Arc::new(BroadcastChannel::<String>::new());
    let mut producers : Vec<JoinHandle<()>> = Vec::new();
    let mut consumers : Vec<JoinHandle<()>>= Vec::new();

    for i in 1..5 {
        let bc2 = bc.clone();
        let c1 = thread::spawn(move || {
            let rec = bc2.create_receiver();
            loop {
                match rec.recv_timeout(Duration::from_secs(5)) {
                    Ok(msg) => println!("receiver {} got ---> {}", &i, msg),
                    Err(_) => break
                }
            }
        });
        consumers.push(c1);
    }

    for i in 1..4 {
        let bc1 = bc.clone();
        let p1 = thread::spawn(move || {
            for j in 1..10 {
                thread::sleep(Duration::from_secs(1));
                bc1.broadcast(String::from(format!("producer{}, message {}", &i, &j)));          
            }
        });
        producers.push(p1);
    }
   

   for p in producers.into_iter(){
        p.join().unwrap();
   }

   for c in consumers.into_iter() {
        c.join().unwrap();
   }

    println!("All done!");
}
