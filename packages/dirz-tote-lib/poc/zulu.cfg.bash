#!/bin/bash
set -o allexport -o errexit -o privileged -o pipefail -o nounset
shopt -s extglob

# Configuration: User
TREE_ROOT="zulu"

TREE_FS_NODES=( \
  documents \
  documents/alpha \
  documents/bravo \
  documents/charlie \
  documents/charlie/delta \
  documents/charlie/echo \
#  data \
#  keys \
#  multimedia \
  projects/charlie \
)

declare -A TREE_JOURNAL_NODES
TREE_JOURNAL_NODES['documents']="_documents"
TREE_JOURNAL_NODES['documents/alpha']="_alpha"
TREE_JOURNAL_NODES['documents/bravo']="_bravo"
TREE_JOURNAL_NODES['documents/charlie']="_charlie"
TREE_JOURNAL_NODES['documents/charlie/delta']="_delta"
TREE_JOURNAL_NODES['documents/charlie/echo']="_echo"
TREE_JOURNAL_NODES['projects/charlie']="project-one project-two delta/project-three echo/project-four"

ROLES=( \
  alice \
  anthony \
  benjamin \
  beth \
  cindy \
  david \
  daniel \
  ellen \
  eric \
)
