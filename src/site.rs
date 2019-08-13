/// This module contains the server that will start for the Alexa web service.   
///
/// All requests will be verified using `alexa_verifier` then processed and
/// responded to using the `crate::skill` module.
use crate::skill::process_request;
use alexa_verifier::RequestVerifier;
use log::{debug, error, info};
use rouille::{router, Request, Response};
use std::{
    io::Read,
    sync::{Mutex, MutexGuard},
};

/// Only one route is needed to accept json POST request from Alexa.   
///
/// All other routes will return 404
fn note_routes(request: &Request, verifier: &mut MutexGuard<RequestVerifier>) -> Response {
    router!(request,
        (POST) (/) => {
            info!("Request received...");

            // Get raw body from request
            let mut body = request.data().unwrap();
            let mut body_bytes: Vec<u8> = vec![];
            body.read_to_end(&mut body_bytes).unwrap();

            // Extract headers needed for request verification
            let signature_cert_chain_url = request.header("SignatureCertChainUrl").unwrap_or("");
            let signature = request.header("Signature").unwrap_or("");

            // Deserialize the request, returning 400 on de error
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

            // Verify the request came from Alexa, 400 if not
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

            // Process and get response from `crate::skill` module, then
            // serialize
            let response = Response::json(&process_request(request));

            // Send back response
            info!("Sending back response...");
            debug!("{:?}", response);
            response
    },
        _ => Response::empty_404()
    )
}

/// Use the specified port to run the web service.   
///
/// `alexa_verifier::RequestVerifier` needs to be mutexed for safe acces, as
/// it contains a certificate cache.
pub fn run(port: &str) -> std::io::Result<()> {
    let verifier = Mutex::from(RequestVerifier::new());

    let addrs = format!("0.0.0.0:{}", port);
    info!("Starting server on {}", addrs);
    rouille::start_server(addrs, move |request| {
        let mut verifier = verifier.lock().unwrap();
        note_routes(&request, &mut verifier)
    });
}
