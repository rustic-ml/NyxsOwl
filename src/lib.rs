//! # NyxsOwl
//!
//! `nyxs_owl` is a Rust library that provides utilities for the NyxsOwl project.
//!
//! ## Example
//!
//! ```
//! use nyxs_owl::Owl;
//!
//! let my_owl = Owl::new("Hedwig");
//! assert_eq!(my_owl.name(), "Hedwig");
//! ```

/// An owl from the NyxsOwl project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owl {
    name: String,
    wisdom_level: u32,
}

impl Owl {
    /// Creates a new owl with the given name and a default wisdom level of 10.
    ///
    /// # Examples
    ///
    /// ```
    /// use nyxs_owl::Owl;
    ///
    /// let owl = Owl::new("Hedwig");
    /// assert_eq!(owl.name(), "Hedwig");
    /// assert_eq!(owl.wisdom_level(), 10);
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            wisdom_level: 10,
        }
    }

    /// Creates a new owl with a specific name and wisdom level.
    ///
    /// # Examples
    ///
    /// ```
    /// use nyxs_owl::Owl;
    ///
    /// let owl = Owl::with_wisdom("Archimedes", 100);
    /// assert_eq!(owl.name(), "Archimedes");
    /// assert_eq!(owl.wisdom_level(), 100);
    /// ```
    pub fn with_wisdom(name: &str, wisdom_level: u32) -> Self {
        Self {
            name: name.to_string(),
            wisdom_level,
        }
    }

    /// Returns the name of the owl.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the wisdom level of the owl.
    pub fn wisdom_level(&self) -> u32 {
        self.wisdom_level
    }

    /// Increases the owl's wisdom by the given amount.
    pub fn gain_wisdom(&mut self, amount: u32) {
        self.wisdom_level += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_owl() {
        let owl = Owl::new("Hedwig");
        assert_eq!(owl.name(), "Hedwig");
        assert_eq!(owl.wisdom_level(), 10);
    }

    #[test]
    fn test_with_wisdom() {
        let owl = Owl::with_wisdom("Archimedes", 100);
        assert_eq!(owl.name(), "Archimedes");
        assert_eq!(owl.wisdom_level(), 100);
    }

    #[test]
    fn test_gain_wisdom() {
        let mut owl = Owl::new("Hedwig");
        owl.gain_wisdom(5);
        assert_eq!(owl.wisdom_level(), 15);
    }
}
