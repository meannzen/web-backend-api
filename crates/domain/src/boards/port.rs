use async_trait::async_trait;

use crate::boards::errors::BoardError;
use crate::boards::model::{Board, BoardCursor, BoardId, BoardListQuery, NewBoard};

#[async_trait]
pub trait BoardRepository: Send + Sync + 'static {
    async fn create(&self, new_board: NewBoard) -> Result<Board, BoardError>;
    async fn find_by_id(&self, id: &BoardId) -> Result<Board, BoardError>;
    async fn list(
        &self,
        query: &BoardListQuery,
        after: Option<BoardCursor>,
        limit: u32,
    ) -> Result<(Vec<Board>, bool), BoardError>;
}
