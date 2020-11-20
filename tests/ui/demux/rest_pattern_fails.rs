use mux_stream_macros::demux;

fn main() {
    demux!(MyEnum { A, .., B, C });
    demux!(MyEnum { A, B, C, .. });
    demux!(MyEnum { .., A, B, C });

    demux!(MyEnum { A, B, C, .. });
    demux!(MyEnum { A, B, C  .. });

    demux!(MyEnum { .., A, B, C, .. });
    demux!(MyEnum { ..  A, B, C, .. });

    demux!(MyEnum { A, ..  B, C, .. });
    demux!(MyEnum { A, .., B, C, .. });

    demux!(MyEnum { .. A, B, C });
    demux!(MyEnum .. { A, B, C });
    demux!(MyEnum { A, B, C } ..);
}
