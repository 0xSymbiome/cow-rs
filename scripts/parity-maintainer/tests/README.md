# parity-maintainer Tests

Source-root validation tests live in `src/main.rs` so they can exercise internal
validation helpers directly. Keep black-box CLI tests in this directory only when
they do not duplicate the unit coverage.
