//! Codec tests

use anyhow::{Context, Result};
use codec::Codec;
use paste::paste;
use score::{
    block::{Block, header::Header},
    extrinsic::Extrinsic,
    extrinsic::{
        AvailAssurance, Preimage, ReportGuarantee, dispute::DisputesExtrinsic,
        ticket::TicketEnvelope,
    },
    service::{RefineContext, WorkDigest, WorkItem, WorkPackage, WorkReport},
};
use specjam::Registry;
use std::path::PathBuf;

macro_rules! impl_codec_tests {
    ($name:ident) => {{
        let scale = if cfg!(feature = "full") {
            specjam::Scale::Full
        } else {
            specjam::Scale::Tiny
        };
        let registry = Registry::with_scale(PathBuf::from("../../res/jam-test-vectors"), scale);
        let test = registry.entry("codec").unwrap().test(stringify!($name)).unwrap();
        let json = test.input.expect_json()?.to_string();
        let data = test.output.expect_bin()?.to_vec();

        (json, data)
    }};
    ($name:ident, $dest:ident) => {
        paste! {
            #[test]
            fn [<decode_ $name>]() -> Result<()> {
                let (json, data) = impl_codec_tests!($name);
                let decoded: $dest = $dest::from_json(json).context("failed to decode json")?;

                println!("decoded: {:?}", decoded);
                assert_eq!(decoded.encode(), data, "encoded data mismatch");
                println!("encoded: {:?}", &(decoded.encode())[68..]);
                assert_eq!(decoded, $dest::decode(&data)?, "decoded data mismatch");
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

                assert_eq!(decoded.encode(), data, "encoded data mismatch");
                assert_eq!(decoded, Vec::<$dest>::decode(&data)?, "decoded data mismatch");
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
    (work_result_0, WorkDigest),
    (work_result_1, WorkDigest)

    @ex
    (assurances_extrinsic, AvailAssurance),
    (guarantees_extrinsic, ReportGuarantee),
    (preimages_extrinsic, Preimage),
    (tickets_extrinsic, TicketEnvelope)
}
