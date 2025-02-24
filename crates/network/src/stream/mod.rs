//! Streams for the network.

/// The stream types in JAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    /// The stream type for the block announcement.
    BlockAnnouncement,

    /// The stream type for the block request.
    BlockRequest,

    /// The stream type for the state request.
    StateRequest,

    /// The stream type for Safrole ticket distribution (first step).
    SafroleTicketDistribution131,

    /// The stream type for Safrole ticket distribution (second step).
    SafroleTicketDistribution132,

    /// The stream type for work-package submission.
    WorkPackageSubmission,

    /// The stream type for work-package sharing.
    WorkPackageSharing,

    /// The stream type for work-report request.
    WorkReportRequest,

    /// The stream type for shard distribution.
    ShardDistribution,

    /// The stream type for audit shard request.
    AuditShardRequest,

    /// The stream type for segment shard request (no justification).
    SegmentShardRequest139,

    /// The stream type for segment shard request (with justification).
    SegmentShardRequest140,

    /// The stream type for assurance distribution.
    AssuranceDistribution,

    /// The stream type for preimage announcement.
    PreimageAnnouncement,

    /// The stream type for audit announcement.
    AuditAnnouncement,

    /// The stream type for judgment publication.
    JudgmentPublication,

    /// Unknown stream type.
    Unknown(u8),
}

impl From<u8> for StreamType {
    fn from(value: u8) -> Self {
        match value {
            0 => StreamType::BlockAnnouncement,
            128 => StreamType::BlockRequest,
            129 => StreamType::StateRequest,
            131 => StreamType::SafroleTicketDistribution131,
            132 => StreamType::SafroleTicketDistribution132,
            133 => StreamType::WorkPackageSubmission,
            134 => StreamType::WorkPackageSharing,
            136 => StreamType::WorkReportRequest,
            137 => StreamType::ShardDistribution,
            138 => StreamType::AuditShardRequest,
            139 => StreamType::SegmentShardRequest139,
            140 => StreamType::SegmentShardRequest140,
            141 => StreamType::AssuranceDistribution,
            142 => StreamType::PreimageAnnouncement,
            144 => StreamType::AuditAnnouncement,
            145 => StreamType::JudgmentPublication,
            _ => StreamType::Unknown(value),
        }
    }
}
