//! Codec tests

use anyhow::Result;
use core::{
    block::{
        header::{Header, HeaderJson},
        Block, BlockJson, Extrinsic, ExtrinsicJson,
    },
    dispute::{DisputesExtrinsic, DisputesExtrinsicJson},
    misc::{
        AvailAssurance, AvailAssuranceJson, GuaranteesExtrinsic, PreimageJson, PreimagesExtrinsic,
        RefineContext, RefineContextJson, ReportGuaranteeJson,
    },
    ticket::{TicketEnvelopeJson, TicketsExtrinsic},
    work::{
        report::{WorkReport, WorkReportJson, WorkResult, WorkResultJson},
        WorkItem, WorkItemJson, WorkPackage, WorkPackageJson,
    },
};
use paste::paste;

macro_rules! load_codec_data {
    ($name:ident) => {{
        let json = include_str!(concat!(
            "../jamtestvectors/codec/data/",
            stringify!($name),
            ".json"
        ));
        let data = include_bytes!(concat!(
            "../jamtestvectors/codec/data/",
            stringify!($name),
            ".bin"
        ));
        (json, data)
    }};
    ($name:ident, $json:ident, $dest:ident) => {
        paste! {
            #[test]
            fn [<decode_ $name>]() -> Result<()> {
                let (json, _data) = load_codec_data!($name);
                let _decoded: $dest = serde_json::from_str::<$json>(json)?.try_into()?;
                Ok(())
            }
        }
    };
    ($name:ident, $json:ty, $dest:ty) => {
        paste! {
            #[test]
            fn [<decode_ $name>]() -> Result<()> {
                let (json, _data) = load_codec_data!($name);
                let _decoded: $dest = serde_json::from_str::<$json>(json)?
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?;
                Ok(())
            }
        }
    };
    ($(($name:ident, $json:ident, $dest:ident)),*) => {
        $(load_codec_data!($name, $json, $dest);)*
    };
    ($(($name:ident, $json:ty, $dest:ty)),*) => {
        $(load_codec_data!($name, $json, $dest);)*
    };
}

load_codec_data! {
    (assurances_extrinsic, Vec<AvailAssuranceJson>, Vec<AvailAssurance>),
    (guarantees_extrinsic, Vec<ReportGuaranteeJson>, GuaranteesExtrinsic),
    (preimages_extrinsic, Vec<PreimageJson>, PreimagesExtrinsic),
    (tickets_extrinsic, Vec<TicketEnvelopeJson>, TicketsExtrinsic)
}

load_codec_data! {
    (block, BlockJson, Block),
    (disputes_extrinsic, DisputesExtrinsicJson, DisputesExtrinsic),
    (extrinsic, ExtrinsicJson, Extrinsic),
    (header_0, HeaderJson, Header),
    (header_1, HeaderJson, Header),
    (refine_context, RefineContextJson, RefineContext),
    (work_item, WorkItemJson, WorkItem),
    (work_package, WorkPackageJson, WorkPackage),
    (work_report, WorkReportJson, WorkReport),
    (work_result_0, WorkResultJson, WorkResult),
    (work_result_1, WorkResultJson, WorkResult)
}
