#!/usr/bin/env -S nix shell nixpkgs#nomad nixpkgs#jq --command bash

export NOMAD_VAR_rev=$(nix flake metadata . --json | jq -r '.locked.rev')
export NOMAD_VAR_narHash=$(nix flake metadata . --json | jq -r '.locked.narHash')

exec nomad $@
