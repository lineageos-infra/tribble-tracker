// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::error;

use crate::database::DbError;

pub mod api;
pub mod internal;

pub(super) enum RouterError {
    Db(DbError),
    BadRequest(&'static str),
}

impl From<DbError> for RouterError {
    fn from(e: DbError) -> Self {
        RouterError::Db(e)
    }
}

impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        match self {
            RouterError::Db(e) => {
                error!("database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "sql error").into_response()
            }
            RouterError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
        }
    }
}
