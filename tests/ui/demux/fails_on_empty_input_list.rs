use mux_stream_macros::demux;

fn main() {
    demux!();
    demux!(MyEnum {});
}
