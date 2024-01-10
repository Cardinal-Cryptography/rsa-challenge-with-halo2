#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

export NODE_IMAGE="public.ecr.aws/p6e8q1z1/aleph-node-liminal:f8de357"

ADMIN=//Alice
ADMIN_PUBKEY=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY

# env
NODE="ws://127.0.0.1:9944"
DOCKER_USER="$(id -u):$(id -g)"
export DOCKER_USER

# command aliases
DOCKER_SH="docker run --rm -e RUST_LOG=debug -u ${DOCKER_USER} --entrypoint /bin/sh"

get_timestamp() {
  date +'%Y-%m-%d %H:%M:%S'
}

log_progress() {
  bold=$(tput bold)
  normal=$(tput sgr0)
  echo "[$(get_timestamp)] [INFO] ${bold}${1}${normal}"
}

prepare_fs() {
  # ensure that we are in the main repo directory
  cd "${SCRIPT_DIR}"

  # forget everything from the past launches - start the chain from a scratch
  rm -rf docker/node_data/

  # ensure that all these folders are present
  mkdir -p docker/node_data/
  mkdir -p docker/keys/

  log_progress "✅ Directories are set up"
}

generate_chainspec() {
  CHAINSPEC_ARGS="\
    --base-path /data \
    --account-ids ${ADMIN_PUBKEY} \
    --sudo-account-id ${ADMIN_PUBKEY} \
    --faucet-account-id ${ADMIN_PUBKEY} \
    --chain-id a0zknet \
    --token-symbol ZKZERO \
    --chain-name 'Aleph Zero ZK Playground'"

  $DOCKER_SH \
    -v "${SCRIPT_DIR}/docker/node_data:/data" \
    "${NODE_IMAGE}" \
    -c "aleph-node bootstrap-chain ${CHAINSPEC_ARGS} > /data/chainspec.aleph.json"

  log_progress "✅ Generated chainspec was written to docker/data/chainspec.aleph.json"
}

export_bootnode_address() {
  BOOTNODE_PEER_ID=$(
    $DOCKER_SH \
      -v "${SCRIPT_DIR}/docker/node_data:/data" \
      "${NODE_IMAGE}" \
      -c "aleph-node key inspect-node-key --file /data/${ADMIN_PUBKEY}/p2p_secret"
  )
  export BOOTNODE_PEER_ID
  log_progress "✅ Exported bootnode address (${BOOTNODE_PEER_ID})"
}

run_snarkeling_node() {
  NODE_PUBKEY=$ADMIN_PUBKEY docker-compose -f docker-compose.yml up --remove-orphans -d
  log_progress "✅ Successfully launched snarkeling node"
}

launch_chain() {
  # general setup
  prepare_fs

  # launching node
  generate_chainspec
  export_bootnode_address
  run_snarkeling_node
}

launch_chain
