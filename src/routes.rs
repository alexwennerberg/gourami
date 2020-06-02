use crate::session;
use crate::*;
use env_logger;
use http::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use std::time::Duration;
use warp::reject::{self, Rejection};
use warp::reply::Response;
use warp::{
    body, body::form, body::json, filters::cookie, filters::query::query, header, path, reply,
};

pub async fn run_server() {
    // NOT TESTED YET
    let public = std::env::var("PUBLIC").ok() == Some("1".to_owned());
    let session_filter = move || session::create_session_filter(public).clone();
    let private_session_filter = move || session::create_session_filter(false).clone();

    // Background worker for sending activitypub messages
    // TODO -- Improve concurrency. each request is blocking.
    let (snd, mut rcv) = tokio::sync::mpsc::unbounded_channel::<(Value, Vec<String>)>();
    tokio::spawn(async move {
        while let Some((msg, destinations)) = rcv.recv().await {
            for destination in destinations {
                // no retries or anything yet
                let res = ap::send_ap_message(&msg, destination).await.ok();
                if res.is_none() {
                    error!("AP message sending failed");
                }
            }
            // Run requst
        }
    });

    let with_sender = warp::any().map(move || snd.clone());

    let webfinger = warp::path!(".well-known" / "webfinger")
        .and(query())
        // TODO content type
        .map(|q| reply::json(&ap::webfinger_json(q)));

    let actor_json = warp::path::end()
        // In practice, the headers may not follow the spec
        // https://www.w3.org/TR/activitypub/#retrieving-objects
        // TODO content type
        // TODO get interop with mastodon working
        .and(
            header::exact_ignore_case(
                "accept",
                r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
            )
            .or(header::exact_ignore_case(
                "accept",
                r#"application/ld+json"#,
            ))
            .or(header::exact_ignore_case(
                "accept",
                r#"profile="https://www.w3.org/ns/activitystreams""#,
            ))
            .or(header::exact_ignore_case("accept", "application/json")),
        )
        .map(
            |_| reply::json(&ap::server_actor_json()), // how do async work
        );

    let home = warp::path::end()
        .and(session_filter())
        .and(query())
        .and(path::full())
        .map(|a, p, u| render_timeline(a, &p, u, get_notes(&p)));

    let user_page = session_filter()
        .and(path!("user" / String))
        .and(query())
        .and(path::full())
        .map(user_page);

    let user_edit_page = private_session_filter()
        .and(path!("user" / String / "edit"))
        .map(render_user_edit_page);

    let edit_user = private_session_filter()
        .and(path!("user" / String / "edit"))
        .and(form())
        .map(edit_user);

    let note_page = session_filter()
        .and(path!("note" / i32))
        .and(path::full())
        .map(note_page);

    let server_info_page = session_filter()
        .and(path("server_info"))
        .map(server_info_page);

    // auth functions
    let register_page = path("register").and(query()).map(register_page);

    let do_register = path("register").and(form()).and(query()).map(do_register);

    let login_page = path("login").map(|| login_page());

    // TODO redirect these login pages
    let do_login = path("login").and(form()).map(do_login);

    let do_logout = path("logout").and(cookie::cookie("EXAUTH")).map(do_logout);

    let create_note = path("create_note")
        .and(private_session_filter())
        .and(form())
        .and(with_sender)
        .and_then(handle_new_note_form);

    let delete_note = path("delete_note")
        .and(private_session_filter())
        .and(form())
        .map(|u: Option<User>, f: DeleteNoteRequest| match u {
            Some(u) => {
                delete_note(f.note_id).unwrap(); // TODO fix unwrap
                let red_url: http::Uri = f.redirect_url.parse().unwrap();
                redirect(red_url)
            }
            None => redirect(http::Uri::from_static("error")),
        });

    // couldn't figure out how to get this folder to render on root properly
    let robots = warp::path("robots.txt").and(warp::fs::file("./static/robots.txt"));
    let static_files = warp::path("static")
        .and(warp::fs::dir("./static"))
        .or(robots);

    // force content type to be application/ld+json; profile="https://www.w3.org/ns/activitystreams
    let post_server_inbox = path!("inbox")
        .and(body::aggregate()) // TODO -- figure out whats going wrong with content type here from mastodon
        // .and(
        //     header::exact_ignore_case(
        //         "content-type",
        //         r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
        //     )
        //     .or(header::exact_ignore_case(
        //         "content-type",
        //         r#"application/ld+json"#,
        //     ))
        //     .or(header::exact_ignore_case(
        //         "content-type",
        //         r#"profile="https://www.w3.org/ns/activitystreams""#,
        //     )),
        // )
        .and(header::headers_cloned())
        .and_then(|buf, headers| async move { post_inbox(buf, headers).await });

    let get_server_outbox = path!("outbox").map(get_outbox);

    // https://github.com/seanmonstar/warp/issues/42 -- how to set up diesel
    // TODO set content length limit
    // TODO redirect via redirect in request
    // TODO secure against xss
    // used for api based authentication
    // let api_filter = session::create_session_filter(&POOL.get());
    let static_json = actor_json.or(webfinger); // rename html renders
    let html_renders = home
        .or(login_page)
        .or(register_page)
        .or(user_page)
        .or(note_page)
        .or(server_info_page)
        .or(user_edit_page);
    let forms = do_register
        .or(do_login)
        .or(do_logout)
        .or(create_note)
        .or(delete_note)
        .or(edit_user);
    let api_post = post_server_inbox;

    let routes = warp::get()
        .and(static_json.or(html_renders))
        .or(warp::post()
            .and(warp::body::content_length_limit(1024 * 32))
            .and(forms))
        .or(warp::post()
            .and(warp::body::content_length_limit(1024 * 1024))
            .and(api_post))
        .or(static_files)
        .with(warp::log("server"))
        .recover(handle_rejection)
        .boxed();
    match std::env::var("SSL_ENABLED").unwrap().as_str() {
        "1" => {
            warp::serve(routes)
                .tls()
                .cert_path(&std::env::var("CERT_PATH").unwrap())
                .key_path(&std::env::var("KEY_PATH").unwrap())
                .run(([0, 0, 0, 0], 443))
                .await
        }
        _ => warp::serve(routes).run(([127, 0, 0, 1], 3030)).await,
    }
}
