use book::level3::L3OrderBook;

use crate::message::common::MatchingMessage;
use crate::protocol::common::MatchingProtocol;
use crate::queue::common::MatchingQueue;

pub trait MatchingEngine {
    type Message: MatchingMessage;
    type Queue: MatchingQueue;

    type Protocol: MatchingProtocol;
    type Orderbook: L3OrderBook;
}
