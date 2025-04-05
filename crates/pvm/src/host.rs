//! Host functions

use crate::{Reason, State, Stepped};

/// The host function type
pub type HostCall<X, Memory> = fn(u32, State<Memory>, X) -> Stepped<Memory, X>;

/// Interface that abstract host functions
pub trait Host {
    /// The memory type of the state
    type Memory: Default + Clone;

    /// Call the host function
    fn call<X: Default>(
        call: u32,
        state: State<Self::Memory>,
        data: X,
    ) -> Stepped<Self::Memory, X> {
        let mut state = state;
        let mut data = data;
        let _ = match call {
            0 => Self::gas(&mut state, &mut data),
            1 => Self::lookup(&mut state, &mut data),
            2 => Self::read(&mut state, &mut data),
            3 => Self::write(&mut state, &mut data),
            4 => Self::info(&mut state, &mut data),
            5 => Self::bless(&mut state, &mut data),
            6 => Self::assign(&mut state, &mut data),
            7 => Self::designate(&mut state, &mut data),
            8 => Self::checkpoint(&mut state, &mut data),
            9 => Self::new(&mut state, &mut data),
            10 => Self::upgrade(&mut state, &mut data),
            11 => Self::transfer(&mut state, &mut data),
            12 => Self::eject(&mut state, &mut data),
            13 => Self::query(&mut state, &mut data),
            14 => Self::solicit(&mut state, &mut data),
            15 => Self::forget(&mut state, &mut data),
            16 => Self::yield_(&mut state, &mut data),
            17 => Self::historical_lookup(&mut state, &mut data),
            18 => Self::fetch(&mut state, &mut data),
            19 => Self::export(&mut state, &mut data),
            20 => Self::machine(&mut state, &mut data),
            21 => Self::peek(&mut state, &mut data),
            22 => Self::poke(&mut state, &mut data),
            23 => Self::zero(&mut state, &mut data),
            24 => Self::void(&mut state, &mut data),
            25 => Self::invoke(&mut state, &mut data),
            26 => Self::expunge(&mut state, &mut data),
            _ => return Stepped::new(Reason::Panic(format!("unknown host call: {call}")), state),
        };

        Stepped::new(Reason::Halt, state)
    }

    /// (ΩG) Get the gas to register
    fn gas<X: Default>(state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        state.registers[7] = state.gas as u64;
        Result::Ok
    }

    /// (ΩL) account lookup
    fn lookup<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩR) storage lookup
    fn read<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩW) storage write
    fn write<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩI) fetch info
    fn info<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩB) bless
    fn bless<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩA) assign
    fn assign<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩD) designate
    fn designate<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩC) checkpoint
    fn checkpoint<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩN) new
    fn new<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩU) upgrade
    fn upgrade<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩT) transfer
    fn transfer<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩE) eject
    fn eject<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩQ) query
    fn query<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩS) solicit
    fn solicit<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩF) forget
    fn forget<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩY) yield
    fn yield_<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩH) historical lookup
    fn historical_lookup<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩP) fetch
    fn fetch<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩX) export
    fn export<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩM) machine
    fn machine<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩP) peek
    fn peek<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩP) poke
    fn poke<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩZ) zero
    fn zero<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩV) void
    fn void<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩI) invoke
    fn invoke<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }

    /// (ΩE) expunge
    fn expunge<X: Default>(_state: &mut State<Self::Memory>, _data: &mut X) -> Result {
        Result::What
    }
}

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
