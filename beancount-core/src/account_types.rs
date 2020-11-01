/// Allowed account types.
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.17ry42rqbuiu>
#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
pub enum AccountType {
    Assets,
    Liabilities,
    Equity,
    Income,
    Expenses,
}

impl AccountType {
    /// Get the default name for this account type.
    ///
    /// # Example
    /// ```rust
    /// use AccountType::*;
    /// assert_eq!(Assets.default_name(), "Assets");
    /// assert_eq!(Liabilities.default_name(), "Liabilities");
    /// assert_eq!(Equity.default_name(), "Equity");
    /// assert_eq!(Income.default_name(), "Income");
    /// assert_eq!(Expenses.default_name(), "Expenses");
    /// ```
    pub fn default_name(&self) -> &'static str {
        use AccountType::*;
        match self {
            Assets => "Assets",
            Liabilities => "Liabilities",
            Equity => "Equity",
            Income => "Income",
            Expenses => "Expenses",
        }
    }
}
