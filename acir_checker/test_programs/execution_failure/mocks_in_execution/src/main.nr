fn main() {
    // Trying to use a mock in `nargo execute` should fail.
    // Safety: test
    let mock = unsafe { std::test::OracleMock::mock("foo") };
    // assert_eq(mock.id, 0);
}
