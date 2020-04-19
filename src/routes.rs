use warp;

def get_routes() {
    let notifications = warp::path("notifications");

    // How does this interact with tokio? who knows!
    let test = warp::path("test").map(|| "Hello world");

    let register = warp::path("register").map(|| "Hello from register");
    let login = warp::path("login").map(|| "Hello from login");
    let logout = warp::path("logout").map(|| "Hello from logout");

    // post
    // user
    // default page -- timeline
    let home = warp::path::end()
        .map(|| render_template(&TimelineTemplate{
            page: "timeline",
            logged_in: true,
            notes: Note::get_for_user(&SqliteConnection::establish("sample.db").unwrap(), 1),
            username: "alex", 
            title: "gourami"}));

    let static_files = warp::path("static")
            .and(warp::fs::dir("./static"));

    // https://github.com/seanmonstar/warp/issues/42 -- how to set up diesel
    // TODO set content length limit 
    // TODO redirect via redirect in request
    // TODO secure against xss
    let create_note = warp::path("create_note")
        .and(warp::body::form())
        .map(|note_req: NewNoteRequest| new_note(&note_req));

    let delete_note = warp::path::param::<i32>()
        .and(warp::path("delete"))
        .map(|note_id| delete_note(note_id));

    // catch all for any other paths
    let not_found = warp::any().map(|| "404 not found");

    let routes = warp::get().and(
        home.or(test).or(static_files).or(not_found))
        .or(warp::post().and(create_note.or(delete_note)));
    routes
}

