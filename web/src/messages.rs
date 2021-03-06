pub enum Message {
    Buffer(BufferMessage),
    Protocol(ProtocolMessage),
}

pub enum BufferMessage {
    Input(usize, String),
    Ignore,
}

impl From<BufferMessage> for Message {
    fn from(msg: BufferMessage) -> Self {
        Message::Buffer(msg)
    }
}

pub enum ProtocolMessage {
    Selected(usize, usize, String),
    Input(usize, usize, String),
    Ignore,
}

impl From<ProtocolMessage> for Message {
    fn from(msg: ProtocolMessage) -> Self {
        Message::Protocol(msg)
    }
}
