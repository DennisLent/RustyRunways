use rusty_runways_core::utils::{
    airplanes::models::AirplaneStatus, coordinate::Coordinate, errors::GameError,
};

#[test]
fn unknown_model_suggests_closest_name() {
    let err = GameError::UnknownModel {
        input: "SparowLight".to_string(),
        suggestion: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("SparrowLight"));
}

#[test]
fn unknown_model_without_suggestion() {
    let err = GameError::UnknownModel {
        input: "X".to_string(),
        suggestion: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("`X` doesn't exist."));
}

#[test]
fn insufficient_funds_display() {
    let err = GameError::InsufficientFunds {
        have: 100.0,
        need: 150.0,
    };
    let msg = format!("{}", err);
    assert_eq!(
        msg,
        "Insufficient funds. Need: $150.00. Currently have: $100.00"
    );
}

#[test]
fn other_error_displays() {
    let cases = vec![
        (
            GameError::OutOfRange {
                distance: 1.0,
                range: 0.5,
            },
            "outside of the airplane range",
        ),
        (
            GameError::RunwayTooShort {
                required: 10.0,
                available: 5.0,
            },
            "requires at least",
        ),
        (
            GameError::MaxPayloadReached {
                current_capacity: 1.0,
                maximum_capacity: 2.0,
                added_weight: 3.0,
            },
            "Cannot load order",
        ),
        (GameError::OrderIdInvalid { id: 1 }, "Order with id 1"),
        (GameError::PlaneIdInvalid { id: 2 }, "Plan with id 2"),
        (GameError::AirportIdInvalid { id: 3 }, "Airport with id 3"),
        (
            GameError::AirportLocationInvalid {
                location: Coordinate { x: 1.0, y: 2.0 },
            },
            "No airport found",
        ),
        (
            GameError::PlaneNotAtAirport { plane_id: 4 },
            "Plane 4 is not located",
        ),
        (
            GameError::PlaneNotReady {
                plane_state: AirplaneStatus::Parked,
            },
            "Airplane not ready",
        ),
        (
            GameError::InsufficientFuel {
                have: 1.0,
                need: 2.0,
            },
            "Insufficient fuel",
        ),
        (GameError::NoCargo, "No cargo to unload"),
        (GameError::SameAirport, "Cannot fly to the airport"),
        (
            GameError::InvalidCommand {
                msg: "oops".to_string(),
            },
            "oops",
        ),
    ];

    for (err, expected) in cases {
        let msg = format!("{}", err);
        assert!(msg.contains(expected));
    }
}
