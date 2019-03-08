use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct PlacerError<'a> {
    pub why: &'a str,
}

impl<'a> Error for PlacerError<'a> { }

impl<'a> Display for PlacerError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Placer Error: {}", self.why)?;

        Ok(())
    }
}

impl<'a> Debug for PlacerError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Placer Error: {}", self.why)?;

        Ok(())
    }
}
