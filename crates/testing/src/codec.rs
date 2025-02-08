//! Codec tests

use anyhow::Result;
use codec::JamCodec;
use paste::paste;
use score::{
    block::{header::Header, Block},
    extrinsic::Extrinsic,
    extrinsic::{
        dispute::DisputesExtrinsic, ticket::TicketEnvelope, AvailAssurance, Preimage,
        ReportGuarantee,
    },
    service::{RefineContext, WorkItem, WorkPackage, WorkReport, WorkResult},
};
use specjam::registry::tests::*;

macro_rules! impl_codec_tests {
    ($name:ident) => {{
        let test = paste::paste!([< TEST_CODEC_ $name:upper >]);
        let json = test.input.to_string();
        let data = hex::decode(&test.output)?;

        (json, data)
    }};
    ($name:ident, $dest:ident) => {
        paste! {
            #[test]
            fn [<decode_ $name>]() -> Result<()> {
                let (json, data) = impl_codec_tests!($name);
                let decoded: $dest = $dest::from_json(json)?;

                assert_eq!(decoded.encode()?, data);
                assert_eq!(decoded, $dest::decode(&data)?);
                Ok(())
            }
        }
    };
    (@ex $name:ident, $dest:ident) => {
        paste! {
            #[test]
            fn [<decode_ $name>]() -> Result<()> {
                let (json, data) = impl_codec_tests!($name);
                let decoded: Vec<$dest> = $dest::load_json(json)?;

                assert_eq!(decoded.encode()?, data);
                assert_eq!(decoded, Vec::<$dest>::decode(&data)?);
                Ok(())
            }
        }
    };
    ($(($name:ident, $dest:ident)),* @ex $(($ex_name:ident, $ex_dest:ident)),*) => {
        $(impl_codec_tests!($name, $dest);)*
        $(impl_codec_tests!(@ex $ex_name, $ex_dest);)*
    };
}

impl_codec_tests! {
    (block, Block),
    (disputes_extrinsic, DisputesExtrinsic),
    (extrinsic, Extrinsic),
    (header_0, Header),
    (header_1, Header),
    (refine_context, RefineContext),
    (work_item, WorkItem),
    (work_package, WorkPackage),
    (work_report, WorkReport),
    (work_result_0, WorkResult),
    (work_result_1, WorkResult)

    @ex
    (assurances_extrinsic, AvailAssurance),
    (guarantees_extrinsic, ReportGuarantee),
    (preimages_extrinsic, Preimage),
    (tickets_extrinsic, TicketEnvelope)
}
