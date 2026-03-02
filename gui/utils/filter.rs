

/// # Filter
/// 
/// A filter is used for the searchbar.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Filter {
  Id(u32),
  Name(String),
  #[default]
  None,
}

#[allow(unused)]
impl Filter {
  /// # is_some
  /// Returns true if the filter is not None !
  #[inline]
  pub fn is_some(&self) -> bool {
    self != &Filter::None
  }

  /// # is_none
  /// Returns true if the filter is None !
  #[inline]
  pub fn is_none(&self) -> bool {
    self == &Filter::None
  }
}