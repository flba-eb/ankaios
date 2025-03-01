// Copyright (c) 2023 Elektrobit Automotive GmbH
//
// This program and the accompanying materials are made available under the
// terms of the Apache License, Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0

#[cfg_attr(test, mockall_double::double)]
use super::ReopenFile;
use api::proto;
use common::{
    execution_interface::{ExecutionCommand, ExecutionReceiver},
    state_change_interface::{StateChangeCommand, StateChangeSender},
};

use prost::Message;
use tokio::{io, select, task::JoinHandle};

fn decode_state_change_request(
    protobuf_data: io::Result<Box<[u8]>>,
) -> io::Result<proto::StateChangeRequest> {
    Ok(proto::StateChangeRequest::decode(&mut Box::new(
        protobuf_data?.as_ref(),
    ))?)
}

pub struct PipesChannelTask {
    output_stream: ReopenFile,
    input_stream: ReopenFile,
    input_pipe_receiver: ExecutionReceiver,
    output_pipe_channel: StateChangeSender,
    request_id_prefix: String,
}

#[cfg_attr(test, mockall::automock)]
impl PipesChannelTask {
    pub fn new(
        output_stream: ReopenFile,
        input_stream: ReopenFile,
        input_pipe_receiver: ExecutionReceiver,
        output_pipe_channel: StateChangeSender,
        request_id_prefix: String,
    ) -> Self {
        Self {
            output_stream,
            input_stream,
            input_pipe_receiver,
            output_pipe_channel,
            request_id_prefix,
        }
    }
    pub async fn run(mut self) {
        loop {
            select! {
                // [impl->swdd~agent-ensures-control-interface-output-pipe-read~1]
                execution_command = self.input_pipe_receiver.recv() => {
                    if let Some(execution_command) = execution_command {
                        let _ = self.forward_execution_command(execution_command).await;
                    }
                }
                // [impl->swdd~agent-listens-for-requests-from-pipe~1]
                // [impl->swdd~agent-forward-request-from-control-interface-pipe-to-server~1]
                state_change_request_binary = self.input_stream.read_protobuf_data() => {
                    if let Ok(state_change_request) = decode_state_change_request(state_change_request_binary) {
                        match state_change_request.try_into() {
                            Ok(StateChangeCommand::RequestCompleteState(mut request_complete_state)) => {
                                request_complete_state.prefix_request_id(&self.request_id_prefix);
                                let _ = self.output_pipe_channel.send(StateChangeCommand::RequestCompleteState(request_complete_state)).await;
                            }
                            Ok(state_change_command) => {
                                let _ = self.output_pipe_channel.send(state_change_command).await;
                            }
                            Err(error) => {
                                log::warn!("Could not convert protobuf in internal data structure: {}", error)
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn run_task(self) -> JoinHandle<()> {
        tokio::spawn(self.run())
    }

    async fn forward_execution_command(&mut self, command: ExecutionCommand) -> io::Result<()> {
        if let Ok(proto) = proto::ExecutionRequest::try_from(command) {
            // [impl->swdd~agent-uses-length-delimited-protobuf-for-pipes~1]
            let binary = proto.encode_length_delimited_to_vec();
            self.output_stream.write_all(&binary).await?;
        }
        Ok(())
    }
}

//////////////////////////////////////////////////////////////////////////////
//                 ########  #######    #########  #########                //
//                    ##     ##        ##             ##                    //
//                    ##     #####     #########      ##                    //
//                    ##     ##                ##     ##                    //
//                    ##     #######   #########      ##                    //
//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
pub fn generate_test_pipes_channel_task_mock() -> __mock_MockPipesChannelTask::__new::Context {
    let pipes_channel_task_mock_context = MockPipesChannelTask::new_context();
    pipes_channel_task_mock_context
        .expect()
        .return_once(|_, _, _, _, _| {
            let mut pipes_channel_task_mock = MockPipesChannelTask::default();
            pipes_channel_task_mock
                .expect_run_task()
                .return_once(|| tokio::spawn(async {}));
            pipes_channel_task_mock
        });
    pipes_channel_task_mock_context
}

#[cfg(test)]
mod tests {
    use mockall::predicate;
    use tokio::sync::mpsc;

    use super::*;

    use crate::control_interface::MockReopenFile;

    #[tokio::test]
    async fn utest_pipes_channel_task_forward_execution_command() {
        let _guard = crate::test_helper::MOCKALL_CONTEXT_SYNC
            .get_lock_async()
            .await;

        let test_command =
            ExecutionCommand::CompleteState(Box::new(common::commands::CompleteState {
                request_id: "req_id".to_owned(),
                ..Default::default()
            }));

        let test_command_binary = proto::ExecutionRequest::try_from(test_command.clone())
            .unwrap()
            .encode_length_delimited_to_vec();

        // [utest->swdd~agent-uses-length-delimited-protobuf-for-pipes~1]
        let mut output_stream_mock = MockReopenFile::default();
        output_stream_mock
            .expect_write_all()
            .with(predicate::eq(test_command_binary))
            .return_once(|_| Ok(()));

        let input_stream_mock = MockReopenFile::default();
        let (_, input_pipe_receiver) = mpsc::channel(1);
        let (output_pipe_sender, _) = mpsc::channel(1);
        let request_id_prefix = String::from("prefix@");

        let mut pipes_channel_task = PipesChannelTask::new(
            output_stream_mock,
            input_stream_mock,
            input_pipe_receiver,
            output_pipe_sender,
            request_id_prefix,
        );

        assert!(pipes_channel_task
            .forward_execution_command(test_command)
            .await
            .is_ok());
    }

    // [utest->swdd~agent-listens-for-requests-from-pipe~1]
    // [utest->swdd~agent-forward-request-from-control-interface-pipe-to-server~1]
    // [utest->swdd~agent-ensures-control-interface-output-pipe-read~1]
    #[tokio::test]
    async fn utest_pipes_channel_task_run_task() {
        let _guard = crate::test_helper::MOCKALL_CONTEXT_SYNC
            .get_lock_async()
            .await;

        let test_output_request = proto::StateChangeRequest {
            state_change_request_enum: Some(
                proto::state_change_request::StateChangeRequestEnum::RequestCompleteState(
                    proto::RequestCompleteState {
                        request_id: "req_id".to_owned(),
                        field_mask: vec![],
                    },
                ),
            ),
        };

        let test_output_request_binary = test_output_request.encode_to_vec();

        let mut input_stream_mock = MockReopenFile::default();
        let mut x = [0; 10];
        x.clone_from_slice(&test_output_request_binary[..]);
        input_stream_mock
            .expect_read_protobuf_data()
            .returning(move || Ok(Box::new(x)));

        let test_input_command =
            ExecutionCommand::CompleteState(Box::new(common::commands::CompleteState {
                request_id: "req_id".to_owned(),
                ..Default::default()
            }));

        let test_input_command_binary =
            proto::ExecutionRequest::try_from(test_input_command.clone())
                .unwrap()
                .encode_length_delimited_to_vec();

        let mut output_stream_mock = MockReopenFile::default();
        output_stream_mock
            .expect_write_all()
            .with(predicate::eq(test_input_command_binary.clone()))
            .return_once(|_| Ok(()));

        let (input_pipe_sender, input_pipe_receiver) = mpsc::channel(1);
        let (output_pipe_sender, mut output_pipe_receiver) = mpsc::channel(1);
        let request_id_prefix = String::from("prefix@");

        let pipes_channel_task = PipesChannelTask::new(
            output_stream_mock,
            input_stream_mock,
            input_pipe_receiver,
            output_pipe_sender,
            request_id_prefix,
        );

        let handle = pipes_channel_task.run_task();

        assert!(input_pipe_sender.send(test_input_command).await.is_ok());
        assert_eq!(
            Some(StateChangeCommand::RequestCompleteState(
                common::commands::RequestCompleteState {
                    request_id: "prefix@req_id".to_owned(),
                    field_mask: vec![],
                }
            )),
            output_pipe_receiver.recv().await
        );

        handle.abort();
    }
}
