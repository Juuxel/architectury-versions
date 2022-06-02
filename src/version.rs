use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    pub components: Vec<u32>,
    pub snapshot: Option<String>,
}

impl FromStr for Version {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut base_version = s;
        let snapshot = if s.contains('-') {
            let (base, snapshot) = s.split_once('-').unwrap();
            base_version = base;
            Some(String::from(snapshot))
        } else {
            None
        };

        let components = base_version
            .split('.')
            .map(|component| component.parse())
            .collect::<Result<Vec<u32>, ParseIntError>>()?;

        Ok(Version {
            components,
            snapshot,
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut components = self.components.iter();
        write!(f, "{}", components.next().unwrap())?;

        for component in components {
            write!(f, ".{}", component)?;
        }

        if let Some(snapshot) = &self.snapshot {
            write!(f, "-{}", snapshot)?;
        }

        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        let component_count = std::cmp::max(self.components.len(), other.components.len());
        for index in 0..component_count {
            let self_component = self.components.get(index).unwrap_or(&0);
            let other_component = other.components.get(index).unwrap_or(&0);

            if self_component != other_component {
                return self_component.cmp(other_component);
            }
        }

        // All components are equal; start checking snapshots.
        // No snapshot > snapshot if the components are equal.
        if self.snapshot.is_some() && other.snapshot.is_none() {
            return Ordering::Less;
        } else if self.snapshot.is_none() && other.snapshot.is_some() {
            return Ordering::Greater;
        }

        // Snapshots are compared lexicographically.
        if let Some(self_snapshot) = &self.snapshot {
            if let Some(other_snapshot) = &other.snapshot {
                return self_snapshot.cmp(other_snapshot);
            }
        }

        return Ordering::Equal;
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
