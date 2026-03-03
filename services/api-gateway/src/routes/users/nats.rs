use std::time::Duration;

use contracts::users::list_users::{
    Request as ListUsersRequest,
    Response as ListUsersResponse,
    SUBJECT as LIST_USERS_SUBJECT,
};

use crate::{app::AppState, error::AppError, nats};

const USER_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn request_list_users(
    state: &AppState,
    limit: i64,
    offset: i64,
) -> Result<ListUsersResponse, AppError> {
    nats::request_json(
        state,
        LIST_USERS_SUBJECT,
        &ListUsersRequest { limit, offset },
        USER_REQUEST_TIMEOUT,
        "user-service request timed out",
        "user-service request failed",
        "can't serialize list users request",
        "can't deserialize list users response",
    )
    .await
}
