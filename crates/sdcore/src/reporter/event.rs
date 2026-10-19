pub struct ReportEvent<'a> {
    pub kind: ReportEventKind,
    pub subject: &'a str,
    pub source: &'a str,
    pub message: &'a str,
    pub artifact: Option<ReportArtifact<'a>>,
}

impl<'a> ReportEvent<'a> {
    pub fn new(kind: ReportEventKind, subject: &'a str, source: &'a str, message: &'a str) -> Self {
        Self {
            kind,
            subject,
            source,
            message,
            artifact: None,
        }
    }

    pub fn with_artifact(
        kind: ReportEventKind,
        subject: &'a str,
        source: &'a str,
        message: &'a str,
        artifact: ReportArtifact<'a>,
    ) -> Self {
        Self {
            kind,
            subject,
            source,
            message,
            artifact: Some(artifact),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ReportArtifact<'a> {
    pub artifact_type: &'a str,
    pub artifact_name: &'a str,
    pub previous_ref: &'a str,
    pub artifact_ref: &'a str,
    pub metadata_json: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportEventKind {
    Started,
    ArtifactBefore,
    ArtifactAfter,
    Output,
    Succeeded,
    Failed,
    Rejected,
}

impl ReportEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ReportEventKind::Started => "started",
            ReportEventKind::ArtifactBefore => "artifact_before",
            ReportEventKind::ArtifactAfter => "artifact_after",
            ReportEventKind::Output => "output",
            ReportEventKind::Succeeded => "succeeded",
            ReportEventKind::Failed => "failed",
            ReportEventKind::Rejected => "rejected",
        }
    }
}
