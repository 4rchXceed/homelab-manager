use crate::{
    context::AgentContext,
    net::{
        message::{FromServerMessage, FromServerMessageContent, ToServerMessageContent},
        send_queue::SendQueue,
    },
    services::{
        copy_sample::copy_sample_file, generate::generate_config_file, run_cmd::run_commands,
    },
};

pub async fn handle_command(
    command: FromServerMessage,
    context: AgentContext,
    response: SendQueue,
) {
    match command.content {
        FromServerMessageContent::Void => response.send(ToServerMessageContent::Void).await,
        FromServerMessageContent::RunCommands(commands, service) => {
            let result = run_commands(commands, service, false, context, None).await;

            match result {
                Ok(_) => {
                    response
                        .send(ToServerMessageContent::RunCommandsResponse(true, None))
                        .await
                }
                Err(e) => {
                    response
                        .send(ToServerMessageContent::RunCommandsResponse(
                            false,
                            Some(e.to_string()),
                        ))
                        .await
                }
            }
        }
        FromServerMessageContent::CopySample(service, file_name) => {
            let result = copy_sample_file(service, file_name, context).await;

            match result {
                Ok(_) => {
                    response
                        .send(ToServerMessageContent::CopySampleResponse(true, None))
                        .await
                }
                Err(e) => {
                    response
                        .send(ToServerMessageContent::CopySampleResponse(
                            false,
                            Some(e.to_string()),
                        ))
                        .await
                }
            }
        }
        FromServerMessageContent::GenerateConfig(commands, sandboxed, service, image) => {
            let result = generate_config_file(commands, sandboxed, service, image, context).await;

            match result {
                Ok(_) => {
                    response
                        .send(ToServerMessageContent::GenerateConfigResponse(true, None))
                        .await
                }
                Err(e) => {
                    response
                        .send(ToServerMessageContent::GenerateConfigResponse(
                            false,
                            Some(e.to_string()),
                        ))
                        .await
                }
            }
        }
    };
}
