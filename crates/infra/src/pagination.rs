use domain::pagination::CursorValue;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

/// Pushes a `WHERE`/`AND <column> ILIKE $n` clause with `%`/`_`/`\` escaped so the
/// search term is matched literally rather than as an ILIKE pattern.
pub fn push_ilike_filter(
    qb: &mut QueryBuilder<Postgres>,
    column: &str,
    value: &str,
    has_where: &mut bool,
) {
    push_ilike_any_filter(qb, &[column], value, has_where);
}

/// Like [`push_ilike_filter`], but matches if any of `columns` contains the term:
/// `WHERE`/`AND (<c1> ILIKE $n OR <c2> ILIKE $m ...)`.
pub fn push_ilike_any_filter(
    qb: &mut QueryBuilder<Postgres>,
    columns: &[&str],
    value: &str,
    has_where: &mut bool,
) {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{}%", escaped);

    qb.push(if *has_where { " AND (" } else { " WHERE (" });
    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            qb.push(" OR ");
        }
        qb.push(*column);
        qb.push(" ILIKE ");
        qb.push_bind(pattern.clone());
    }
    qb.push(")");
    *has_where = true;
}

/// Pushes `WHERE`/`AND <column> = $n` for a text value.
pub fn push_eq_filter(
    qb: &mut QueryBuilder<Postgres>,
    column: &str,
    value: String,
    has_where: &mut bool,
) {
    qb.push(if *has_where { " AND " } else { " WHERE " });
    qb.push(column);
    qb.push(" = ");
    qb.push_bind(value);
    *has_where = true;
}

/// Pushes `WHERE`/`AND <column> <op> $n` for a timestamp. `op` must be a trusted literal.
pub fn push_cmp_filter(
    qb: &mut QueryBuilder<Postgres>,
    column: &str,
    op: &str,
    value: chrono::DateTime<chrono::Utc>,
    has_where: &mut bool,
) {
    qb.push(if *has_where { " AND " } else { " WHERE " });
    qb.push(format!("{column} {op} "));
    qb.push_bind(value);
    *has_where = true;
}

/// Pushes the `(col, id) op (val, id)` keyset condition for cursor-based pagination.
pub fn push_cursor_condition(
    qb: &mut QueryBuilder<Postgres>,
    col: &str,
    op: &str,
    cursor: Option<(&CursorValue, Uuid)>,
    has_where: &mut bool,
) {
    let Some((value, id)) = cursor else { return };

    qb.push(if *has_where { " AND " } else { " WHERE " });
    qb.push(format!("({col}, id) {op} ("));
    match value {
        CursorValue::Timestamp(ts) => {
            qb.push_bind(*ts);
        }
        CursorValue::Text(s) => {
            qb.push_bind(s.clone());
        }
    }
    qb.push(", ");
    qb.push_bind(id);
    qb.push(")");
    *has_where = true;
}

pub fn push_order_and_limit(
    qb: &mut QueryBuilder<Postgres>,
    col: &str,
    order: &str,
    fetch_limit: i64,
) {
    qb.push(format!(" ORDER BY {col} {order}, id {order} "));
    qb.push(" LIMIT ");
    qb.push_bind(fetch_limit);
}

/// Given rows fetched with a `limit + 1` cap, splits off the lookahead row and
/// reports whether a next page exists.
pub fn split_has_next_page<T>(mut rows: Vec<T>, limit: u32) -> (Vec<T>, bool) {
    let has_next_page = rows.len() > limit as usize;
    rows.truncate(limit as usize);
    (rows, has_next_page)
}
