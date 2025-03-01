#!/bin/bash

# Copyright (c) 2023 Elektrobit Automotive GmbH
#
# This program and the accompanying materials are made available under the
# terms of the Apache License, Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0.
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations
# under the License.
#
# SPDX-License-Identifier: Apache-2.0

set -e

script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
base_dir="$script_dir/.."
target_dir="$base_dir/build/doc"
mkdir -p "$base_dir/build/"
rm -rf "$target_dir"
echo "Generate Markdown from ./api/proto/ankaios.proto ..."
cp "$base_dir/doc/" "$target_dir" -ru
protoc --plugin=protoc-gen-doc="/usr/local/bin/protoc-gen-doc" --doc_out="$target_dir/docs/reference" --doc_opt=markdown,_ankaios.proto.md --proto_path="$base_dir/api/proto" ankaios.proto
echo "Generate Markdown from ./api/proto/ankaios.proto done."

[[ ${1,,} = serve ]] && mkdocs serve -f "$target_dir/mkdocs.yml"
[[ ${1,,} = build ]] && mkdocs build -f "$target_dir/mkdocs.yml" -d html && exit 0
[[ ${1,,} = deploy ]] && mkdocs gh-deploy -f "$target_dir/mkdocs.yml" --force && exit 0
