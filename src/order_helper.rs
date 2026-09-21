pub struct OrderBy<'q, DB>
where
    DB: sqlx::Database,
{
    query: &'q mut sqlx::QueryBuilder<DB>,
    has_order: bool,
}

impl<'q, DB> OrderBy<'q, DB>
where
    DB: sqlx::Database,
{
    pub fn new(query: &'q mut sqlx::QueryBuilder<DB>) -> Self {
        Self {
            query,
            has_order: false,
        }
    }

    pub fn push(&mut self, f: impl FnOnce(&mut sqlx::QueryBuilder<DB>)) -> &mut Self {
        if self.has_order {
            self.query.push(", ");
        } else {
            self.query.push(" ORDER BY ");
            self.has_order = true;
        }

        f(self.query);

        self
    }

    pub fn push_if(&mut self, condition: bool, f: impl FnOnce(&mut sqlx::QueryBuilder<DB>)) -> &mut Self {
        if condition {
            self.push(f);
        }

        self
    }
}
