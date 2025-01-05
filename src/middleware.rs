use crate::files::ImageFileManager;
use crate::{CustomError, RedirectToLogin};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use warp::Filter;

pub fn with_auth(
    conn: Arc<Mutex<Connection>>,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::cookie::optional("session")
        .and(warp::any().map(move || conn.clone()))
        .and_then(
            |session: Option<String>, conn: Arc<Mutex<Connection>>| async move {
                match session {
                    None => Err(warp::reject::custom(RedirectToLogin)),
                    Some(session_id) => {
                        let conn_guard = conn.lock().map_err(|_| {
                            warp::reject::custom(CustomError {
                                message: "Internal server error".to_string(),
                            })
                        })?;

                        let is_valid = crate::database::verify_session(&conn_guard, &session_id)
                            .map_err(|_| {
                                warp::reject::custom(CustomError {
                                    message: "Authentication error".to_string(),
                                })
                            })?;

                        if !is_valid {
                            return Err(warp::reject::custom(RedirectToLogin));
                        }

                        Ok(())
                    }
                }
            },
        )
        .untuple_one()
}

pub fn with_file_manager(
    file_manager: Arc<ImageFileManager>,
) -> impl Filter<Extract = (Arc<ImageFileManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || file_manager.clone())
}
