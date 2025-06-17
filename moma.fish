#!/usr/bin/env fish

function get_latest_skse_url
    curl -s https://skse.silverlock.org/ | string match -r 'href="(.*skse64_.*?\.7z)"' |
        string replace -r 'href="' '' |
        string replace -r '"' '' |
        string match -v gog |
        head -n1
end

set -l moma_path "$HOME/.moma"
set -l moma_cache "$HOME/.moma/.cache"

set -l overlay_work_dir "$moma_path/.overlay/work"
set -l overlay_lower_dir "$HOME/.steam/steam/steamapps/common/Skyrim Special Edition"
set -l overlay_upper_dir "$moma_path/.overlay/mods-merged"

set -l active_dir "$moma_path/active"

set -l skse_url (get_latest_skse_url)
set -l skse_archive "$moma_path/skse.7z"

rm -rf $overlay_upper_dir $overlay_work_dir

mkdir -p $overlay_work_dir $overlay_lower_dir $active_dir $moma_path/mods $moma_cache $moma_path/mods/skse $overlay_upper_dir $overlay_work_dir

# Download SKSE if not present
if not test -d "$moma_path/mods/skse"
    set -l full_version (string match -r 'skse64_[0-9_]+(?=\.7z)' -- $skse_url)

    curl -s $skse_url -o $moma_cache/skse.7z
    7z x $moma_cache/skse.7z -o$moma_path/mods/skse &>/dev/null

    mv $moma_path/mods/skse/$full_version/* $moma_path/mods/skse
    rm -rf $moma_path/mods/skse/$full_version
end

# Copy mods into upper dir
for mod in $moma_path/mods/*

    if test -d "$mod"
        cp -rn "$mod/"* $overlay_upper_dir
    end
end

if mount | grep -q "$active_dir"
    sudo umount $active_dir
end

sudo mount -t overlay overlay \
    -olowerdir=$overlay_lower_dir,upperdir=$overlay_upper_dir,workdir=$overlay_work_dir \
    $active_dir

set -l launcher_path "$HOME/.moma/active/launch-skyrim"
set -l proton "$HOME/.steam/steam/steamapps/common/Proton Hotfix/proton"
set -l skse_loader "$HOME/.moma/active/skse64_loader.exe"

echo "#!/usr/bin/env bash
export STEAM_COMPAT_DATA_PATH=\"\$HOME/.moma/.proton\"
export STEAM_COMPAT_CLIENT_INSTALL_PATH=\"\$HOME/.steam/steam\"

cd \"\$HOME/.moma/active\"
\"$proton\" run \"\$HOME/.moma/active/skse64_loader.exe\"" >$launcher_path

chmod +x $launcher_path

STEAM_COMPAT_DATA_PATH=/home/bas/.moma/proton STEAM_COMPAT_CLIENT_INSTALL_PATH=/home/bas/.steam/steam /home/bas/.local/share/Steam/steamapps/common/Proton\ Hotfix/proton run /home/bas/.moma/skyrim/active/skse64_loader.exe
