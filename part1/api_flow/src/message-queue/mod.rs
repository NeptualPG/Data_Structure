//! The `message-queue` module simulates a queue such as Kafka, SQS or Redis
//! Streams.
//!
//! It uses `std::sync::mpsc`, the multi-producer / single-consumer channel from
//! the standard library. That is enough to show the two ideas that matter:
//! the API thread *sends* a task and moves on, while a background worker thread
//! *receives* tasks and does the slow work (sending an email) later.

use std::sync::mpsc::{self, Receiver, Sender, SendError};
use std::thread::{self, JoinHandle};

/// A unit of background work.
///
/// This is the "message" that travels through the queue. The queue is typed
/// (`mpsc::channel::<Task>()`), so a worker can only ever receive a `Task`.
#[derive(Debug)]
pub enum Task {
    /// Ask the background worker to send a welcome email to a new user.
    SendWelcomeEmail {
        to_name: String,
        to_email: String,
    },
}

/// The handle the application uses to publish tasks.
///
/// Only the *sending* half of the channel lives here; the receiving half was
/// moved into the worker thread.
pub struct MessageQueue {
    sender: Sender<Task>,
}

impl MessageQueue {
    /// Create the queue and start the worker thread.
    ///
    /// Returns the handle the application will use plus the `JoinHandle` of
    /// the worker. The caller keeps the `JoinHandle` so it can wait for the
    /// worker to finish before the program exits.
    pub fn start() -> (MessageQueue, JoinHandle<()>) {
        // `mpsc::channel` returns `(Sender<T>, Receiver<T>)`: two ends of one
        // pipe. Anything sent through the sender arrives at the receiver.
        let (sender, receiver): (Sender<Task>, Receiver<Task>) = mpsc::channel();

        // `thread::spawn` starts a real OS thread and runs the closure on it.
        // `move` takes ownership of `receiver`, which is what lets the closure
        // be `'static` (it may outlive this function).
        let worker = thread::spawn(move || {
            println!("      [queue]  background worker thread started");

            // `recv` blocks until a message arrives. It returns:
            //   Ok(task) -> a message was received
            //   Err(_)   -> every sender was dropped, so the channel is closed
            // `while let` is the loop form of `if let`: keep looping while we
            // are still getting messages. This is the graceful shutdown path.
            while let Ok(task) = receiver.recv() {
                handle_task(&task);
            }

            println!("      [queue]  channel closed, worker thread stopping");
        });

        (MessageQueue { sender }, worker)
    }

    /// Publish a task.
    ///
    /// Returns `Err` if the worker is gone (the receiving end was dropped), so
    /// a broken queue is reported instead of panicking.
    pub fn send(&self, task: Task) -> Result<(), SendError<Task>> {
        self.sender.send(task)
    }
}

/// The worker's job. In a real system this would talk to an email provider.
fn handle_task(task: &Task) {
    match task {
        Task::SendWelcomeEmail { to_name, to_email } => {
            // Simulated work: pretend to wait for a network call.
            std::thread::sleep(std::time::Duration::from_millis(150));
            println!("      [worker] welcome email sent to {to_name} <{to_email}>");
        }
    }
}
