#[test]
fn test() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/mux/works.rs");
    t.pass("tests/ui/mux/enum_path_works.rs");
    t.pass("tests/ui/mux/input_streams_are_independent.rs");
    t.compile_fail("tests/ui/mux/fails_on_empty_input_list.rs");

    t.pass("tests/ui/demux/works.rs");
    t.pass("tests/ui/demux/enum_path_works.rs");
    t.pass("tests/ui/demux/rest_pattern.rs");
    t.pass("tests/ui/demux/output_streams_are_independent.rs");
    t.compile_fail("tests/ui/demux/fails_on_empty_input_list.rs");
    t.compile_fail("tests/ui/demux/non_exhaustive_fails.rs");
    t.compile_fail("tests/ui/demux/rest_pattern_fails.rs");
}
