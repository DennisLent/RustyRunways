use RustyRunways::utils::errors::GameError;

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
