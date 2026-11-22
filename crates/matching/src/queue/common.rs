pub trait MatchingMessage {}

pub trait MatchingQueue {
    type Message: MatchingMessage;
}
