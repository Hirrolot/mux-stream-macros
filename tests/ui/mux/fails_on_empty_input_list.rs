use mux_stream_macros::mux;

fn main() {
    mux!();
    mux!(MyEnum {});
}
