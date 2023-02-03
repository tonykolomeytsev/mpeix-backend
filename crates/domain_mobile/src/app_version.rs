use std::{cmp::Ordering, collections::VecDeque, fmt::Display, num::ParseIntError, str::FromStr};

#[derive(Debug, Clone)]
pub struct AppVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Display for AppVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
pub enum Error {
    PartParsingError(ParseIntError),
    WrongPartsNumber,
}

impl FromStr for AppVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s
            .split('.')
            .map(|it| it.parse::<u16>())
            .collect::<Result<VecDeque<u16>, _>>()
            .map_err(Error::PartParsingError)?;

        if parts.len() != 3 {
            return Err(Error::WrongPartsNumber);
        }

        Ok(Self {
            major: parts.pop_front().unwrap(),
            minor: parts.pop_front().unwrap(),
            patch: parts.pop_front().unwrap(),
        })
    }
}

impl Ord for AppVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
            _ => (),
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
            _ => (),
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
            _ => (),
        }
        Ordering::Equal
    }
}

impl PartialOrd for AppVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AppVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for AppVersion {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WrongPartsNumber => write!(
                f,
                "Version should consist of three parts: major.minor.patch"
            ),
            Error::PartParsingError(err) => write!(f, "Version parsing error: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::AppVersion;

    #[test]
    fn test_parsing() {
        for major in 1..10 {
            for minor in 1..20 {
                for patch in 1..30 {
                    let version_name = format!("{major}.{minor}.{patch}");
                    let app_version = version_name.parse::<AppVersion>();
                    assert!(app_version.is_ok());
                    assert_eq!(version_name, app_version.unwrap().to_string());
                }
            }
        }
    }

    #[test]
    fn test_comparison() {
        let v1_11 = "1.11.0".parse::<AppVersion>().unwrap();
        let v1_10 = "1.10.3".parse::<AppVersion>().unwrap();
        let v1_12 = "1.12.0".parse::<AppVersion>().unwrap();
        let v2_00 = "2.0.2".parse::<AppVersion>().unwrap();

        assert!(v1_11 > v1_10);
        assert!(v1_10 < v1_12);
        assert!(v1_12 < v2_00);
        assert!(v2_00 > v1_11);
    }
}
