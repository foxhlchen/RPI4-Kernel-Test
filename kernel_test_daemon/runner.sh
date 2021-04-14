#!/bin/bash
LOGPATH=./logs
LOGFILE=${LOGPATH}/runner.`date +%Y-%m-%d`.log
REPO_DIR=/home/fox/Workspace/linux_src/linux-stable-rc
PW="qwerty"
TASK_CACHE="task.cache"

# Create log folder
mkdir $LOGPATH

# Fetch args
if [ "$#" -lt 4 ]; then
    echo "args error $@" >> LOGFILE
    >&2 echo "args error $@"
    exit -1
fi

state=$1
task_id=$2
ver=$3
branch=$4

update_result () {
    echo "Update task $@" >> LOGFILE
    if [ `uname -r` != "$ver" ]; then
        echo "Build failed." >> LOGFILE
        >&2 echo "Build failed"
        exit -101    
    fi


}

build_kernel () {
    echo "New task $@" >> LOGFILE

    # Switch to repo folder
    cd $REPO_DIR

    # Update code
    git remote update >> LOGFILE
    if [ "$?" -ne 0 ]; then
        echo "update repo failed." >> LOGFILE
        >&2 echo "update repo failed"
        exit -2
    fi

    # Switch repo branch
    git checkout remotes/origin/$branch
    if [ "$?" -ne 0 ]; then
        echo "checkout branch failed." >> LOGFILE
        >&2 echo "checkout branch failed"
        exit -3
    fi

    # Setup build
    make defconfig
    if [ "$?" -ne 0 ]; then
        echo "setup build failed." >> LOGFILE
        >&2 echo "setup build failed"
        exit -4
    fi

    # Build kernel
    make
    if [ "$?" -ne 0 ]; then
        echo "build failed." >> LOGFILE
        >&2 echo "build failed"
        exit -5
    fi

    # Install kernel
    echo $PW | sudo -S make modules_install install
    if [ "$?" -ne 0 ]; then
        echo "Install kernel failed." >> LOGFILE
        >&2 echo "Install kernel failed"
        exit -6
    fi

    # Update Task state cache
    echo "DONE" > $TASK_CACHE &&
    echo $task_id >> $TASK_CACHE &&
    echo $ver >> $TASK_CACHE &&
    echo $branch >> $TASK_CACHE

    # Reboot
    echo $PW | sudo -S reboot
    if [ "$?" -ne 0 ]; then
        echo "Reboot failed." >> LOGFILE
        >&2 echo "Reboot failed"
        exit -7
    fi    
}

if [ "$state" = "NEW" ]; then
    build_kernel
else
    update_result
fi
