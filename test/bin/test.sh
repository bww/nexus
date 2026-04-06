#!/usr/bin/env bash

set -eo pipefail

# where am i?
me="$0"
me_home=$(dirname "$0")
me_home=$(cd "$me_home" && pwd)

NEXUS=${NEXUS:-nexus}
NEXUS_ROLE=${NEXUS_ROLE:-tester}
NEXUS_PROJECT=${NEXUS_PROJECT}

# parse arguments
set -- $(getopt p:r: $*)
for i; do
  case "$i"
  in
    -p)
      NEXUS_PROJECT="$2"; shift;
      shift;;
    -r)
      NEXUS_ROLE="$2"; shift;
      shift;;
    --)
      shift; break;;
  esac
done

export NEXUS_PROJECT
export NEXUS_ROLE
export NEXUS_AGENT=$($NEXUS agent new --role $NEXUS_ROLE)

echo "agent: $NEXUS_AGENT"

$NEXUS help
