use std::{collections::HashMap, sync::Mutex};

use actix_session::{config::BrowserSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App,
};
use awc::Client;
use http_replay_proxy::{cli::CliArguments, records::{RecordOptions, SessionState, RecordSession}, app::{forward, end_record_handler, start_record_handler}};
use url::Url;

pub async fn create_app(args: CliArguments) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let forward_url = Url::parse(&args.forward_to).expect("Forward address invalid");

    let record_options = RecordOptions {
        record_dir: args.record_dir,
        record: args.record,
    };

    let record_sessions = web::Data::new(SessionState {
        sessions: Mutex::new(HashMap::<String, RecordSession>::new()),
    });
    return App::new()
        .app_data(web::Data::new(Client::default()))
        .app_data(web::Data::new(forward_url.clone()))
        .app_data(web::Data::new(record_options.clone()))
        .app_data(record_sessions.clone())
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                .cookie_secure(false)
                .session_lifecycle(BrowserSession::default())
                .cookie_http_only(true)
                .build(),
        )
        .route("/end-record", web::post().to(end_record_handler))
        .route("/start-record/{name}", web::post().to(start_record_handler))
        .default_service(web::to(forward));
}