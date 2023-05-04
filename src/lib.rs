use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpclient::{HttpClient, HttpClientSender, HttpRequest};
use wasmcloud_interface_httpserver::{
    HttpRequest as ServerReq, HttpResponse, HttpServer, HttpServerReceiver,
};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct HelloTursoActor {}

const DB_URL: &str = "https://spin-psarna.turso.io";
const READONLY_TOKEN: &str = "Bearer eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhIjoicm8iLCJpYXQiOjE2ODMxODY4MDUsImlkIjoiNzIyY2IyYTEtY2M3MC0xMWVkLWFkM2MtOGVhNWEwNjcyYmM2In0.PD7ERvhUJwLAcsevYhutyrWj7L7gVEOaZYofiS-fftSyzqs5XQHz5leBSKkYDBGaqI0vy0Wl1cL_Whc8-mWlCQ";

fn get_row(responses: &serde_json::Value) -> Option<String> {
    let response = responses.as_array()?;
    let results = response[0].get("results")?;
    let rows = results.get("rows")?;
    rows.get(0).map(|r| r.to_string())
}

/// Implementation of the HttpServer capability contract
#[async_trait]
impl HttpServer for HelloTursoActor {
    async fn handle_request(&self, ctx: &Context, _req: &ServerReq) -> RpcResult<HttpResponse> {
        let request = HttpRequest {
            method: "POST".to_string(),
            url: DB_URL.to_string(),
            headers: std::collections::HashMap::from([(
                "Authorization".to_string(),
                vec![READONLY_TOKEN.to_string()],
            )]),
            body: br#"{"statements": ["SELECT 'hello, Turso!', count(*) from sqlite_master"]}"#
                .to_vec(),
        };
        let resp = HttpClientSender::new().request(ctx, &request).await?;
        let resp_json: serde_json::Value =
            serde_json::from_slice(&resp.body).map_err(|e| RpcError::HostError(e.to_string()))?;

        if let Some(err) = resp_json.get("error") {
            return Ok(HttpResponse::ok(format!("Error: {err}")));
        }

        let results = get_row(&resp_json).unwrap_or_else(|| "no results".to_string());
        Ok(HttpResponse::ok(results))
    }
}
