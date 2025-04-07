#!/bin/bash
set -x
set -o allexport -o errexit -o privileged -o pipefail -o nounset
shopt -s extglob

# Configuration: Script
BASEDIR="$HOME/tmp/tote-poc/fs-singleton-zulu"
FS_PASSWORD="passw0rD?"

FS_TYPE_GOCRYPTFS="gocryptfs"
FS_TYPE_CRYFS="cryfs"
FS_TYPE_LUKS="luks"

JOURNAL_TYPE_GIT="git"
JOURNAL_TYPE_RSYNC_GIT="rsync-git"

function create_tree() {
    local tote_base_dir="$1"
    local tote_name="$2"
    local root_fs_type="$3"
    local password="$4"

    local root_fs_path="$tote_base_dir/tree/$tote_name/$tote_name.$root_fs_type"

    case "$root_fs_type" in
        gocryptfs)
            create_fs_gocryptfs "$root_fs_path" "$password"

            ;;
        *)
            echo "Unsupported filesystem type: $root_fs_type" >&2
            exit 1
    esac

    local mount_point_path="$tote_base_dir/mnt/fs/$tote_name"
    mkdir -p "$mount_point_path"
    mount_fs "$root_fs_path" "$mount_point_path" "$root_fs_type" "$password"
}

function create_fs() {
    local tote_base_dir="$1"
    local tote_name="$2"
    local fs_relative_path="$3"
    local fs_type="$4"
    local password="$5"

    local fs_path="$tote_base_dir/mnt/fs/$tote_name/$fs_relative_path.$fs_type"

    case "$fs_type" in
        gocryptfs)
            create_fs_gocryptfs "$fs_path" "$password"
            ;;
        *)
            echo "Unsupported filesystem type: $fs_type" >&2
            exit 1
    esac

    local mount_point_path="$tote_base_dir/mnt/jrn/$tote_name/$fs_relative_path.fs"
    mkdir -p "$mount_point_path"
    mount_fs "$fs_path" "$mount_point_path" "$fs_type" "$password"
}

function create_fs_gocryptfs() {
    local fs_path="$1"
    local password="$2"

    mkdir -p "$fs_path"
    echo "$password" | gocryptfs -init "$fs_path"
}

function mount_fs() {
    local fs_path="$1"
    local mount_point_path="$2"
    local fs_type="$3"
    local password="$4"

    case "$fs_type" in
        gocryptfs)
            mount_fs_gocryptfs "$fs_path" "$mount_point_path" "$password"
            ;;
        *)
            echo "Unsupported filesystem type: $fs_type" >&2
            exit 1
    esac
}

function mount_fs_gocryptfs() {
    local fs_path="$1"
    local mount_point_path="$2"
    local password="$3"

    echo "$password" | gocryptfs "$fs_path" "$mount_point_path"
}

function journal_path() {
    local tote_base_dir="$1"
    local tote_name="$2"
    local relative_fs_path="$3"
    local relative_journal_path="$4"
    echo "$tote_base_dir/mnt/jrn/$tote_name/$relative_fs_path.fs/$relative_journal_path"
}

function working_git_path() {
    local working_base_dir="$1"
    local tote_name="$2"
    local relative_fs_path="$3"
    local relative_journal_path="$4"
    echo "$working_base_dir/git/$tote_name/$relative_fs_path/$relative_journal_path"
}

function create_journal() {
    local tote_base_dir="$1"
    local tote_name="$2"
    local fs_relative_path="$3"
    local working_base_dir="$4"
    local journal_relative_path="$5"
    local journal_type="$6"

    case "$journal_type" in
        git)
            create_journal_git  "$tote_base_dir" "$tote_name" "$fs_relative_path" \
                                "$working_base_dir" "$journal_relative_path"
            ;;
        *)
            echo "Unsupported journal type: $journal_type" >&2
            exit 1
    esac

}

# Create a git repository in BASEDIR/upstream and a working clone of it in BASEDIR/working
# eg; upstream/totes/tree_root.category/git/node_a.node_b.git
# @param tree_root string
# @param category string
function create_journal_git() {
    local tote_base_dir="$1"
    local tote_name="$2"
    local fs_relative_path="$3"
    local working_base_dir="$4"
    local journal_relative_path="$5"

    # Create the upstream tote structure and git repository
	local repo_dir
	repo_dir="$(journal_path "$tote_base_dir" "$tote_name" "$fs_relative_path" "$journal_relative_path").git"
	mkdir -p "$repo_dir"
	git init --bare "$repo_dir"

    # Create a working clone of the repository for maintenance work
	local working_repo_dir
	working_repo_dir="$(working_git_path "$working_base_dir" "$tote_name" "$fs_relative_path" "$journal_relative_path")"
	mkdir -p "$working_repo_dir"
	cd "$working_repo_dir"

    # Initialize the Git repo from the working clone
    git init .
    git config submodule.recurse true  # user workflow: perform submodule recursion on all available commands by default
    git config push.recurseSubmodules on-demand  # user workflow: push submodules recursively as needed
    local touch_filename="${tote_name^^}.${fs_relative_path^^}.${journal_relative_path^^}.md"
    touch "${touch_filename//\//.}"  # creates a file like: ZULU.DOCUMENTS.DELTA.ECHO.md
    touch .gitignore
    git add .
    git commit -m "init: $tote_name // $fs_relative_path // $journal_relative_path"
    git remote add origin "$repo_dir"
    git push -u origin master  # sets the default upstream with -u
}

# Create submodules for each OU repo within the ORG repo
function create_git_submodule() {
    local tote_basedir="$1"
    local tote_name="$2"
    local working_dir="$3"
    local parent_fs_relative_path="$4"
    local submodule_fs_relative_path="$5"

    local parent_journal_name submodule_name
    parent_journal_name="$(basename "$parent_fs_relative_path")"
    submodule_name="$(basename "$submodule_fs_relative_path")"

    local parent_working_path="$(working_git_path "$working_dir" "$tote_name" "$parent_fs_relative_path" "_$parent_journal_name")"
    local submodule_git_path="$(journal_path "$tote_basedir" "$tote_name" "$submodule_fs_relative_path" "_$submodule_name.git")"

    local pwd="$PWD"

    cd "$parent_working_path"
    git config submodule.recurse true  # user workflow: perform submodule recursion on all available commands by default
    git config push.recurseSubmodules on-demand  # user workflow: push submodules recursively as needed
    git submodule add "$submodule_git_path" "$submodule_name"
    git commit -m"added submodule: ${submodule_name}"
    git push

    cd "$submodule_name"
    git config submodule.recurse true  # user workflow: perform submodule recursion on all available commands by default
    git config push.recurseSubmodules on-demand  # user workflow: push submodules recursively as needed
    cd ..

    cd "$pwd"
}

function update_git_submodules() {
    # Update all parent OU components with the changes, reverse recursively
    local submodule_dir="$PWD"
    local dir="$submodule_dir"
    while [ -e "$dir/.git" ]; do
        [ ! -z "$(git status -s)" ] && {
            git pull --rebase --recurse
            git submodule
            git commit -am 'submodule update'
            git push
        }
        cd ..
        dir="$PWD"
    done
    cd "$submodule_dir" # Return to the component OU directory
}

function main () {
    # shellcheck source=./zulu.cfg.bash
    source "$(dirname "$0")/zulu.cfg.bash"

    local tote_name="$TREE_ROOT"
    local basedir="$BASEDIR"
    local upstream_dir="$BASEDIR/upstream"
    local downstream_dir="$BASEDIR/downstream"
    local working_dir="$BASEDIR/working"
    local upstream_tote_basedir="$upstream_dir/tote"
    local downstream_tote_dir="$downstream_dir/tote"


    mkdir -p "$basedir" "$upstream_dir" "$downstream_dir" "$working_dir"
    cd "$basedir"

    create_tree "$upstream_tote_basedir" "$tote_name" "$FS_TYPE_GOCRYPTFS" "$FS_PASSWORD"

    for relative_fs_path in "${TREE_FS_NODES[@]}"; do
        create_fs "$upstream_tote_basedir" "$tote_name" "$relative_fs_path" "$FS_TYPE_GOCRYPTFS" "$FS_PASSWORD"

        for relative_journal_path in ${TREE_JOURNAL_NODES["$relative_fs_path"]}; do
            create_journal "$upstream_tote_basedir" "$tote_name" "$relative_fs_path" \
                            "$working_dir" "$relative_journal_path" "$JOURNAL_TYPE_GIT"
        done
    done

    for super_relative_fs_path in "${TREE_FS_NODES[@]}"; do
        fs_has_cwd_journal "$super_relative_fs_path" ||
            continue

        # search for sub-filesystems
        for sub_relative_fs_path in "${TREE_FS_NODES[@]}"; do
            fs_has_cwd_journal "$super_relative_fs_path" ||  # submodule fs must have a cwd journal
                continue
            [[ "$(dirname "$sub_relative_fs_path")" == "$super_relative_fs_path" ]]  || # basepath must be super's path
               continue

            create_git_submodule "$upstream_tote_basedir" "$tote_name" "$working_dir" \
                                        "$super_relative_fs_path" "$sub_relative_fs_path"
        done
    done

    for super_relative_fs_path in "${TREE_FS_NODES[@]}"; do
        fs_has_cwd_journal "$super_relative_fs_path" ||
            continue

        local super_working_path
        super_working_path="$(working_git_path "$working_dir" "$tote_name" "$super_relative_fs_path" "_$(basename "$super_relative_fs_path")")"

        cd "$super_working_path"
        git checkout master
        git pull --recurse --rebase
        git submodule update --init --recursive

        # search for sub-filesystems
        for sub_relative_fs_path in "${TREE_FS_NODES[@]}"; do
            fs_has_cwd_journal "$super_relative_fs_path" ||  # submodule fs must have a cwd journal
                continue
            [[ "$(dirname "$sub_relative_fs_path")" == "$super_relative_fs_path" ]]  || # basepath must be super's path
               continue

            local sub_working_path
            sub_working_path="$super_working_path/$(basename "$sub_relative_fs_path")"
            cd "$sub_working_path"
            git checkout master
            git pull --recurse --rebase
            git submodule update --init --recursive
            git submodule foreach --recursive 'git checkout master'
        done

        cd "$super_working_path"
        git commit -am'submodule push'
        git push
    done

}

function fs_has_cwd_journal() {
        local relative_fs_path="$1"
        local journal_filename
        journal_filename="_$(basename "$relative_fs_path")"

        # check the journal paths to see if it has a '.' repository
        for relative_journal_path in ${TREE_JOURNAL_NODES["$relative_fs_path"]}; do
            if [[ "$relative_journal_path" == "$journal_filename" ]]; then
                return 0
            fi
        done

        return 1
}

main
exit 0
