use anyhow::anyhow;
use helper::{ensure, ensure_eq, ensure_matches, ensure_ok, ensure_some};
use spin_sdk::{
    http::{IntoResponse, Request},
    http_component,
};

/// Send an HTTP request and return the response.
#[http_component]
async fn handle_front(req: Request) -> anyhow::Result<impl IntoResponse> {
    handle_front_impl(req).await.map_err(|e| anyhow!(e))
}

async fn handle_front_impl(_req: Request) -> Result<impl IntoResponse, String> {
    let mut res: http::Response<String> = ensure_ok!(spin_sdk::http::send(
        spin_sdk::http::Request::new(
            spin_sdk::http::Method::Get,
            "http://middle.spin.internal/hello/from/front"
        )
    )
    .await);

    let component_header = ensure_some!(res.headers().get("spin-component"));
    let component_header = String::from_utf8_lossy(component_header.as_bytes());
    ensure!(component_header.contains("internal-http-middle-component"));

    ensure_eq!("/hello/from/front", ensure_some!(res.headers().get("middle-received-path")));
    ensure_eq!("/hello/from/middle", ensure_some!(res.headers().get("back-received-path")));
    ensure_eq!("GET", ensure_some!(res.headers().get("middle-received-method")));
    ensure_eq!("POST", ensure_some!(res.headers().get("back-received-method")));
    ensure_eq!("body from middle", ensure_some!(res.headers().get("back-received-body")));

    res.headers_mut()
        .append("spin-component", ensure_ok!("internal-http-front-component".try_into()));

    // Double-check that we can only send requests to allowed hosts.
    ensure_matches!(
        spin_sdk::http::send::<_, spin_sdk::http::Response>(spin_sdk::http::Request::new(
            spin_sdk::http::Method::Get,
            "http://back.spin.internal/hello/from/middle"
        ))
        .await,
        Err(spin_sdk::http::SendError::Http(
            spin_sdk::http::Error::UnexpectedError(_)
        )),
    );
    Ok(res)
}
