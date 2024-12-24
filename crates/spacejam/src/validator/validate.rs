use score::{
    block::Block,
    extrinsic::Extrinsic,
    state::State,
    validator::{Context, Result, ValidateExtrinsic},
};
use std::{marker::PhantomData, sync::Arc};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};

/// Block validation service
pub struct Validation<Validator: ValidateExtrinsic> {
    /// The safrole of the validation
    pub safrole: Arc<State>,
    /// The sender of the validation
    pub sender: Sender<()>,
    /// The receiver of the validation
    pub receiver: Receiver<Block>,
    /// The validator of the validation
    pub _validator: PhantomData<Validator>,
}

impl<Validator: ValidateExtrinsic> Validation<Validator> {
    /// Creates a new block validation service
    pub fn new(safrole: Arc<State>, sender: Sender<()>, receiver: Receiver<Block>) -> Self {
        Self {
            safrole,
            sender,
            receiver,
            _validator: Default::default(),
        }
    }

    /// Spawn the validation service
    ///
    /// Receives blocks from the network and validates them
    pub async fn spawn(&mut self) -> Result<()> {
        todo!("spawn the validation service")
    }

    /// Validate the block
    pub async fn validate(&self, safrole: Arc<State>, block: Block) -> Result<Block> {
        let mut queue = JoinSet::<Result<()>>::new();
        let header = Arc::new(block.header);
        let context = Context {
            safrole,
            header: header.clone(),
        };

        let actx = context.clone();
        let dctx = context.clone();
        let pctx = context.clone();
        let gctx = context.clone();

        let assurances = Arc::new(block.extrinsic.assurances);
        let disputes = Arc::new(block.extrinsic.disputes);
        let preimages = Arc::new(block.extrinsic.preimages);
        let guarantees = Arc::new(block.extrinsic.guarantees);
        let tickets = Arc::new(block.extrinsic.tickets);

        let cloned_assurances = assurances.clone();
        let cloned_disputes = disputes.clone();
        let cloned_preimages = preimages.clone();
        let cloned_guarantees = guarantees.clone();
        let cloned_tickets = tickets.clone();

        queue.spawn(async move { Validator::validate_assurances(actx, cloned_assurances).await });
        queue.spawn(async move { Validator::validate_disputes(dctx, cloned_disputes).await });
        queue.spawn(async move { Validator::validate_preimages(pctx, cloned_preimages).await });
        queue.spawn(async move { Validator::validate_guarantees(gctx, cloned_guarantees).await });
        queue.spawn(async move { Validator::validate_tickets(context, cloned_tickets).await });

        let _ = queue
            .join_all()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        // TODO: error handling
        let header = Arc::try_unwrap(header).expect("header should be unwrapped");
        let assurances = Arc::try_unwrap(assurances).expect("assurances should be unwrapped");
        let disputes = Arc::try_unwrap(disputes).expect("disputes should be unwrapped");
        let preimages = Arc::try_unwrap(preimages).expect("preimages should be unwrapped");
        let guarantees = Arc::try_unwrap(guarantees).expect("guarantees should be unwrapped");
        let tickets = Arc::try_unwrap(tickets).expect("tickets should be unwrapped");

        Ok(Block {
            header,
            extrinsic: Extrinsic {
                assurances,
                disputes,
                preimages,
                guarantees,
                tickets,
            },
        })
    }
}
