//! Host functions

use crate::{Reason, State, Stepped};
use score::{service::ServiceAccount, Gas};

/// Call the host function
pub fn call<X: Default, Memory: parser::Memory>(
    call: u32,
    state: State<Memory>,
    data: X,
) -> Stepped<Memory, X> {
    let mut state = state;
    let mut data = data;
    match call {
        0..5 => self::state(call, &mut state, Default::default(), &mut data),
        5 => self::bless(&mut state, &mut data),
        6 => self::assign(&mut state, &mut data),
        7 => self::designate(&mut state, &mut data),
        8 => self::checkpoint(&mut state, &mut data),
        9 => self::new(&mut state, &mut data),
        10 => self::upgrade(&mut state, &mut data),
        11 => self::transfer(&mut state, &mut data),
        12 => self::eject(&mut state, &mut data),
        13 => self::query(&mut state, &mut data),
        14 => self::solicit(&mut state, &mut data),
        15 => self::forget(&mut state, &mut data),
        16 => self::yield_(&mut state, &mut data),
        17 => self::historical_lookup(&mut state, &mut data),
        18 => self::fetch(&mut state, &mut data),
        19 => self::export(&mut state, &mut data),
        20 => self::machine(&mut state, &mut data),
        21 => self::peek(&mut state, &mut data),
        22 => self::poke(&mut state, &mut data),
        23 => self::zero(&mut state, &mut data),
        24 => self::void(&mut state, &mut data),
        25 => self::invoke(&mut state, &mut data),
        26 => self::expunge(&mut state, &mut data),
        _ => return Stepped::new(Reason::Panic(format!("unknown host call: {call}")), state),
    };

    Stepped::new(Reason::Halt, state)
}

/// General host calls
///
/// parameters: ϱ,ω,µ,s,...
///
/// with the range 0..4
pub fn state<X: Default, Memory: parser::Memory>(
    call: u32,
    state: &mut State<Memory>,
    _account: ServiceAccount,
    data: &mut X,
) {
    match call {
        0 => self::gas(&mut state.registers, state.gas as u64),
        1 => self::lookup(state, data),
        2 => self::read(state, data),
        3 => self::write(state, data),
        4 => self::info(state, data),
        _ => {}
    };
}

/// (ΩG) Get the gas to register
fn gas(registers: &mut [u64; 13], gas: Gas) {
    registers[7] = gas;
}

/// (ΩL) account lookup
fn lookup<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩR) storage lookup
fn read<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩW) storage write
fn write<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩI) fetch info
fn info<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩB) bless
fn bless<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩA) assign
fn assign<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩD) designate
fn designate<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩC) checkpoint
fn checkpoint<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩN) new
#[allow(clippy::new_ret_no_self)]
fn new<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩU) upgrade
fn upgrade<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩT) transfer
fn transfer<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩE) eject
fn eject<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩQ) query
fn query<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩS) solicit
fn solicit<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩF) forget
fn forget<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩY) yield
fn yield_<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩH) historical lookup
fn historical_lookup<X: Default, Memory: parser::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) {
}

/// (ΩP) fetch
fn fetch<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩX) export
fn export<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩM) machine
fn machine<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩP) peek
fn peek<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩP) poke
fn poke<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩZ) zero
fn zero<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩV) void
fn void<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩI) invoke
fn invoke<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩE) expunge
fn expunge<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// Host call results
#[repr(u64)]
pub enum Result {
    /// The return value indicating an item does not exist.
    None = u64::MAX,
    /// Name unknown.
    What = u64::MAX - 1,
    /// The inner PVM memory index provided for reading/writing is not accessible.
    OOB = u64::MAX - 2,
    /// Storage full
    Full = u64::MAX - 3,
    /// Core index unknown
    Core = u64::MAX - 4,
    /// Insufficient funds
    Cash = u64::MAX - 5,
    /// Gas limit too low
    Low = u64::MAX - 6,
    /// The item is already solicited or cannot be forgotten.
    Huh = u64::MAX - 7,
    /// The return value indicating general success.
    Ok = 0,
}
