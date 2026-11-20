use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::boards::errors::BoardError;
use domain::boards::model::{
    Board, BoardCode, BoardCursor, BoardId, BoardListQuery, BoardName, NewBoard,
};
use domain::boards::port::BoardRepository;
use domain::pagination::SortField as _;
use sqlx::PgPool;
use uuid::Uuid;

use crate::pagination;

#[derive(sqlx::FromRow)]
struct BoardRow {
    id: Uuid,
    name: String,
    code: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<BoardRow> for Board {
    type Error = anyhow::Error;

    fn try_from(row: BoardRow) -> Result<Self, Self::Error> {
        let name = BoardName::parse(&row.name).map_err(|e| anyhow::anyhow!(e))?;
        let code = BoardCode::parse(&row.code).map_err(|e| anyhow::anyhow!(e))?;
        Ok(Board::new(
            BoardId::from(row.id),
            NewBoard { name, code },
            row.created_at,
            row.updated_at,
        ))
    }
}

pub struct PgBoardRepository {
    pool: PgPool,
}

impl PgBoardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoardRepository for PgBoardRepository {
    async fn create(&self, new_board: NewBoard) -> Result<Board, BoardError> {
        let row = sqlx::query_as!(
            BoardRow,
            r#"
            INSERT INTO boards (name, code)
            VALUES ($1, $2)
            RETURNING id, name, code, created_at, updated_at
            "#,
            new_board.name.as_ref(),
            new_board.code.as_ref(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("boards_code_key") => {
                BoardError::CodeTaken
            }
            _ => BoardError::Internal(anyhow::anyhow!(e).context("failed to create board")),
        })?;

        Board::try_from(row).map_err(BoardError::Internal)
    }

    async fn find_by_id(&self, id: &BoardId) -> Result<Board, BoardError> {
        let row = sqlx::query_as!(
            BoardRow,
            r#"
            SELECT id, name, code, created_at, updated_at
            FROM boards WHERE id = $1
            "#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            BoardError::Internal(anyhow::anyhow!(e).context("failed to find board by id"))
        })?
        .ok_or(BoardError::NotFound)?;

        Board::try_from(row).map_err(BoardError::Internal)
    }

    async fn list(
        &self,
        query: &BoardListQuery,
        after: Option<BoardCursor>,
        limit: u32,
    ) -> Result<(Vec<Board>, bool), BoardError> {
        let fetch_limit = (limit + 1) as i64;
        let col = query.sort_by.column();
        let order = query.direction.sql_order();
        let op = query.direction.cursor_op();

        let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> =
            sqlx::QueryBuilder::new("SELECT id, name, code, created_at, updated_at FROM boards ");

        let mut has_where = false;

        if let Some(ref search) = query.filter.search {
            pagination::push_ilike_any_filter(
                &mut query_builder,
                &["name", "code"],
                search,
                &mut has_where,
            );
        }

        pagination::push_cursor_condition(
            &mut query_builder,
            col,
            op,
            after.as_ref().map(|c| (&c.value, c.id)),
            &mut has_where,
        );

        pagination::push_order_and_limit(&mut query_builder, col, order, fetch_limit);

        let rows = query_builder
            .build_query_as::<BoardRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                BoardError::Internal(anyhow::anyhow!(e).context("failed to list boards"))
            })?;

        let (rows, has_next_page) = pagination::split_has_next_page(rows, limit);

        let boards = rows
            .into_iter()
            .map(Board::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(BoardError::Internal)?;

        Ok((boards, has_next_page))
    }
}
