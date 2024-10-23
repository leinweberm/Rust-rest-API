#![allow(dead_code)]
use std::error::Error;
use serde::Serialize;
use warp::reject::Rejection;
use warp::reject;
use warp::http::StatusCode;
use warp::reply::WithStatus;

use crate::requests::dto::generic_response::{GenericResponse, Status};

pub enum ApiErrors {
	UnauthorizedError,
	ValidationError
}

/// # UnauthorizedError
#[derive(Debug, Serialize)]
pub struct UnauthorizedError {
	status: Status,
	#[serde(skip_serializing)]
	status_code: StatusCode,
	message: String
}

impl UnauthorizedError {
	pub fn new() -> Self {
		Self {
			status: Status::Error,
			status_code: StatusCode::UNAUTHORIZED,
			message: String::from("unauthorized")
		}
	}

	pub async fn response(&self) -> WithStatus<warp::reply::Json> {
		let response = GenericResponse::<UnauthorizedError> {
			status: self.status,
			message: &self.message,
			data: None
		};
		warp::reply::with_status(warp::reply::json(&response), self.status_code)
	}
}

impl reject::Reject for UnauthorizedError {}

/// # TokenExpiredError
#[derive(Debug, Serialize)]
pub struct TokenExpiredError {
	status: Status,
	#[serde(skip_serializing)]
	status_code: StatusCode,
	message: String
}

impl TokenExpiredError {
	pub fn new() -> Self {
		Self {
			status: Status::Error,
			status_code: StatusCode::UNAUTHORIZED,
			message: String::from("tokenExpired")
		}
	}

	pub async fn response(&self) -> WithStatus<warp::reply::Json> {
		let response = GenericResponse::<UnauthorizedError> {
			status: self.status,
			message: &self.message,
			data: None
		};

		warp::reply::with_status(warp::reply::json(&response), self.status_code)
	}
}

impl reject::Reject for TokenExpiredError {}

/// # ValidationError
#[derive(Debug, Serialize)]
pub struct ValidationError {
	status: Status,
	#[serde(skip_serializing)]
	status_code: StatusCode,
	message: String
}

impl ValidationError {
	pub fn new(error: Option<&str>) -> Self {
		Self {
			status: Status::Error,
			status_code: StatusCode::BAD_REQUEST,
			message: error.unwrap_or("validationError").to_string()
		}
	}

	pub async fn response(&self) -> WithStatus<warp::reply::Json> {
		let response = GenericResponse::<ValidationError> {
			status: self.status,
			message: &self.message,
			data: None
		};
		warp::reply::with_status(warp::reply::json(&response), self.status_code)
	}
}

impl reject::Reject for ValidationError {}

/// # NotFoundError
#[derive(Debug, Serialize)]
pub struct NotFoundError {
	status: Status,
	#[serde(skip_serializing)]
	status_code: StatusCode,
	message: String
}

impl NotFoundError {
	pub fn new() -> Self {
		Self {
			status: Status::Error,
			status_code: StatusCode::NOT_FOUND,
			message: String::from("notFound")
		}
	}

	pub async fn response(&self) -> WithStatus<warp::reply::Json> {
		let response = GenericResponse::<NotFoundError> {
			status: self.status,
			message: &self.message,
			data: None
		};
		warp::reply::with_status(warp::reply::json(&response), self.status_code)
	}
}

impl reject::Reject for NotFoundError {}

/// # InternalServerError
#[derive(Debug, Serialize)]
pub struct InternalServerError {
	status: Status,
	#[serde(skip_serializing)]
	status_code: StatusCode,
	message: String
}

impl InternalServerError {
	pub fn new() -> Self {
		Self {
			status: Status::Error,
			status_code: StatusCode::INTERNAL_SERVER_ERROR,
			message: String::from("internalServerError")
		}
	}

	pub async fn response(&self) -> WithStatus<warp::reply::Json> {
		let response = GenericResponse::<InternalServerError> {
			status: self.status,
			message: &self.message,
			data: None
		};
		warp::reply::with_status(warp::reply::json(&response), self.status_code)
	}
}

impl reject::Reject for InternalServerError {}

pub async fn handle_rejection(error: Rejection) -> Result<impl warp::Reply, Rejection> {
	error!(target: "api", "request error - {:?}", error);
	let mut response = GenericResponse::<()> {
			status: Status::Error,
			message: "internalServerError",
			data: None
		};

	if error.is_not_found() {
		response.message = "notFound";
		return Ok(warp::reply::with_status(warp::reply::json(&response), StatusCode::NOT_FOUND))

	} else if let Some(not_found_error) = error.find::<NotFoundError>() {
		return Ok(not_found_error.response().await)

	} else if let Some(unauthorized_error) = error.find::<UnauthorizedError>() {
		return Ok(unauthorized_error.response().await)

	} else if let Some(validation_error) = error.find::<ValidationError>() {
		return Ok(validation_error.response().await)

	} else if let Some(internal_server_error) = error.find::<InternalServerError>() {
		return Ok(internal_server_error.response().await)

	} else if let Some(token_expired_error) = error.find::<TokenExpiredError>() {
		return Ok(token_expired_error.response().await)

	} else if let Some(_) = error.find::<warp::reject::MethodNotAllowed>() {
		response.message = "methodNotAllowed";
		return Ok(warp::reply::with_status(warp::reply::json(&response), StatusCode::METHOD_NOT_ALLOWED))

	} else if let Some(body_deserialize_error) = error.find::<warp::filters::body::BodyDeserializeError>() {
		match body_deserialize_error.source() {
			Some(cause) => {
				error!(target: "api", "BodyDeserializeError - {:?}", cause);
				let msg = format!("validationError - {}", cause);
        let validation_error = GenericResponse::<()> {
					status: Status::Error,
					message: &msg,
					data: None
				};
				return Ok(warp::reply::with_status(warp::reply::json(&validation_error), StatusCode::BAD_REQUEST))
			},
			None => {
				response.message = "badRequest";
				return Ok(warp::reply::with_status(warp::reply::json(&response), StatusCode::BAD_REQUEST))
			},
		}
	}

	Ok(warp::reply::with_status(warp::reply::json(&response), StatusCode::INTERNAL_SERVER_ERROR))
}