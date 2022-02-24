use crate::util::{VersionTriple, VersionTripleError};
use serde::{ser::Serializer, Serialize};
use std::fmt::{self, Debug, Display};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VersionNumberError {
    #[error("Failed to parse extra version from {version:?}: {source}")]
    ExtraVersionInvalid {
        version: String,
        source: std::num::ParseIntError,
    },
    #[error("Failed to parse version triple.")]
    VersionTriplerError(#[from] VersionTripleError),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VersionNumber {
    pub triple: VersionTriple,
    pub extra: Option<Vec<u32>>,
}

impl Display for VersionNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.triple)?;
        if let Some(extra) = &self.extra {
            for number in extra {
                write!(f, ".{}", number)?;
            }
        }
        Ok(())
    }
}

impl Serialize for VersionNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl VersionNumber {
    pub fn new_from_triple(triple: VersionTriple) -> Self {
        Self {
            triple,
            extra: None,
        }
    }

    pub fn from_other_and_number(other: &VersionNumber, number: u32) -> Self {
        other.extra.as_ref().map_or_else(
            || VersionNumber::new(other.triple, Some(vec![number])),
            |bundle_version_extra| {
                let extra = {
                    let mut extras = bundle_version_extra.clone();
                    extras.push(number);
                    extras
                };
                VersionNumber::new(other.triple, Some(extra))
            },
        )
    }

    pub const fn new(triple: VersionTriple, extra: Option<Vec<u32>>) -> Self {
        Self { triple, extra }
    }

    pub fn from_str(v: &str) -> Result<Self, VersionNumberError> {
        match v.split(".").count() {
            1 | 2 | 3 => {
                let triple = VersionTriple::from_str(v)?;
                Ok(Self {
                    triple,
                    extra: None,
                })
            }
            _ => {
                let mut s = v.split(".");
                let triple = VersionTriple::from_split(&mut s, v)?;
                let extra = Some(
                    s.map(|s| {
                        s.parse()
                            .map_err(|source| VersionNumberError::ExtraVersionInvalid {
                                version: v.to_owned(),
                                source,
                            })
                    })
                    .collect::<Result<Vec<u32>, _>>()?,
                );
                Ok(Self { triple, extra })
            }
        }
    }
}
