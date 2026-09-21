//! # Tiny sqlx QueryBuilder Helpers
//!
//! ## Examples
//!
//! ```rust
//! // normal query builder instance
//! let mut query = sqlx::QueryBuilder::<Postgres>::new(
//!     r#"
//!    SELECT
//!        "P"."id",
//!        "P"."name",
//!        "P"."price",
//!        "P"."created_at"
//!    FROM "products" AS "P"
//!    "#,
//! );
//!
//! // add where clause
//! {
//!     let mut where_clause = Where::new(&mut query);
//!
//!     // add product_name condition if product_name is not None
//!     where_clause.and_opt(product_name.as_deref(), |q, name| {
//!         q.push(r#""P"."name" ILIKE "#)
//!             .push_bind(format!("%{name}%"));
//!     });
//!
//!     // add brand_name condition if brand_name is not empty
//!     where_clause.and_if(!brand_names.is_empty(), |q| {
//!         q.push(r#""P"."brand_name" = ANY("#)
//!             .push_bind(&brand_names)
//!             .push(")");
//!     });
//! }
//!
//! // add order by clause
//! {
//!     let mut order_by = OrderBy::new(&mut query);
//!
//!     for sort in &sorts {
//!         order_by.push(|q| {
//!             q.push(match sort.key {
//!                 SortKey::Name => r#""P"."name""#,
//!                 SortKey::Price => r#""P"."price""#,
//!                 SortKey::CreatedAt => r#""P"."created_at""#,
//!             });
//!
//!             q.push(match sort.direction {
//!                 SortDirection::Asc => " ASC",
//!                 SortDirection::Desc => " DESC",
//!             });
//!         });
//!     }
//! }
//!
//! // normal QueryBuilder modification
//! query
//!     .push(" LIMIT ")
//!     .push_bind(limit)
//!     .push(" OFFSET ")
//!     .push_bind(offset);
//!
//! let products = query.build_query_as::<Product>().fetch_all(pool).await?;
//! ```
mod order_helper;
mod where_helper;

pub use order_helper::OrderBy;
pub use where_helper::Where;

#[cfg(test)]
mod tests {
    use assert2::assert as require;
    use sqlx::Postgres;

    use super::*;

    #[test]
    fn where_adds_where_for_first_condition() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut where_ = Where::new(&mut query);

            where_.and(|q| {
                q.push("name = 'foo'");
            });
        }

        require!(query.sql() == "SELECT * FROM products WHERE name = 'foo'");
    }

    #[test]
    fn where_joins_multiple_conditions_with_and() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut where_ = Where::new(&mut query);

            where_
                .and(|q| {
                    q.push("name = 'foo'");
                })
                .and(|q| {
                    q.push("price >= 100");
                })
                .and(|q| {
                    q.push("active = TRUE");
                });
        }

        require!(
            query.sql()
                == concat!(
                    "SELECT * FROM products",
                    " WHERE name = 'foo'",
                    " AND price >= 100",
                    " AND active = TRUE",
                )
        );
    }

    #[test]
    fn where_and_if_ignores_false_condition() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut where_ = Where::new(&mut query);

            where_
                .and_if(false, |q| {
                    q.push("name = 'foo'");
                })
                .and_if(true, |q| {
                    q.push("active = TRUE");
                });
        }

        require!(query.sql() == "SELECT * FROM products WHERE active = TRUE");
    }

    #[test]
    fn where_and_opt_ignores_none() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        let name: Option<&str> = None;
        let price = Some(100);

        {
            let mut where_ = Where::new(&mut query);

            where_
                .and_opt(name, |q, name| {
                    q.push("name = '").push(name).push("'");
                })
                .and_opt(price, |q, price| {
                    q.push("price >= ").push(price.to_string());
                });
        }

        require!(query.sql() == "SELECT * FROM products WHERE price >= 100");
    }

    #[test]
    fn where_does_nothing_when_no_conditions_are_added() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut where_ = Where::new(&mut query);

            where_.and_if(false, |q| {
                q.push("active = TRUE");
            });
        }

        require!(query.sql() == "SELECT * FROM products");
    }

    #[test]
    fn order_by_adds_order_by_for_first_expression() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut order_by = OrderBy::new(&mut query);

            order_by.push(|q| {
                q.push("name ASC");
            });
        }

        require!(query.sql() == "SELECT * FROM products ORDER BY name ASC");
    }

    #[test]
    fn order_by_separates_multiple_expressions_with_comma() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut order_by = OrderBy::new(&mut query);

            order_by
                .push(|q| {
                    q.push("name ASC");
                })
                .push(|q| {
                    q.push("price DESC");
                })
                .push(|q| {
                    q.push("created_at DESC");
                });
        }

        require!(
            query.sql()
                == concat!(
                    "SELECT * FROM products",
                    " ORDER BY name ASC",
                    ", price DESC",
                    ", created_at DESC",
                )
        );
    }

    #[test]
    fn order_by_push_if_ignores_false_condition() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut order_by = OrderBy::new(&mut query);

            order_by
                .push_if(false, |q| {
                    q.push("name ASC");
                })
                .push_if(true, |q| {
                    q.push("price DESC");
                });
        }

        require!(query.sql() == "SELECT * FROM products ORDER BY price DESC");
    }

    #[test]
    fn order_by_does_nothing_when_no_expressions_are_added() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut order_by = OrderBy::new(&mut query);

            order_by.push_if(false, |q| {
                q.push("name ASC");
            });
        }

        require!(query.sql() == "SELECT * FROM products");
    }

    #[test]
    fn where_and_order_by_can_be_combined() {
        let mut query = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM products");

        {
            let mut where_ = Where::new(&mut query);

            where_
                .and(|q| {
                    q.push("active = TRUE");
                })
                .and(|q| {
                    q.push("price >= 100");
                });
        }

        {
            let mut order_by = OrderBy::new(&mut query);

            order_by
                .push(|q| {
                    q.push("name ASC");
                })
                .push(|q| {
                    q.push("price DESC");
                });
        }

        require!(
            query.sql()
                == concat!(
                    "SELECT * FROM products",
                    " WHERE active = TRUE",
                    " AND price >= 100",
                    " ORDER BY name ASC",
                    ", price DESC",
                )
        );
    }
}
