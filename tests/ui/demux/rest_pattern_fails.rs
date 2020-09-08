use mux_stream_macros::demux;

fn main() {
    demux!(A, .., B, C);
    demux!(A, B, C,, ..);
    demux!(.., A, B, C);

    demux!(A, B, C, ..);
    demux!(A, B, C  ..);

    demux!(.., A, B, C, ..);
    demux!(..  A, B, C, ..);

    demux!(A, ..  B, C, ..);
    demux!(A, .., B, C, ..);
}
