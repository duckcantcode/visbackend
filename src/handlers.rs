use std::time::Duration;

use actix_web::{Error, HttpRequest, HttpResponse, get, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use tokio::time::{sleep, timeout};

use crate::json::Incoming;

#[get("/ws")]
pub async fn echo(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        let mut session_clone = session.clone();

        // spawn data output
        let data_output = rt::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
            }
        });

        // control
        async move {
            // receive messages from websocket
            loop {
                let timeout_result = timeout(Duration::from_secs(5), stream.next()).await;
                match timeout_result {
                    Ok(stream_result) => {
                        if let Some(msg) = stream_result {
                            match msg {
                                Ok(AggregatedMessage::Text(text)) => {
                                    let msg = serde_json::from_str::<Incoming>(&text);
                                    match msg {
                                        Ok(incoming) => match incoming._type.as_str() {
                                            "heartbeat" => (),
                                            "song" => {}

                                            _ => {
                                                log::warn!(
                                                    "Unknown incoming type {}",
                                                    incoming._type
                                                );
                                            }
                                        },
                                        Err(err) => {
                                            log::warn!(
                                                "Parsing Websocket message as JSON failed: {}",
                                                err
                                            );
                                        }
                                    }
                                }

                                Ok(AggregatedMessage::Ping(msg)) => {
                                    // respond to PING frame with PONG frame
                                    session.pong(&msg).await.unwrap();
                                }

                                _ => {}
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            // 60sec timeout occurred - term socket, we don't care if that was successful or not because it may be already closed
            let _ = session.close(None).await;
            data_output.abort();
        }
        .await
    });

    // respond immediately with response connected to WS session
    Ok(res)
}
