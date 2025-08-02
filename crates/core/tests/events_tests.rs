use rusty_runways_core::events::{Event, ScheduledEvent};
use std::collections::BinaryHeap;

#[test]
fn scheduled_events_ordered_by_time() {
    let mut heap = BinaryHeap::new();
    heap.push(ScheduledEvent {
        time: 10,
        event: Event::Restock,
    });
    heap.push(ScheduledEvent {
        time: 5,
        event: Event::Restock,
    });
    assert_eq!(heap.pop().unwrap().time, 5);
}

#[test]
fn scheduled_events_equality_ignores_payload() {
    let a = ScheduledEvent {
        time: 3,
        event: Event::Restock,
    };
    let b = ScheduledEvent {
        time: 3,
        event: Event::LoadingEvent { plane: 1 },
    };
    assert_eq!(a, b);
}
