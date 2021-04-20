# RPI4-Kernel-Test
A small test program for stable-rc kernel release on RPI4. 
It's triggered by emails in maillist(stable@vger.kernel.org), fetches source code from the repo then builds and reboots the kernel to see if there is any problem.

## Getting Started

This is an example of how you may give instructions on setting up your project locally.
To get a local copy up and running follow these simple example steps.

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
