#!/bin/sh

# SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
#
# SPDX-License-Identifier: Apache-2.0

cargo run -q --features serde_yaml,schemars --bin gen-config-schema > config-schema.yaml

if [ ! -z "$(git diff --name-only 'config-schema.yaml' 2>&1)" ]; then
	echo "config-schema.yaml has changed, try adding and committing it again"
	exit 1
fi
