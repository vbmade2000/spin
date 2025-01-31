use anyhow::Result;
use spin_sdk::{
    http::{IntoResponse, Request},
    http_component,
};

/// Send an HTTP request and return the response.
#[http_component]
async fn send_outbound(_req: Request) -> Result<impl IntoResponse> {
    // Test self-request via relative URL
    let mut res: http::Response<String> = spin_sdk::http::send(
        http::Request::builder()
            .method("GET")
            .uri("/hello")
            .body(())?,
    )
    .await?;

    // Test self-request via self.alt
    let res_alt: http::Response<String> = spin_sdk::http::send(
        http::Request::builder()
            .method("GET")
            .uri("http://self.alt/hello")
            .body(())?,
    )
    .await?;

    assert_eq!(res.body(), res_alt.body());
    assert_eq!(res.status(), res_alt.status());

    res.headers_mut()
        .insert("spin-component", "outbound-http-component".try_into()?);

    Ok(res)
}
