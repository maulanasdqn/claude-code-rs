use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc, oneshot};

pub struct QuestionRequest {
    pub question: String,
    pub responder: oneshot::Sender<Option<String>>,
}

#[derive(Clone)]
pub struct QuestionBridge {
    sender: mpsc::UnboundedSender<QuestionRequest>,
}

impl QuestionBridge {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<QuestionRequest>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { sender: tx }, rx)
    }

    pub async fn ask(&self, question: String) -> Option<String> {
        let (tx, rx) = oneshot::channel();
        let req = QuestionRequest { question, responder: tx };
        if self.sender.send(req).is_err() {
            return None;
        }
        rx.await.ok().flatten()
    }
}

#[derive(Default)]
pub struct OptionalQuestionBridge(Mutex<Option<QuestionBridge>>);

impl OptionalQuestionBridge {
    pub fn new() -> Self { Self(Mutex::new(None)) }
    pub fn set(&self, b: QuestionBridge) { *self.0.lock().unwrap() = Some(b); }
    pub fn get(&self) -> Option<QuestionBridge> { self.0.lock().unwrap().clone() }
}

pub type SharedQuestionBridge = Arc<OptionalQuestionBridge>;
