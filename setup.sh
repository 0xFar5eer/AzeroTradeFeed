apt-get update; apt-get upgrade -y; apt-get dist-upgrade; apt-get install -y sudo curl
sudo useradd user
sudo usermod -aG sudo user; 
sudo hostnamectl set-hostname va0


sudo cat > ~/.profile <<EOL
# ~/.profile: executed by Bourne-compatible login shells.

if [ "$BASH" ]; then
  if [ -f ~/.bashrc ]; then
    . ~/.bashrc
  fi
fi

mesg n || true
EOL



sudo cat > ~/.bashrc <<'EOL'
# ~/.bashrc: executed by bash(1) for non-login shells.

# Note: PS1 and umask are already set in /etc/profile. You should not
# need this unless you want different defaults for root.
# PS1='${debian_chroot:+($debian_chroot)}\h:\w\$ '
# umask 022

# You may uncomment the following lines if you want \`ls' to be colorized:
# export LS_OPTIONS='--color=auto'
# eval "\`dircolors\`"
# alias ls='ls $LS_OPTIONS'
# alias ll='ls $LS_OPTIONS -l'
# alias l='ls $LS_OPTIONS -lA'
#
# Some more alias to avoid making mistakes:
# alias rm='rm -i'
# alias cp='cp -i'
# alias mv='mv -i'
case $- in
    *i*) ;;
      *) return;;
esac

# don't put duplicate lines or lines starting with space in the history.
# See bash(1) for more options
HISTCONTROL=ignoreboth

# append to the history file, don't overwrite it
shopt -s histappend

# for setting history length see HISTSIZE and HISTFILESIZE in bash(1)
HISTFILESIZE=99999999
HISTSIZE=99999999

# check the window size after each command and, if necessary,
# update the values of LINES and COLUMNS.
shopt -s checkwinsize

# If set, the pattern "**" used in a pathname expansion context will
# match all files and zero or more directories and subdirectories.
#shopt -s globstar

# make less more friendly for non-text input files, see lesspipe(1)
#[ -x /usr/bin/lesspipe ] && eval "$(SHELL=/bin/sh lesspipe)"

# set variable identifying the chroot you work in (used in the prompt below)
if [ -z "${debian_chroot:-}" ] && [ -r /etc/debian_chroot ]; then
    debian_chroot=$(cat /etc/debian_chroot)
fi

# set a fancy prompt (non-color, unless we know we "want" color)
case "$TERM" in
    xterm-color) color_prompt=yes;;
esac

# uncomment for a colored prompt, if the terminal has the capability; turned
# off by default to not distract the user: the focus in a terminal window
# should be on the output of commands, not on the prompt
force_color_prompt=yes

if [ -n "$force_color_prompt" ]; then
    if [ -x /usr/bin/tput ] && tput setaf 1 >&/dev/null; then
# We have color support; assume it's compliant with Ecma-48
# (ISO/IEC-6429). (Lack of such support is extremely rare, and such
# a case would tend to support setf rather than setaf.)
color_prompt=yes
    else
color_prompt=
    fi
fi

if [ "$color_prompt" = yes ]; then
    PS1='${debian_chroot:+($debian_chroot)}\[\033[01;31m\]\u\[\033[01;33m\]@\[\033[01;36m\]\h \[\033[01;33m\]\w \[\033[01;35m\]\$ \[\033[00m\]'
else
    PS1='${debian_chroot:+($debian_chroot)}\u@\h:\w\$ '
fi
unset color_prompt force_color_prompt

# If this is an xterm set the title to user@host:dir
case "$TERM" in
xterm*|rxvt*)
    PS1="\[\e]0;${debian_chroot:+($debian_chroot)}\u@\h: \w\a\]$PS1"
    ;;
*)
    ;;
esac

# enable color support of ls and also add handy aliases
if [ -x /usr/bin/dircolors ]; then
    test -r ~/.dircolors && eval "$(dircolors -b ~/.dircolors)" || eval "$(dircolors -b)"
    alias ls='ls --color=auto'
    #alias dir='dir --color=auto'
    #alias vdir='vdir --color=auto'

    #alias grep='grep --color=auto'
    #alias fgrep='fgrep --color=auto'
    #alias egrep='egrep --color=auto'
fi

# colored GCC warnings and errors
#export GCC_COLORS='error=01;31:warning=01;35:note=01;36:caret=01;32:locus=01:quote=01'

# some more ls aliases
#alias ll='ls -l'
#alias la='ls -A'
#alias l='ls -CF'

# Alias definitions.
# You may want to put all your additions into a separate file like
# ~/.bash_aliases, instead of adding them here directly.
# See /usr/share/doc/bash-doc/examples in the bash-doc package.

if [ -f ~/.bash_aliases ]; then
    . ~/.bash_aliases
fi

# enable programmable completion features (you don't need to enable
# this, if it's already enabled in /etc/bash.bashrc and /etc/profile
# sources /etc/bash.bashrc).
if ! shopt -oq posix; then
  if [ -f /usr/share/bash-completion/bash_completion ]; then
    . /usr/share/bash-completion/bash_completion
  elif [ -f /etc/bash_completion ]; then
    . /etc/bash_completion
  fi
fi
export TZ=Europe/Moscow
ulimit -n 256000

cd ~/aleph-node-runner
EOL


sudo cat >> /etc/security/limits.conf <<EOL
* soft     nproc          256000
* hard     nproc          256000
* soft     nofile         256000
* hard     nofile         256000
root soft     nproc          256000
root hard     nproc          256000
root soft     nofile         256000
root hard     nofile         256000
EOL

sudo cat >> /etc/pam.d/common-session <<EOL
session required pam_limits.so
EOL

sudo cat > /etc/sysctl.conf << EOL
fs.file-max = 1000000
vm.swappiness = 10
vm.dirty_ratio = 60
vm.dirty_background_ratio = 2
net.ipv4.tcp_synack_retries = 2
net.ipv4.ip_local_port_range = 2000 65535
net.ipv4.tcp_rfc1337 = 1
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 5
net.ipv4.tcp_keepalive_intvl = 15
net.core.rmem_default = 31457280
net.core.rmem_max = 12582912
net.core.wmem_default = 31457280
net.core.wmem_max = 12582912
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.core.optmem_max = 25165824
net.ipv4.tcp_mem = 65535 131072 262144
net.ipv4.udp_mem = 65535 131072 262144
net.ipv4.tcp_rmem = 8192 87380 16777216
net.ipv4.udp_rmem_min = 16384
net.ipv4.tcp_wmem = 8192 65535 16777216
net.ipv4.udp_wmem_min = 16384
net.netfilter.nf_conntrack_max = 1548576
net.nf_conntrack_max = 1548576
net.ipv4.tcp_max_tw_buckets = 1440000
net.ipv4.tcp_tw_recycle = 1
net.ipv4.tcp_tw_reuse = 1
net.ipv6.conf.all.disable_ipv6 = 1
net.ipv6.conf.default.disable_ipv6 = 1
net.ipv6.conf.lo.disable_ipv6 = 1
fs.file-max = 1000000
fs.inotify.max_user_watches=1048576
EOL

sudo sysctl -p

sudo apt-get update; sudo apt-get dist-upgrade -y; sudo apt-get upgrade -y; sudo apt-get install -y sudo nano screen unzip wget iotop atop htop bind9 bind9utils bind9-doc pkg-config libssl-dev


sudo service bind9 start; sudo mv /etc/resolv.conf /etc/_resolv.conf; sudo cat > /etc/resolv.conf <<EOL
#nameserver 127.0.0.1

nameserver 1.1.1.1
nameserver 2001:4860:4860::8844
nameserver 8.8.8.8
nameserver 2606:4700:4700::1111
EOL

# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup toolchain install nightly
rustup component add clippy
cargo install cargo-chef cargo-udeps cargo-audit cargo-edit


# docker & docker-compose
sudo apt update; sudo apt install -y apt-transport-https ca-certificates curl gnupg mlocate; updatedb; curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker.gpg; echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker.gpg] https://download.docker.com/linux/debian bookworm stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null; sudo apt update; sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin; 
sudo curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose; sudo chmod +x /usr/local/bin/docker-compose; 
sudo usermod -aG docker user; 

# github runner
#cd ~; mkdir actions-runner && cd actions-runner; curl -o actions-runner-linux-x64-2.309.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.309.0/actions-runner-linux-x64-2.309.0.tar.gz; tar xzf ./actions-runner-linux-x64-2.309.0.tar.gz; 
#./config.sh --url https://github.com/0xFar5eer/AzeroTradeFeed --token XXXXXXX
#sudo ./svc.sh install; sudo ./svc.sh start

# tig stack
cd ~; git clone https://github.com/0xFar5eer/tig-stack/; cd tig-stack; sudo cat > ~/tig-stack/telegraf/telegraf.conf.template <<EOL
[agent]
  hostname = "$host" # set this to a name you want to identify your node in the grafana dashboard
  flush_interval = "5s"
  interval = "5s"
# Input Plugins
[[inputs.cpu]]
    percpu = true
    totalcpu = true
    collect_cpu_time = false
    report_active = false
[[inputs.disk]]
    ignore_fs = ["devtmpfs", "devfs"]
[[inputs.io]]
[[inputs.mem]]
[[inputs.net]]
[[inputs.system]]
[[inputs.swap]]
[[inputs.netstat]]
[[inputs.processes]]
[[inputs.kernel]]
[[inputs.diskio]]
[[inputs.prometheus]]
  urls = ["http://localhost:9615"]
# Output Plugin InfluxDB
[[outputs.influxdb]]
  database = "azeromainnet"
  urls = [ "https://stats.stakingbridge.com:8086" ]
  username = "azeromainnet"
  password = "azeromainnetpassword"
  insecure_skip_verify = true
EOL
docker-compose up -d


sudo cat > mycron <<EOL
# Restart Azero node
* * * * *       sleep  0; cd ~/aleph-node-runner; ./docker_block_watcher.sh
* * * * *       sleep 10; cd ~/aleph-node-runner; ./docker_block_watcher.sh
* * * * *       sleep 20; cd ~/aleph-node-runner; ./docker_block_watcher.sh
* * * * *       sleep 30; cd ~/aleph-node-runner; ./docker_block_watcher.sh
* * * * *       sleep 40; cd ~/aleph-node-runner; ./docker_block_watcher.sh
* * * * *       sleep 50; cd ~/aleph-node-runner; ./docker_block_watcher.sh
EOL
crontab mycron
rm mycron



# sh scripts
host=$(hostname)
sudo cat > ~/aleph-node-runner/docker_block_watcher.sh <<EOL
#!/bin/bash

# Modified script from paulormart (Turboflakes)
# https://gist.github.com/paulormart

# Bash script to be executed in the remote server to monitor block drift
#
# > Make a file executable
# chmod +x ./substrate_block_watcher_docker.sh
#
# > Positional arguments:
# 1st - blocks threshold
# 2nd - node RPC port
# 3rd - docker name
#
# > Test and run with the following parameters e.g.:
# ./substrate_block_watcher.sh 20 9944 docker-container-name
#
# > Schedule a cron job to execute every minute
# https://www.digitalocean.com/community/tutorials/how-to-use-cron-to-automate-tasks-ubuntu-1804
#
# example:
# * * * * * /opt/substrate_block_watcher_docker/substrate_block_watcher_docker.sh 20 docker-node-service 9944 >> /opt/substrate_block_watcher_docker/block-watcher.log
#

# Add a variable to store the previous block number file path
PREVIOUS_BLOCK_FILE="previous_block_number.txt"

DOCKER_CONTAINER="$host"
BLOCKS_THRESHOLD=200
RPC_PORT=9933

# Verify if Docker container is running
CONTAINER_STATUS=\$(docker inspect --format="{{.State.Status}}" \$DOCKER_CONTAINER)

if [ "\$CONTAINER_STATUS" != "running" ];
then
    echo "ERROR: Docker container \$DOCKER_CONTAINER is not running."
    exit
fi

# Verify if node is running on the RPC PORT specified
STATUS_CODE=\$(curl --write-out %{http_code} --silent --output /dev/null \
  -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
  'http://localhost:'\$RPC_PORT'')

if [[ "\$STATUS_CODE" -ne 200 ]];
then
    echo "ERROR: RPC port: \$RPC_PORT fails to connect."
    exit
fi

# --- Fetch node health
# NOTE: system_health response example:
# {
#  "isSyncing": false,
#  "peers": 37,
#  "shouldHavePeers": true
# }
IS_SYNCING="\$( curl --silent -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
  'http://localhost:'\$RPC_PORT'' \
  | jq '.result.isSyncing' )"

# ---
# NOTE: Skip Monitoring if node is syncing old blocks
#
if [ "\$IS_SYNCING" = true ]
then
  echo "INFO: Node is syncing -> SKIPPING monitor."
  exit
fi

# --- Fetch RPC \`system_syncState\`
# NOTE: system_health response example:
# {
#   "currentBlock": 11132625,
#   "highestBlock": 11132625,
#   "startingBlock": 10862594
# }
CURRENT_BLOCK_NUMBER="\$( curl --silent -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_syncState", "params":[]}' \
  'http://localhost:'\$RPC_PORT'' \
  | jq '.result.currentBlock' )"
# ---

# Read the previous block number from the file, if it exists
if [ -f "\$PREVIOUS_BLOCK_FILE" ]; then
  PREVIOUS_BLOCK_NUMBER=\$(cat \$PREVIOUS_BLOCK_FILE)
else
  PREVIOUS_BLOCK_NUMBER=-1
fi

# Store the current block number in the file for the next execution
echo \$CURRENT_BLOCK_NUMBER > \$PREVIOUS_BLOCK_FILE

# --- Fetch Finalized block number
# Get Finalized head
BLOCK_HASH="\$( curl --silent -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getFinalizedHead", "params":[]}' \
  'http://localhost:'\$RPC_PORT'' \
  | jq '.result' )"
BLOCK_HASH=\$( echo "\$BLOCK_HASH" | awk -F ' ' '{ printf \$1 }' )

# Get Header
FINALIZED_BLOCK_NUMBER="\$( curl --silent -H Content-Type:application/json \
  -d '{"id":1, "jsonrpc": "2.0", "method": "chain_getHeader", "params": ['\$BLOCK_HASH']}' \
  'http://localhost:'\$RPC_PORT'' \
  | jq '.result.number' )"
# Note: To convert hex block number decimal
# we first need to emove "" and 0x from heximal number eg: "0xaa1047" -> aa1047
FINALIZED_BLOCK_NUMBER=\${FINALIZED_BLOCK_NUMBER//\"/}
FINALIZED_BLOCK_NUMBER=\${FINALIZED_BLOCK_NUMBER//0x/}
# Convert block number hex to decimal
FINALIZED_BLOCK_NUMBER=\$(( 16#\$FINALIZED_BLOCK_NUMBER ))
BLOCK_DRIFT=\$(( \$CURRENT_BLOCK_NUMBER-\$FINALIZED_BLOCK_NUMBER ))
# ---
DATE=\$(date '+%Y-%m-%d %H:%M:%S')
echo "\$DATE [\$DOCKER_CONTAINER]: 🧱 Current Block : (\$CURRENT_BLOCK_NUMBER) | 📏 Block drift (\$BLOCK_DRIFT) 👀"

if [ "\$BLOCK_DRIFT" -gt "\$BLOCKS_THRESHOLD" ] || [ "\$CURRENT_BLOCK_NUMBER" -eq "\$PREVIOUS_BLOCK_NUMBER" ]
then
  # restart container
  echo "\$DATE [\$DOCKER_CONTAINER] ⚡ RESTARTING \$DOCKER_CONTAINER ⚡ "
  docker restart \$DOCKER_CONTAINER
fi
EOL

sudo cat > ~/aleph-node-runner/logs.sh <<EOL
container_id=\`docker ps | grep $host | cut -f1 -d " "\`
docker logs -f --since=1m \$container_id
EOL

ip=$(hostname  -I | cut -f1 -d' ')
sudo cat > ~/aleph-node-runner/update.sh <<EOL
container_id=\`docker ps | grep $host | cut -f1 -d " "\`
docker stop \$container_id
yes y | ./run_node.sh -n $host --ip $ip --mainnet
EOL


sudo cat > ~/aleph-node-runner/restart.sh <<EOL
container_id=\`docker ps | grep $host | cut -f1 -d " "\`
docker stop \$container_id
docker restart $container_id
EOL


chmod +x ~/aleph-node-runner/*.sh