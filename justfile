base := "https://raw.githubusercontent.com/kubernetes/kubernetes"
dra  := "staging/src/k8s.io/kubelet/pkg/apis/dra"

latest := "1.34"

update-proto version=latest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir="v{{replace(version, '.', '_')}}"

    if [[ "{{version}}" == "1.34" ]]; then
        mkdir -p proto/${dir}/v1 proto/${dir}/v1beta1
        curl -sSL -o proto/${dir}/v1/api.proto      {{base}}/v{{version}}.0/{{dra}}/v1/api.proto
        curl -sSL -o proto/${dir}/v1beta1/api.proto {{base}}/v{{version}}.0/{{dra}}/v1beta1/api.proto

    else
        echo "Unknown version: {{version}}"
        exit 1
    fi
  
    # fetch gogo.proto (required by all Kubernetes protos)
    mkdir -p proto/vendor/github.com/gogo/protobuf/gogoproto
    curl -sSL -o proto/vendor/github.com/gogo/protobuf/gogoproto/gogo.proto \
        https://raw.githubusercontent.com/gogo/protobuf/master/gogoproto/gogo.proto

update-all-proto:
    just update-proto 1.34

bootstrap:
    just update-all-proto 
    cargo build

