use book::level3::L3OrderBook;

use crate::protocol::common::MatchingProtocol;
use crate::queue::common::MatchingQueue;

pub trait MatchingEngine {
    type InputQueue: MatchingQueue;
    type OutputQueue: MatchingQueue;

    type Protocol: MatchingProtocol;
    type Orderbook: L3OrderBook;
}
