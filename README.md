# rust-mqtt-pi
Rust MQTT Client for Raspberry PI

## Build for ARM target (armv6) i.e. rpi-1

1) `git clone https://github.com/raspberrypi/tools.git`

2) `export PATH="~/foo/raspi/tools/arm-bcm2708/arm-rpi-4.9.3-linux-gnueabihf/bin:$PATH"`

3) Edit your cargo config : 
   `sudo nano ~/.cargo/config`
   Paste this into the file:
   `[target.arm-uknown-linux-gnueabihf]`
   `linker=arm-linux-gnueabihf-gcc`
   Save file : Ctrl + O, Ctrl + X
   
4) Add rust target for arm :
   `rustup add target arm-unknown-linux-gnueabihf`
   
5) Build for arm target : 
   `cargo build --target arm-unknown-linux-gnueabihf`
   
6) Copy `/../foo/<project name>/target/arm-unknown-linux-gnueabihf/debug/<project name>` to rpi


7) Execute
