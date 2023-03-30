use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use axum::{extract::State, http::StatusCode, response::Html, routing::get, Router};
use minijinja::{Environment, Source, UndefinedBehavior};
use serde_json::json;

#[derive(Default)]
struct AppState<'source> {
    template_rendering_env: Environment<'source>,
}

type SharedState<'source> = Arc<AppState<'source>>;

#[tokio::main]
async fn main() {
    let mut env = Environment::new();

    env.set_undefined_behavior(UndefinedBehavior::Chainable);

    let mut templates_source = Source::new();

    // some common templates are being registered here to the Source

    env.set_source(templates_source);

    let state = Arc::new(AppState {
        template_rendering_env: env,
    });

    let app = Router::new()
        .route("/", get(process_request))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn register_font_mut(
    font: String,
    available_fonts: &Vec<String>,
    registered_fonts: &mut HashSet<String>,
) -> String {
    if available_fonts.contains(&font) {
        registered_fonts.insert(font.clone());

        return font.clone();
    }

    "".to_string()
}

fn register_font(font: String, available_fonts: &Vec<String>) -> String {
    if available_fonts.contains(&font) {
        return font.clone();
    }

    "".to_string()
}

async fn process_request<'source>(
    State(state): State<SharedState<'source>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let available_fonts = vec![
        "Arial".to_string(),
        "Verdana".to_string(),
        "Menlo".to_string(),
    ];

    let mut document_template_renderer_env = state.template_rendering_env.clone();

    // This fails:
    //
    // let mut registered_fonts: HashSet<String> = HashSet::new();
    // document_template_renderer_env.add_function("register_font", move |font: String| {
    //     register_font(font, &available_fonts, &mut registered_fonts)
    // });
    //
    // This example is oversimplified, and it may seem that I could just call register_font() in Rust,
    // passing ctx.font to register_font_mut().
    //
    // There are some good reasons behind this:
    // * register_font() can be conditionally called within templates
    // * templates are dynamic and we don't know those conditions in order to reproduce those
    // * template rendering context data is also much more complex and dynamic, so we don't know the exact data as well

    // This expectedly works:
    document_template_renderer_env.add_function("register_font", move |font: String| {
        register_font(font, &available_fonts)
    });

    let template = "font-family: {{ register_font(font) }}".to_string();

    let ctx = json!({"font": "Arial"});

    let html_page = document_template_renderer_env
        .render_str(&template, &ctx)
        .unwrap();

    println!("{:#?}", html_page);

    Ok(Html(html_page))
}
