use crate::skill::process_request;
use alexa_verifier::RequestVerifier;
use log::{debug, error, info};
use rouille::{router, Request, Response};
use std::{
    io::Read,
    sync::{Mutex, MutexGuard},
};

fn note_routes(request: &Request, verifier: &mut MutexGuard<RequestVerifier>) -> Response {
    router!(request,
        (POST) (/) => {
            info!("Request received...");

            let mut body = request.data().unwrap();
            let mut body_bytes: Vec<u8> = vec![];
            body.read_to_end(&mut body_bytes).unwrap();

            let signature_cert_chain_url = request.header("SignatureCertChainUrl").unwrap_or("");
            let signature = request.header("Signature").unwrap_or("");

            let _request = serde_json::from_slice::<alexa_sdk::Request>(&body_bytes);
            if let Err(e) = _request {
                error!("Could not deserialize request");
                error!("{:?}", e);
                let response = Response::empty_400();
                info!("Sending back response...");
                debug!("{:?}", response);
                return response;
            }
            let request = _request.unwrap();
            debug!("{:?}", request);

            if verifier
                .verify(
                    signature_cert_chain_url,
                    signature,
                    &body_bytes,
                    request.body.timestamp.as_str(),
                    None
                ).is_err() {
                    error!("Could not validate request came from Alexa");
                    let response = Response::empty_400();
                    info!("Sending back response...");
                    debug!("{:?}", response);
                    return response;
                };
            debug!("Request is validated...");

            let response = Response::json(&process_request(request));
            info!("Sending back response...");
            debug!("{:?}", response);
            response
    },
        _ => Response::empty_404()
    )
}

pub fn run() -> std::io::Result<()> {
    info!("Starting server on 0.0.0.0:8086");
    let verifier = Mutex::from(RequestVerifier::new());

    rouille::start_server("0.0.0.0:8086", move |request| {
        let mut verifier = verifier.lock().unwrap();
        note_routes(&request, &mut verifier)
    });
}
