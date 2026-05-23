// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub mod api;
pub mod internal;

pub(super) enum RouterError {
    Db(sqlx::Error),
    BadRequest(&'static str),
}

impl From<sqlx::Error> for RouterError {
    fn from(e: sqlx::Error) -> Self {
        RouterError::Db(e)
    }
}

impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        match self {
            RouterError::Db(e) => {
                eprintln!("database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "sql error").into_response()
            }
            RouterError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
        }
    }
}
