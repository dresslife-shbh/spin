use crate::ensure_eq;
use anyhow::Result;
use reqwest::{Method, Request, Response};
use std::str;

pub async fn assert_status(url: &str, expected: u16) -> Result<()> {
    let resp = make_request(Method::GET, url, "").await?;
    let status = resp.status();

    let body = resp.bytes().await?;
    let actual_body = str::from_utf8(&body).unwrap().to_string();

    ensure_eq!(status, expected, "{}", actual_body);

    Ok(())
}

pub async fn assert_http_response(
    url: &str,
    method: Method,
    body: &str,
    expected: u16,
    expected_headers: &[(&str, &str)],
    expected_body: Option<&str>,
) -> Result<()> {
    let res = make_request(method, url, body).await?;

    let status = res.status();
    let headers = res.headers().clone();
    let body = res.bytes().await?;
    let actual_body = str::from_utf8(&body).unwrap().to_string();

    ensure_eq!(
        expected,
        status.as_u16(),
        "Expected status {expected} but got {status}. Response body: '{actual_body}'"
    );

    for (k, v) in expected_headers {
        ensure_eq!(
            &headers
                .get(k.to_string())
                .unwrap_or_else(|| panic!("cannot find header {}", k))
                .to_str()?,
            v
        )
    }

    if let Some(expected_body_str) = expected_body {
        ensure_eq!(expected_body_str, actual_body);
    }

    Ok(())
}

pub async fn create_request(method: Method, url: &str, body: &str) -> Result<Request> {
    let mut req = reqwest::Request::new(method, url.try_into()?);
    *req.body_mut() = Some(body.to_owned().into());

    Ok(req)
}

pub async fn make_request(method: Method, path: &str, body: &str) -> Result<Response> {
    let req = create_request(method, path, body).await?;
    let client = reqwest::Client::new();
    Ok(client.execute(req).await?)
}
