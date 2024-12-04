use anyhow::Result;
use spacejam_safrole::{Error, Markers, State};

#[test]
fn enact_epoch_change_with_no_tickets_1() -> Result<()> {
    let mut state = State::default();
    let output = state.enact(1, Default::default(), Default::default())?;
    assert_eq!(output, Ok(Markers::default()));
    Ok(())
}

#[test]
fn enact_epoch_change_with_no_tickets_2() -> Result<()> {
    let mut state = State::default();
    state.tau = 1;

    let output = state.enact(1, Default::default(), Default::default())?;
    assert_eq!(output, Err(Error::BadSlot));
    Ok(())
}

#[test]
fn enact_epoch_change_with_no_tickets_3() -> Result<()> {
    let mut state = State::default();
    state.tau = 1;

    let output = state.enact(10, Default::default(), Default::default())?;
    assert_eq!(output, Ok(Markers::default()));
    assert_eq!(state.tau, 10);
    Ok(())
}
