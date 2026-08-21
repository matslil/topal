//! Prepared, schema-checked database adapter boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Integer(i64),
    Text(String),
    Null,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Integer,
    Text,
    Nullable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prepared {
    parameter_count: usize,
    row: Vec<Kind>,
}
impl Prepared {
    #[must_use]
    pub fn new(parameter_count: usize, row: Vec<Kind>) -> Self {
        Self {
            parameter_count,
            row,
        }
    }
    /// Validates parameters and rows without string interpolation.
    /// # Errors
    /// Returns an exact parameter or schema mismatch.
    pub fn validate(
        &self,
        parameters: &[Value],
        rows: &[Vec<Value>],
    ) -> Result<(), DatabaseFailure> {
        if parameters.len() != self.parameter_count {
            return Err(DatabaseFailure::ParameterCount);
        }
        if rows.iter().any(|row| {
            row.len() != self.row.len()
                || row.iter().zip(&self.row).any(|(value, kind)| {
                    !matches!(
                        (value, kind),
                        (Value::Integer(_), Kind::Integer)
                            | (Value::Text(_), Kind::Text)
                            | (Value::Null, Kind::Nullable)
                    )
                })
        }) {
            return Err(DatabaseFailure::SchemaMismatch);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseFailure {
    ParameterCount,
    SchemaMismatch,
    UncertainCommit,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rows_are_schema_checked() {
        let p = Prepared::new(1, vec![Kind::Integer]);
        assert_eq!(
            p.validate(
                &[Value::Text("x".into())],
                &[vec![Value::Text("bad".into())]]
            ),
            Err(DatabaseFailure::SchemaMismatch)
        );
    }
}
