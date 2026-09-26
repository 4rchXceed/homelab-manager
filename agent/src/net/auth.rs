use log::trace;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_rustls::client::TlsStream;

use crate::{
    client::Client,
    config::Config,
    db::database::Database,
    errors::{AuthError, ClientRuntimeError, NetRunError},
};

pub async fn handle_server_auth(
    socket: &mut TlsStream<TcpStream>,
    config: &Config,
    db: &mut Database,
) -> Result<(), ClientRuntimeError> {
    trace!("Starting auth protocol");

    let mut id = config.id.clone();
    id.push_str("\n");

    trace!("[OK] Sending id: {id}");

    socket
        .write(id.as_bytes())
        .await
        .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

    if let ack = Client::recv(socket, 4).await?
        && ack != b"AUTH"
    {
        let str = String::from_utf8_lossy(ack.as_slice());

        trace!("[ERROR] AUTH ACK NOK! Got: {str}");

        return Err(ClientRuntimeError::auth(AuthError::AuthAckNotSent));
    }

    trace!("[OK] AUTH ACK");

    let api_key_newline = config.api_key.clone() + "\n";

    socket
        .write(api_key_newline.as_bytes())
        .await
        .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

    let auth_ack = Client::recv(socket, 2).await?;

    if auth_ack != b"OK" {
        let str = String::from_utf8_lossy(&auth_ack);

        trace!("[ERROR]: API KEY NOT OK! Got: {str}");

        return Err(ClientRuntimeError::auth(AuthError::InvalidApiKey));
    }

    trace!("[OK] API KEY OK");

    let reverse_api_key = Client::recv(socket, 36).await?;

    let reverse_api_key_str = String::from_utf8(reverse_api_key)
        .map_err(|_| ClientRuntimeError::auth(AuthError::ReverseApiKeyIsntUtf8))?;

    if let Some(correct_reverse_api_key) = &db.reverse_api_key {
        trace!("[OK] API KEY FOUND (for security reasons, the api key isn't shown)");
        if &reverse_api_key_str != correct_reverse_api_key {
            trace!("[ERROR] COULD NOT AUTH SERVER");

            socket
                .write(b"ER")
                .await
                .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

            return Err(ClientRuntimeError::auth(AuthError::InvalidReverseApiKey));
        }
    } else {
        trace!("First time receiving api key! Adding it to the db");

        db.reverse_api_key = Some(reverse_api_key_str);

        trace!("Saving the database");

        db.save(config.database_path.clone())
            .await
            .map_err(|e| ClientRuntimeError::auth(AuthError::NewReverseApiKeyDbSave(e)))?;
    }

    socket
        .write(b"OK")
        .await
        .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

    trace!("Finished auth protocol");

    return Ok(());
}
