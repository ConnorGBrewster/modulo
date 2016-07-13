use ::core::core_msg::ToCoreThreadMsg;
use ::file::file_msg::{FileId, ToFileThreadMsg};
use ::file::file_thread::FileThread;
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};

pub struct Core {
    file_threads: HashMap<FileId, Sender<ToFileThreadMsg>>,
    file_receiver: Receiver<ToCoreThreadMsg>,
    file_sender: Sender<ToCoreThreadMsg>,
    file_id_counter: usize,
}

impl Core {
    pub fn start() -> (Sender<ToCoreThreadMsg>, JoinHandle<()>) {
        let (sender, receiver) = mpsc::channel();

        let file_sender = sender.clone();
        let handle = thread::spawn(move || {
            let mut core = Core {
                file_threads: HashMap::new(),
                file_receiver: receiver,
                file_sender: file_sender,
                file_id_counter: 0,
            };
            core.run();
        });

        (sender, handle)
    }

    /// Runs the event loop for the `CoreThread`
    pub fn run(&mut self) {
        while let Ok(msg) = self.file_receiver.recv() {
            match msg {
                ToCoreThreadMsg::FileThreadMsg(id, msg) => {
                    let file_thread = match self.file_threads.get(&id) {
                        Some(file_thread) => file_thread,
                        None => return warn!("Received message for closed file thread: {:?}", id),
                    };

                    let _ = file_thread.send(msg);
                },
                ToCoreThreadMsg::SpawnFileThread => self.spawn_file_thread(),
            }
        }
    }

    pub fn spawn_file_thread(&mut self) {
        let (sender, receiver) = mpsc::channel();
        let id = FileId(self.file_id_counter);
        let file_thread = FileThread::start(id,
                                            self.file_sender.clone(),
                                            receiver);
        self.file_threads.insert(id, sender);
        self.file_id_counter += 1;
    }
}
