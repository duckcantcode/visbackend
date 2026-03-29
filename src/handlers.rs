use std::{path::Path, time::Duration};

use actix_web::{Error, HttpRequest, HttpResponse, get, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use tokio::time::{sleep, timeout};

use crate::{
    AppState, backend,
    json::{Incoming, Outgoing, OutgoingSongInfo},
};

#[get("/ws")]
pub async fn echo(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        let mut session_clone = session.clone();
        let state_clone = state.clone();

        // spawn data output
        let data_output = rt::spawn(async move {
            let mut last_path = "".to_string();
            // TODO: replace above with Option<>
            loop {
                let song_handle = state_clone.current_song.read().await;
                let song = song_handle.clone();
                drop(song_handle);
                match song {
                    None => (),
                    Some(song) => {
                        if last_path != song.song_path {
                            last_path = song.song_path;
                            let serialized = serde_json::to_string(&Outgoing {
                                _type: "song".to_string(),
                                song_info: Some(OutgoingSongInfo {
                                    fft: song.fft,
                                    period: song.period,
                                }),
                            })
                            .unwrap();

                        // only send on update
                        let _timeout_result = timeout(
                            Duration::from_millis(500),
                            session_clone.text(serialized.clone()),
                        )
                        .await;
                        // TODO: close session if timeout occurs
                        }
                    }
                }
                sleep(Duration::from_secs(1)).await;
            }
        });

        // control
        async move {
            // receive messages from websocket
            loop {
                // terminate socket if 5 seconds with no message
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
                                            "song" => {
                                                if let Some(song_path) = incoming.song_path {
                                                    // verify path exists
                                                    if let Ok(parsed_path) =
                                                        Path::new(&song_path).canonicalize()
                                                    {
                                                        let data = backend::conv(&parsed_path);
                                                        let mut song_handle =
                                                            state.current_song.write().await;
                                                        *song_handle = Some(data);
                                                        drop(song_handle);
                                                    }
                                                }
                                            }
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

            // timeout occurred - term socket, we don't care if that was successful or not because it may be already closed
            let _ = session.close(None).await;
            data_output.abort();
        }
        .await
    });

    // respond immediately with response connected to WS session
    Ok(res)
}
