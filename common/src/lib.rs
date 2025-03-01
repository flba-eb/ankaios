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

pub const CHANNEL_CAPACITY: usize = 20;
pub const DEFAULT_SOCKET_ADDRESS: &str = "127.0.0.1:50051";
pub const DEFAULT_SERVER_ADDRESS: &str = "http://127.0.0.1:50051";

pub mod commands;
pub mod communications_client;
pub mod communications_server;
pub mod execution_interface;
pub mod helpers;
pub mod objects;
pub mod request_id_prepending;
pub mod state_change_interface;
#[cfg(feature = "test_utils")]
pub mod test_utils;
