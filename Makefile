.PHONY: cover
cover:
	@rm -rf target/debug/deps/*.gcda
	@CARGO_INCREMENTAL=0 \
		RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" \
		RUSTDOCFLAGS="-Cpanic=abort" \
		LLVM_PROFILE_FILE="noter-%p-%m.profraw" \
		cargo test --lib
	@grcov . \
		--binary-path ./target/debug/ \
		-s . \
		-t html \
		--branch \
		--ignore-not-existing \
		-o ./coverage
