use std::{error::Error, fmt};

use crate::parser::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    MissingAttribute {
        command: String,
        key: String,
        span: Option<SourceSpan>,
    },
    UnknownCommand {
        command: String,
        span: Option<SourceSpan>,
        suggestion: Option<String>,
    },
    InvalidScriptVersion {
        version: String,
        span: Option<SourceSpan>,
    },
    UnsupportedScriptVersion {
        version: u32,
        supported_version: u32,
        span: Option<SourceSpan>,
    },
}

impl CompileError {
    pub(super) fn with_span(self, span: Option<SourceSpan>) -> Self {
        match self {
            Self::MissingAttribute {
                command,
                key,
                span: existing,
            } => Self::MissingAttribute {
                command,
                key,
                span: existing.or(span),
            },
            Self::UnknownCommand {
                command,
                span: existing,
                suggestion,
            } => Self::UnknownCommand {
                command,
                span: existing.or(span),
                suggestion,
            },
            Self::InvalidScriptVersion {
                version,
                span: existing,
            } => Self::InvalidScriptVersion {
                version,
                span: existing.or(span),
            },
            Self::UnsupportedScriptVersion {
                version,
                supported_version,
                span: existing,
            } => Self::UnsupportedScriptVersion {
                version,
                supported_version,
                span: existing.or(span),
            },
        }
    }

    pub fn span(&self) -> Option<SourceSpan> {
        match self {
            Self::MissingAttribute { span, .. }
            | Self::UnknownCommand { span, .. }
            | Self::InvalidScriptVersion { span, .. }
            | Self::UnsupportedScriptVersion { span, .. } => *span,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = self.span() {
            write!(formatter, "line {}, column {}: ", span.line, span.column)?;
        }

        match self {
            Self::MissingAttribute { command, key, .. } => {
                write!(
                    formatter,
                    "missing required attribute `{key}` for @{command}"
                )
            }
            Self::UnknownCommand {
                command,
                suggestion,
                ..
            } => {
                write!(formatter, "unknown command @{command}")?;
                if let Some(suggestion) = suggestion {
                    write!(formatter, "; did you mean @{suggestion}?")?;
                }
                Ok(())
            }
            Self::InvalidScriptVersion { version, .. } => {
                write!(formatter, "invalid script format version `{version}`")
            }
            Self::UnsupportedScriptVersion {
                version,
                supported_version,
                ..
            } => {
                write!(
                    formatter,
                    "unsupported script format version {version}; supported version is {supported_version}"
                )
            }
        }
    }
}

impl Error for CompileError {}

pub(super) fn span_for(spans: &[SourceSpan], index: usize) -> Option<SourceSpan> {
    spans.get(index).copied()
}
