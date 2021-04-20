# RPI4-Kernel-Test
A small test program for stable-rc kernel release on RPI4. 
It's triggered by emails in maillist(stable@vger.kernel.org), fetches source code from the repo then builds and reboots the kernel to see if there is any problem.

## Getting Started

The program comprises two parts: `controller` and `worker`. The controller polling email box regularly dispatches testing tasks to the worker and also monitoring the health of the worker. The worker on the other hand receiving tasks from the controller, executes the runner.sh to fetch, build, and boot specified linux kernels, then reports results back to the controller.

### Installation

1. Install Rust
2. Clone the repo
   ```sh
   git clone https://github.com/foxhlchen/RPI4-Kernel-Test.git
   ```
3. Build and Install 
   ```sh
   cargo build
   cargo install
   ```
4. Configurate `setting.toml` (see config section)
5. Start the controller
   ```sh
   ./controller
   ```
6. Start the worker
   ```sh
   ./worker
   ```
   
### Config

The controller and worker will read settings.toml from its working directory when started.

1. controller's `settings.toml`:
   ```toml
   [imap]
   domain = "imap.gmail.com"
   username = "myaddr@gmail.com"
   password = "mypasswd"
   mailbox = "MyBox/Linux/Stable"

   [smtp]
   domain = "smtp.gmail.com"
   username = "myaddr@gmail.com"
   password = "mypasswd"
   from = "My Name <myaddr@gmail.com>"

   [rpc]
   addr = "[::]:9999"
   taskcache = "task.cache"

   [log]
   conf_path = "log4rs.yaml"

   ```
2. worker's `settings.toml`:
   ```toml
   [rpc]
   addr = "[::]:9999"
   taskcache = "task.cache"

   [log]
   conf_path = "log4rs.yaml"

   [execute]
   runner = "runner.sh"   
   ```
3. log4rs.yaml:  
   see [log4rs](https://docs.rs/log4rs)
