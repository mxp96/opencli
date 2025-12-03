use crate::result::{OpenCliError, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::cmp::Ordering;

static VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<constraint>[~^>=<]*)(?P<version>.+)$").unwrap());

static SIMPLE_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^[vVrR]?(?P<major>[0-9]+)(?:\.(?P<minor>[0-9]+))?(?:\.(?P<patch>[0-9]+))?(?P<suffix>.*)$",
    )
    .unwrap()
});

static RANGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<op1>[>=<]+)\s*(?P<ver1>[0-9a-zA-Z\.-]+)(?:\s*,\s*(?P<op2>[>=<]+)\s*(?P<ver2>[0-9a-zA-Z\.-]+))?$").unwrap()
});

#[derive(Debug, Clone, PartialEq)]
pub enum VersionConstraint {
    Exact(Version),
    Caret(Version),
    Tilde(Version),
    GreaterThan(Version),
    GreaterEqual(Version),
    LessThan(Version),
    LessEqual(Version),
    Range(Version, Version),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub suffix: String,
}

impl VersionConstraint {
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        if input == "*" || input == "latest" {
            return Ok(VersionConstraint::GreaterEqual(Version::new(0, 0, 0)));
        }

        if input.contains(',') {
            return Self::parse_range(input);
        }

        if let Some(caps) = VERSION_REGEX.captures(input) {
            let constraint = caps.name("constraint").map_or("", |m| m.as_str());
            let version_str = caps.name("version").unwrap().as_str();

            let version = Version::parse(version_str)?;

            match constraint {
                "^" => Ok(VersionConstraint::Caret(version)),
                "~" => Ok(VersionConstraint::Tilde(version)),
                ">=" => Ok(VersionConstraint::GreaterEqual(version)),
                ">" => Ok(VersionConstraint::GreaterThan(version)),
                "<=" => Ok(VersionConstraint::LessEqual(version)),
                "<" => Ok(VersionConstraint::LessThan(version)),
                "" => Ok(VersionConstraint::Exact(version)),
                _ => Err(OpenCliError::Config(
                    format!("Invalid version constraint: {}", constraint).into(),
                )),
            }
        } else {
            let version = Version::parse(input)?;
            Ok(VersionConstraint::Exact(version))
        }
    }

    fn parse_range(input: &str) -> Result<Self> {
        if let Some(caps) = RANGE_REGEX.captures(input) {
            let op1 = caps.name("op1").unwrap().as_str();
            let ver1 = Version::parse(caps.name("ver1").unwrap().as_str())?;

            if let (Some(op2), Some(ver2_str)) = (caps.name("op2"), caps.name("ver2")) {
                let ver2 = Version::parse(ver2_str.as_str())?;

                match (op1, op2.as_str()) {
                    (">=", "<") | (">", "<") | (">=", "<=") => {
                        Ok(VersionConstraint::Range(ver1, ver2))
                    }
                    _ => Err(OpenCliError::Config("Invalid range constraint".into())),
                }
            } else {
                match op1 {
                    ">=" => Ok(VersionConstraint::GreaterEqual(ver1)),
                    ">" => Ok(VersionConstraint::GreaterThan(ver1)),
                    "<=" => Ok(VersionConstraint::LessEqual(ver1)),
                    "<" => Ok(VersionConstraint::LessThan(ver1)),
                    _ => Err(OpenCliError::Config("Invalid constraint operator".into())),
                }
            }
        } else {
            Err(OpenCliError::Config("Invalid range format".into()))
        }
    }

    pub fn matches(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Exact(v) => {
                version.major == v.major && version.minor == v.minor && version.patch == v.patch
            }
            VersionConstraint::Caret(v) => {
                if v.major == 0 {
                    version.major == 0 && version.minor == v.minor && version.patch >= v.patch
                } else {
                    version.major == v.major
                        && (version.minor > v.minor
                            || (version.minor == v.minor && version.patch >= v.patch))
                }
            }
            VersionConstraint::Tilde(v) => {
                version.major == v.major && version.minor == v.minor && version.patch >= v.patch
            }
            VersionConstraint::GreaterThan(v) => version > v,
            VersionConstraint::GreaterEqual(v) => version >= v,
            VersionConstraint::LessThan(v) => version < v,
            VersionConstraint::LessEqual(v) => version <= v,
            VersionConstraint::Range(min, max) => version >= min && version < max,
        }
    }

    pub fn latest_matching<'a>(&self, versions: &'a [Version]) -> Option<&'a Version> {
        versions.iter().filter(|v| self.matches(v)).max()
    }
}

impl Version {
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        if let Some(caps) = SIMPLE_VERSION_REGEX.captures(input) {
            let major = caps.name("major").unwrap().as_str().parse().map_err(|_| {
                OpenCliError::Config(format!("Invalid major version: {}", input).into())
            })?;
            let minor = caps
                .name("minor")
                .map_or(Ok(0), |m| m.as_str().parse())
                .map_err(|_| {
                    OpenCliError::Config(format!("Invalid minor version: {}", input).into())
                })?;
            let patch = caps
                .name("patch")
                .map_or(Ok(0), |m| m.as_str().parse())
                .map_err(|_| {
                    OpenCliError::Config(format!("Invalid patch version: {}", input).into())
                })?;
            let suffix = caps.name("suffix").map_or("", |m| m.as_str()).to_string();

            Ok(Version {
                major,
                minor,
                patch,
                suffix,
            })
        } else {
            let clean_input = if input.starts_with('v')
                || input.starts_with('V')
                || input.starts_with('r')
                || input.starts_with('R')
            {
                &input[1..]
            } else {
                input
            };

            let parts: Vec<&str> = clean_input.split('.').collect();
            if parts.is_empty() {
                return Err(OpenCliError::Config(
                    format!("Invalid version format: {}", input).into(),
                ));
            }

            let major_part = parts[0];
            let (major_str, suffix) =
                if let Some(pos) = major_part.find(|c: char| !c.is_ascii_digit()) {
                    (&major_part[..pos], major_part[pos..].to_string())
                } else {
                    (major_part, String::new())
                };

            let major = major_str.parse().map_err(|_| {
                OpenCliError::Config(format!("Invalid version format: {}", input).into())
            })?;

            let minor = if parts.len() > 1 {
                let minor_part = parts[1];
                let minor_str = minor_part
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>();
                minor_str.parse().unwrap_or(0)
            } else {
                0
            };

            let patch = if parts.len() > 2 {
                let patch_part = parts[2];
                let patch_str = patch_part
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>();
                patch_str.parse().unwrap_or(0)
            } else {
                0
            };

            let final_suffix = if parts.len() > 1 && suffix.is_empty() {
                let remaining = parts[1..].join(".");
                remaining
                    .chars()
                    .skip_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect()
            } else {
                suffix
            };

            Ok(Version {
                major,
                minor,
                patch,
                suffix: final_suffix,
            })
        }
    }

    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            suffix: String::new(),
        }
    }

    pub fn with_suffix(major: u32, minor: u32, patch: u32, suffix: String) -> Self {
        Self {
            major,
            minor,
            patch,
            suffix,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.suffix.is_empty() {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            write!(
                f,
                "{}.{}.{}{}",
                self.major, self.minor, self.patch, self.suffix
            )
        }
    }
}
