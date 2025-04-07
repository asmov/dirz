#!/bin/bash
set -o allexport -o errexit -o privileged -o pipefail -o nounset
shopt -s extglob

# usage:
# tote <action> <media> <tote name>
# e.g.,
#   tote open mydisk1 mytote
function main () {
	local action="$1"
	local params="${2-}"
	local dirpath="$PWD"

	case "$action" in
		open)
			mount_tote "$dirpath" "$params"
			;;
		close)
			umount_tote "$dirpath" "$params"
			;;
		backup) backup
			;;
		restore) restore
			;;
		*)
			echo "usage: $0 {mount <tote name> | umount <tote_name> | backup | restore}"
			exit 1 
	esac
}

function datestamp () {
	date "+%Y-%m-%d-%H%M-%S"
}

function setup_tmpwork_dir () {
	local homedir tmpdir
	homedir="$(realpath "$HOME")"

	# use the system tmp dir if this user doesn't have a home
	if [ -d "$homedir" ]; then 
		tmpdir="$homedir/tmp"
	else
		tmpdir="/tmp/${USER}"	
	fi
	
	[ -d "$tmpdir" ] || {
		mkdir "$tmpdir"
		chmod 700 "$tmpdir"
	}

	local tmpworkdir="${tmpdir}/tote_${DATESTAMP}"

	[ -d "$tmpworkdir" ] || {
		mkdir "$tmpworkdir"
		chmod 700 "$tmpworkdir"
	}

	cd "$tmpworkdir"
}

function tmpwork_dir () {
	local homedir tmpdir
	homedir="$(realpath "$HOME")"

	# use the system tmp dir if this user doesn't have a home
	if [ -d "$homedir" ]; then 
		tmpdir="$homedir/tmp"
	else
		tmpdir="/tmp/${USER}"	
	fi

	echo "$tmpdir/tote_${DATESTAMP}"
}


# Zips each directory structure configured in .config/tote/backup_directories
# Creates a GPG encrypted XZ zip containing them all.
function backup () {
	setup_tmpwork_dir

	local tmpworkdir="$(tmpwork_dir)"
	local tmpworkdir_basename="$(basename "$tmpworkdir")"

	mkdir "$tmpworkdir_basename"
	cd "$tmpworkdir_basename"
	chmod 700 .

	mapfile -t dirs < $HOME/.config/tote/backup_directories

	for dir in "${dirs[@]}"; do
		echo "backing up directory: ${dir} ..."

		if [ -d "$dir" ]; then 
			backup_dir "$dir"
		else
			echo "warning: Directory doesn't exist; skipped: ${dir}"
		fi
	done

	cd ..

	local datestamp="$DATESTAMP"
	local digest_filename="tote_backup_${datestamp}.tar.xz"

	tar cJf "$digest_filename" -C "${tmpworkdir}" "$tmpworkdir_basename"
	chmod 400 "$digest_filename"

	encrypt_backup "$digest_filename"
	rm -f "$digest_filename"

	rm -rf "$tmpworkdir_basename"

	echo "backup complete"

	which xdg-open >/dev/null && xdg-open . 
}

function backup_dir () {
	local source_dir datestamp source_basedir source_dirname backup_name backup_zip_filename
	source_dir="$1"
	datestamp="$(datestamp)"
	source_basedir="$(realpath "${source_dir}/..")"
	source_dirname="$(basename "$source_dir")"
	backup_name="$(echo "${dir}" | sed -e's/\//_/g' -e's/^\_//')"
	backup_zip_filename="${backup_name}.tar.xz"

	tar cJf "$backup_zip_filename" -C "$source_basedir" "$source_dirname"
	chmod 400 "$backup_zip_filename"

	echo "$source_dir" > "${backup_name}.dir"
	chmod 400 "${backup_name}.dir"
}

function encrypt_backup () {
	local backup_filename="${1}"
	local encrypted_backup_filename="${1}.gpg"

	gpg -c "$backup_filename"
	chmod 400 "$encrypted_backup_filename"
}

function mount_tote () {
	local dirpath tote_name tote_encfs_path tote_mount_path
	dirpath="$1"
	tote_name="${2}"
	local tote_encfs_path="${dirpath}/.${tote_name}.encfs"
	local tote_mount_path="$dirpath/${tote_name}"

	encfs "$tote_encfs_path" "$tote_mount_path"
}

function umount_tote () {
	local dirpath tote_name tote_mount_path
	dirpath="$1"
	tote_name="$2"
	local tote_mount_path="$dirpath/${tote_name}"

	umount "$tote_mount_path"
}

function load_config_media () {

}

DATESTAMP="$(datestamp)"

main "$1" "${2-}"
exit 0
