use std::sync::Arc;

use crate::boards::errors::BoardError;
use crate::boards::model::{Board, BoardCode, BoardCursor, BoardId, BoardListQuery, NewBoard};
use crate::boards::port::BoardRepository;

pub struct BoardService {
    repo: Arc<dyn BoardRepository>,
}

impl BoardService {
    pub fn new(repo: Arc<dyn BoardRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, name: String, code: &str) -> Result<Board, BoardError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(BoardError::InvalidName("must not be blank".to_string()));
        }
        let code = BoardCode::parse(code).map_err(BoardError::InvalidCode)?;
        self.repo.create(NewBoard { name, code }).await
    }

    pub async fn get_by_id(&self, id: BoardId) -> Result<Board, BoardError> {
        self.repo.find_by_id(&id).await
    }

    pub async fn list(
        &self,
        query: &BoardListQuery,
        after: Option<BoardCursor>,
        limit: u32,
    ) -> Result<(Vec<Board>, bool), BoardError> {
        self.repo.list(query, after, limit).await
    }
}
