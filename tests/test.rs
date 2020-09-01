#[test]
fn test() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/mux/works.rs");
    t.compile_fail("tests/ui/mux/fails_on_empty_input_streams.rs");

    t.pass("tests/ui/demux/works.rs");
    t.compile_fail("tests/ui/demux/fails_on_empty_arms.rs");
}
