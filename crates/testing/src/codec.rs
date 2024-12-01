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

#[test]
fn decode_assurances_extrinsic() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/assurances_extrinsic.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/assurances_extrinsic.bin");

    let _assurances: Vec<AvailAssurance> = serde_json::from_str::<Vec<AvailAssuranceJson>>(json)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>>>()?;

    // TODO: implement the jamcodec
    // assert_eq!(assurances.encode(), data);
    Ok(())
}

#[test]
fn decode_block() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/block.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/block.bin");

    let _block: Block = serde_json::from_str::<BlockJson>(json)?.try_into()?;

    // TODO: implement the jamcodec
    // assert_eq!(block.encode(), data);
    Ok(())
}

#[test]
fn decode_disputes_extrinsic() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/disputes_extrinsic.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/disputes_extrinsic.bin");

    let _disputes: DisputesExtrinsic =
        serde_json::from_str::<DisputesExtrinsicJson>(json)?.try_into()?;
    Ok(())
}

#[test]
fn decode_extrinsic() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/extrinsic.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/extrinsic.bin");

    let _extrinsic: Extrinsic = serde_json::from_str::<ExtrinsicJson>(json)?.try_into()?;
    Ok(())
}

#[test]
fn decode_guarantees_extrinsic() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/guarantees_extrinsic.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/guarantees_extrinsic.bin");

    let _guarantees: GuaranteesExtrinsic = serde_json::from_str::<Vec<ReportGuaranteeJson>>(json)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

#[test]
fn decode_header_0() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/header_0.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/header_0.bin");

    let _header: Header = serde_json::from_str::<HeaderJson>(json)?.try_into()?;
    Ok(())
}

#[test]
fn decode_header_1() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/header_1.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/header_1.bin");

    let _header: Header = serde_json::from_str::<HeaderJson>(json)?.try_into()?;
    Ok(())
}

#[test]
fn decode_preimages_extrinsic() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/preimages_extrinsic.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/preimages_extrinsic.bin");

    let _preimages: PreimagesExtrinsic = serde_json::from_str::<Vec<PreimageJson>>(json)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}

#[test]
fn decode_refine_context() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/refine_context.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/refine_context.bin");

    let _refine_context: RefineContext =
        serde_json::from_str::<RefineContextJson>(json)?.try_into()?;

    Ok(())
}

#[test]
fn decode_tickets_extrinsic() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/tickets_extrinsic.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/tickets_extrinsic.bin");

    let _tickets: TicketsExtrinsic = serde_json::from_str::<Vec<TicketEnvelopeJson>>(json)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}

#[test]
fn decode_work_item() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/work_item.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/work_item.bin");

    let _work_item: WorkItem = serde_json::from_str::<WorkItemJson>(json)?.try_into()?;

    Ok(())
}

#[test]
fn decode_work_package() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/work_package.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/work_package.bin");

    let _work_package: WorkPackage = serde_json::from_str::<WorkPackageJson>(json)?.try_into()?;

    Ok(())
}

#[test]
fn decode_work_report() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/work_report.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/work_report.bin");

    let _work_report: WorkReport = serde_json::from_str::<WorkReportJson>(json)?.try_into()?;
    Ok(())
}

#[test]
fn decode_work_result_0() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/work_result_0.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/work_result_0.bin");

    let _work_result: WorkResult = serde_json::from_str::<WorkResultJson>(json)?.try_into()?;
    Ok(())
}

#[test]
fn decode_work_result_1() -> Result<()> {
    let json = include_str!("../jamtestvectors/codec/data/work_result_1.json");
    let _data = include_bytes!("../jamtestvectors/codec/data/work_result_1.bin");

    let _work_result: WorkResult = serde_json::from_str::<WorkResultJson>(json)?.try_into()?;
    Ok(())
}
