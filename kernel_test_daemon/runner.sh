#!/bin/bash
# MUST use __absolute__ path
LOGPATH=`pwd`/logs
LOGFILE=${LOGPATH}/runner.`date +%Y-%m-%d`.log
REPO_DIR=/home/fox/Workspace/linux_src/linux-stable-rc
PW="qwerty"
TASK_CACHE=`pwd`/task.cache

# Create log folder
mkdir $LOGPATH &> /dev/null

# Fetch args
if [ "$#" -lt 4 ]; then
    echo "args error $@" >> $LOGFILE
    >&2 echo "args error $@"
    exit -1
fi

state=$1
task_id=$2
ver=$3
branch=$4

update_result () {
    if [[ `uname -r` != *"$ver"* ]]; then
        echo "Build failed." >> $LOGFILE
        >&2 echo "Build failed"
        exit -101    
    fi

    echo "$ver Suceeded!"
}

build_kernel () {
    # Switch to repo folder
    echo "=== Switch to $REPO_DIR" >> $LOGFILE
    cd $REPO_DIR
    make clean

    # Update code
    echo "=== Update repo" >> $LOGFILE
    git remote update >> $LOGFILE 2>&1
    if [ "$?" -ne 0 ]; then
        echo "update repo failed." >> $LOGFILE
        >&2 echo "update repo failed"
        exit -2
    fi

    # Switch repo branch
    echo "=== Checkout branch remotes/origin/$branch" >> $LOGFILE
    git checkout remotes/origin/$branch >> $LOGFILE 2>&1
    if [ "$?" -ne 0 ]; then
        echo "checkout branch failed." >> $LOGFILE
        >&2 echo "checkout branch failed"
        exit -3
    fi

    # Setup build
    echo "=== Setup Build" >> $LOGFILE
    make defconfig >> $LOGFILE 2>&1
    if [ "$?" -ne 0 ]; then
        echo "setup build failed." >> $LOGFILE
        >&2 echo "setup build failed"
        exit -4
    fi

    # Build kernel
    echo "=== Build kernel" >> $LOGFILE
    make -j `nproc` >> $LOGFILE 2>&1
    if [ "$?" -ne 0 ]; then
        echo "build failed." >> $LOGFILE
        >&2 echo "build failed"
        exit -5
    fi

    # Install kernel
    echo "=== Install kernel" >> $LOGFILE
    echo $PW | sudo -S make modules_install install >> $LOGFILE 2>&1
    if [ "$?" -ne 0 ]; then
        echo "Install kernel failed." >> $LOGFILE
        >&2 echo "Install kernel failed"
        exit -6
    fi

    # Update Task state cache
    echo "DONE" > $TASK_CACHE &&
    echo $task_id >> $TASK_CACHE &&
    echo $ver >> $TASK_CACHE &&
    echo $branch >> $TASK_CACHE

    # rm modules
    echo $PW | sudo -S rm -r /usr/lib/modules/`uname -r` >> $LOGFILE 2>&1
    if [ "$ver" == "`uname -r | cut -d'-' -f1`" ]; then
        echo "=== Reinstall modules" >> $LOGFILE
        echo $PW | sudo -S make modules_install >> $LOGFILE 2>&1
    fi

    # Reboot
    echo "=== Reboot" >> $LOGFILE
    echo $PW | sudo -S reboot >> $LOGFILE 2>&1
    if [ "$?" -ne 0 ]; then
        echo "Reboot failed." >> $LOGFILE
        >&2 echo "Reboot failed"
        exit -7
    fi    
}

if [ "$state" = "NEW" ]; then
    echo "=== New task $@" >> $LOGFILE
    build_kernel
else
    echo "=== Update task $@" >> $LOGFILE
    update_result
fi
