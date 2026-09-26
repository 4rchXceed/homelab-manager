use crate::{
    context::AgentContext,
    net::{
        message::{FromServerMessage, FromServerMessageContent, ToServerMessageContent},
        send_queue::SendQueue,
    },
};

pub async fn handle_command(
    command: FromServerMessage,
    context: AgentContext,
    response: SendQueue,
) {
    match command.content {
        FromServerMessageContent::Void => response.send(ToServerMessageContent::Void).await,
    };
}
