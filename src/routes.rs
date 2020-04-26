use crate::*;
use crate::session;
use env_logger;
use warp::{filters::cookie, path, body::json, body::form, filters::query::query};

// I had trouble decoupling routes from server -- couldnt figure out the return type
pub async fn run_server() {
    // NOT TESTED YET
    let public = false; // std::env::var("PUBLIC").unwrap_or("false");
    let session_filter = move || session::create_session_filter(public).clone();


    // we have to pass the full paths for redirect to work without javascript
    let home = warp::path::end()
        .and(session_filter())
        .and(query())
        .and(path::full())
        .map(render_timeline);

    let user_page = session_filter()
        .and(path!("user" / String))
        .and(form())
        .and(path::full())
        .map(user_page);

    let note_page = session_filter()
        .and(path!("note" / i32))
        .and(path::full())
        .map(note_page);

    let notification_page = session_filter()
        .and(path("notifications"))
        .map(render_notifications);

    let server_info_page = session_filter()
        .and(path("server"))
        .map(server_info_page);

    // auth functions
    let register_page = path("register")
        .and(query())
        .map(register_page);

    let do_register = path("register")
        .and(form())
        .and(query())
        .map(do_register);

    let login_page = path("login")
        .map(|| login_page());

    // TODO redirect these login pages
    let do_login = path("login")
        .and(form())
        .map(do_login);

    let do_logout = path("logout")
        .and(cookie::cookie("EXAUTH"))
        .map(do_logout);

    // CRUD actions
    let create_note = path("create_note")
        .and(session_filter())
        .and(form())
        // Verbose -- see if you can refactor
        .map(|u: Option<User>, f: NewNoteRequest| match u {
            Some(u) => {
                new_note(u, &f.note_input).unwrap(); // TODO fix unwrap
                let red_url: http::Uri = f.redirect_url.parse().unwrap();
                redirect(red_url)},
            None => redirect(warp::http::Uri::from_static("error"))});

    let delete_note = path("delete_note")
        .and(session_filter())
        .and(form())
        .map(|u: Option<User>, f: DeleteNoteRequest | match u {
            Some(u) => {
                delete_note(f.note_id).unwrap(); // TODO fix unwrap
                let red_url: http::Uri = f.redirect_url.parse().unwrap();
                redirect(red_url)},
            None => redirect(warp::http::Uri::from_static("error"))});


    let static_files = warp::path("static")
        .and(warp::fs::dir("./static"));

    // activityPub stuff
    //
    // POST
    let post_user_inbox = path!("user" / String / "inbox.json" )
        .and(json())
        .map(ap::post_user_inbox);

    let post_user_outbox = path!("user" / String / "outbox.json" )
        .and(json())
        .map(ap::post_user_outbox);

    let get_user_outbox = path!("user" / String / "outbox.json" )
        .map(ap::get_user_outbox);

    // let get_user_inbox = path!("user" / String / "outbox.json" )
    //     .and(json())
    //     .map(ap::post_user_outbox);

    let user_followers = path!("user" / String / "followers.json" )
        .map(ap::user_followers);

    let user_following = path!("user" / String / "following.json" )
        .map(ap::user_following);

    // https://github.com/seanmonstar/warp/issues/42 -- how to set up diesel
    // TODO set content length limit 
    // TODO redirect via redirect in request
    // TODO secure against xss
        // used for api based authentication
    // let api_filter = session::create_session_filter(&POOL.get());
    let html_renders = home.or(login_page).or(register_page).or(user_page).or(note_page).or(server_info_page).or(notification_page);
    let forms = do_register.or(do_login).or(do_logout).or(create_note).or(delete_note);
    // let api
    // catch all for any other paths

    let routes = warp::get().and(html_renders)
        .or(
            warp::post()
            .and(warp::body::content_length_limit(1024 * 32))
            .and(forms))
        .or(static_files)
        .with(warp::log("server"))
        .recover(handle_rejection)
        .boxed();
    env_logger::init();
    match std::env::var("GOURAMI_ENV").unwrap().as_str() {
    "PROD" => warp::serve(routes)
        .tls()
        .cert_path(&std::env::var("CERT_PATH").unwrap())
        .key_path(&std::env::var("KEY_PATH").unwrap())
        .run(([0, 0, 0, 0], 443))
        .await ,
    _ => warp::serve(routes).run(([127,0,0,1], 3030)).await
    }
}
