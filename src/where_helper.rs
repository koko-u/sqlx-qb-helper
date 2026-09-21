/// QueryBuilder adapter for adding where clauses
pub struct Where<'q, DB>
where
    DB: sqlx::Database,
{
    query: &'q mut sqlx::QueryBuilder<DB>,
    has_condition: bool,
}

impl<'q, DB> Where<'q, DB>
where
    DB: sqlx::Database,
{
    /// create where adapter from QueryBuilder
    pub fn new(query: &'q mut sqlx::QueryBuilder<DB>) -> Self {
        Self {
            query,
            has_condition: false,
        }
    }

    /// add where clause section
    ///
    /// # examples
    /// ```rust
    /// where_clause.and(|qb| {
    ///     qb.push(r#""name" = "#)
    ///       .push_bind(&name);
    /// );
    /// ```
    pub fn and(&mut self, f: impl FnOnce(&mut sqlx::QueryBuilder<DB>)) -> &mut Self {
        if self.has_condition {
            self.query.push(" AND ");
        } else {
            self.query.push(" WHERE ");
            self.has_condition = true;
        }

        f(self.query);

        self
    }

    /// add where clause section when the condition met
    ///
    /// # examples
    /// ```rust
    /// where_clause.and_if(!brand_names.is_empty(), |qb| {
    ///     qb.push(r#" "brand_name" = "#).push_bind(&brand_name);
    /// });
    /// ```
    pub fn and_if(&mut self, condition: bool, f: impl FnOnce(&mut sqlx::QueryBuilder<DB>)) -> &mut Self {
        if condition {
            self.and(f);
        }

        self
    }

    /// add where clause section when the value is Some(_)
    pub fn and_opt<T>(
        &mut self,
        value: Option<T>,
        f: impl FnOnce(&mut sqlx::QueryBuilder<DB>, T),
    ) -> &mut Self {
        if let Some(value) = value {
            self.and(|query| f(query, value));
        }

        self
    }
}
