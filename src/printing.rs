use core::fmt;

use crate::{Cfg, Predicate};

impl fmt::Display for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#[cfg({})]", self.0)
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Predicate::*;

        match self {
            Any(predicates) => {
                f.write_str("any(")?;
                for (i, predicate) in predicates.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    predicate.fmt(f)?;
                }
                f.write_str(")")
            }
            All(predicates) => {
                f.write_str("all(")?;
                for (i, predicate) in predicates.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    predicate.fmt(f)?;
                }
                f.write_str(")")
            }
            Not(predicate) => write!(f, "not({})", predicate),
            Name(name) => f.write_str(&name),
            NameValue(name, value) => write!(f, "{} = \"{}\"", name, value),
        }
    }
}
