#!/usr/bin/env bash

set -eo pipefail

# where am i? (define this in the test suite)
# me="$0"
# me_home=$(dirname "$0")
# me_home=$(cd "$me_home" && pwd)

# import assert utils
. "$me_home/_assert.sh"

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

export NEXUS_ROLE
export NEXUS_AGENT=$($NEXUS agent new --role $NEXUS_ROLE)

another_agent=$($NEXUS agent new --role another)

project_dir="$(mktemp -d)"
trap "rm -r ${project_dir}" EXIT
export NEXUS_PROJECT="${project_dir}"

echo "project: $NEXUS_PROJECT"
echo "agent: $NEXUS_AGENT"
