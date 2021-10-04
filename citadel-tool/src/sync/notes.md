
### Old Sync

#### citadel-current-watcher.path

    [Path]                                                                                                                                                                                                               
    PathChanged=/run/citadel/realms/current

#### citadel-current-watcher.service

    [Service]
    Type=oneshot
    ExecStart=/usr/libexec/citadel-desktop-sync --clear
    ExecStart=/usr/bin/systemctl restart citadel-desktop-watcher.path

#### citadel-desktop-watcher.path

    [Path]
    PathChanged=/run/citadel/realms/current/current.realm/rootfs/usr/share/applications
    PathChanged=/run/citadel/realms/current/current.realm/home/.local/share/applications

#### citadel-desktop-watcher.service

    [Service]
    Type=oneshot
    ExecStart=/usr/libexec/citadel-desktop-sync

### New Sync

* Added a new command line option `--all` for syncronizing all active realms
