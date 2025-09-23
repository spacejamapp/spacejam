//! SpaceVM common library

use service::{
    api::{self, AccumulateArgs, Accumulated, AuthorizeArgs, RefineArgs},
    service::result::{Executed, Refined},
};
use spacevm::{
    SpaceVM,
    pvm::{AccumulateState, Invocation, Reason, score::safrole::ValidatorData},
};

/// (ΨA): Accumulation invocation
#[unsafe(no_mangle)]
pub extern "C" fn accumulate(args: AccumulateArgs) -> Accumulated {
    let context = AccumulateState {
        accounts: args.context.accounts,
        validators: args.context.validators.map(|v| ValidatorData {
            bandersnatch: v.bandersnatch,
            ed25519: v.ed25519,
            bls: v.bls,
            metadata: v.metadata,
        }),
        authorization: args.context.authorization.clone(),
        privileges: args.context.privileges,
        entropy: args.context.entropy,
    };
    let accumulated = <SpaceVM as Invocation>::accumulate(
        context,
        args.timeslot,
        args.service,
        args.gas,
        args.operands,
    );

    Accumulated {
        context: api::AccumulateState {
            accounts: accumulated.context.accounts,
            validators: accumulated.context.validators.map(|v| api::ValidatorData {
                bandersnatch: v.bandersnatch,
                ed25519: v.ed25519,
                bls: v.bls,
                metadata: v.metadata,
            }),
            authorization: accumulated.context.authorization,
            privileges: accumulated.context.privileges,
            entropy: accumulated.context.entropy,
        },
        transfers: accumulated.transfers,
        hash: accumulated.hash,
        gas: accumulated.gas,
        reason: match accumulated.reason {
            Reason::Halt => api::Reason::Halt,
            Reason::Panic(message) => api::Reason::Panic(message),
            Reason::Fault { page } => api::Reason::Fault { page },
            Reason::HostCall(addr) => api::Reason::HostCall(addr),
            Reason::OOG => api::Reason::OOG,
            Reason::Continue => api::Reason::Continue,
        },
    }
}

/// (ΨR): Refine invocation
#[unsafe(no_mangle)]
pub extern "C" fn refine(args: RefineArgs) -> Refined {
    let mut args = args;
    <SpaceVM as Invocation>::refine(
        args.core,
        args.index,
        &args.package,
        &args.auth_output,
        &args.all_imports,
        args.export_offset,
        &mut args.accounts,
        args.timeslot,
    )
}

/// (ΨI): Is-Authorized invocation
#[unsafe(no_mangle)]
pub extern "C" fn authorize(args: AuthorizeArgs) -> Executed {
    let mut args = args;
    <SpaceVM as Invocation>::is_authorized(
        &args.package,
        args.core_idx,
        &mut args.accounts,
        args.timeslot,
    )
}
