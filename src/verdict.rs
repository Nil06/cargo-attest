//! Verdict types — the actionable output of an attestation run.
//!
//! Design choice: a verdict is more than a bool. We distinguish:
//! - `Trusted`: every check we ran passed and at least one was load-bearing.
//! - `Unverified`: nothing was *wrong*, but we lacked evidence to claim trust.
//! - `Mismatch`: a check actively failed (hash diverged, signature invalid).
//! - `Error`: the run itself could not complete (network, malformed input).
//!
//! Consumers (CI scripts, humans) should treat `Unverified` and `Mismatch`
//! as distinct: the first is "missing proof", the second is "broken proof".

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Verdict {
    Trusted {
        subject: Subject,
        checks: Vec<Check>,
    },
    Unverified {
        subject: Subject,
        reason: String,
        checks: Vec<Check>,
    },
    Mismatch {
        subject: Subject,
        reason: String,
        checks: Vec<Check>,
    },
    Error {
        subject: Subject,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Subject {
    GithubRelease {
        repo: String,
        tag: String,
        asset: Option<String>,
    },
    Crate {
        name: String,
        version: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Check {
    /// Short identifier, e.g. "sha256-body-checksum", "slsa-provenance".
    pub name: String,
    pub outcome: CheckOutcome,
    /// Free-form human-readable detail.
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckOutcome {
    Pass,
    Fail,
    Skip,
}

impl Verdict {
    /// Exit code convention: 0 = trusted, 1 = unverified, 2 = mismatch, 3 = error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Verdict::Trusted { .. } => 0,
            Verdict::Unverified { .. } => 1,
            Verdict::Mismatch { .. } => 2,
            Verdict::Error { .. } => 3,
        }
    }

    /// One-line human summary (terminals).
    pub fn summary(&self) -> String {
        match self {
            Verdict::Trusted { .. } => "TRUSTED".into(),
            Verdict::Unverified { reason, .. } => format!("UNVERIFIED: {reason}"),
            Verdict::Mismatch { reason, .. } => format!("MISMATCH: {reason}"),
            Verdict::Error { message, .. } => format!("ERROR: {message}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_distinct() {
        let subj = Subject::Crate {
            name: "x".into(),
            version: "0".into(),
        };
        let codes = [
            Verdict::Trusted {
                subject: subj.clone(),
                checks: vec![],
            }
            .exit_code(),
            Verdict::Unverified {
                subject: subj.clone(),
                reason: "".into(),
                checks: vec![],
            }
            .exit_code(),
            Verdict::Mismatch {
                subject: subj.clone(),
                reason: "".into(),
                checks: vec![],
            }
            .exit_code(),
            Verdict::Error {
                subject: subj,
                message: "".into(),
            }
            .exit_code(),
        ];
        // All four should be distinct.
        for (i, a) in codes.iter().enumerate() {
            for b in &codes[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn json_roundtrip() {
        let v = Verdict::Trusted {
            subject: Subject::GithubRelease {
                repo: "foo/bar".into(),
                tag: "1.0".into(),
                asset: None,
            },
            checks: vec![Check {
                name: "sha256".into(),
                outcome: CheckOutcome::Pass,
                detail: "match".into(),
            }],
        };
        let s = serde_json::to_string(&v).unwrap();
        let v2: Verdict = serde_json::from_str(&s).unwrap();
        assert_eq!(v, v2);
    }
}
