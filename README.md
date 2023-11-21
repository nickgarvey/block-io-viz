# block-io-viz

Visualization of reads & writes to a block device.

It does this by loading an eBPF program to monitor disk I/O, and then serves that data to a frontend via a websocket.

Run with: `RUST_LOG=debug cargo xtask run`

Work in progress.

![block-io-viz demo gif](https://raw.githubusercontent.com/nickgarvey/block-io-viz/main/block-io-viz-demo.gif)
