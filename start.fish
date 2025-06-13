#!/usr/bin/env fish

set -x PROTON_LOG 1
set -x STEAM_COMPAT_DATA_PATH ~/.moma/.proton
set -x STEAM_COMPAT_CLIENT_INSTALL_PATH ~/.steam/steam

~/.steam/steam/steamapps/common/"Proton Hotfix"/proton run ~/.moma/active/skse64_loader.exe
