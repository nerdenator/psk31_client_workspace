CARGO_MANIFEST := src-tauri/Cargo.toml

# Force recompile + launch dev server with mock radio + debug logging
# Use this to test radio CAT commands without hardware attached.
dev-mock:
	touch src-tauri/src/lib.rs && MOCK_RADIO=1 RUST_LOG=baudacious_lib=debug npm run tauri dev

# Force recompile + normal dev server (real hardware)
dev:
	touch src-tauri/src/lib.rs && npm run tauri dev

# Run all Rust tests
test:
	cargo test --manifest-path $(CARGO_MANIFEST)

# Run Playwright E2E tests
test-e2e:
	npm test

# Rust type-check only (fast, no binary produced)
check:
	cargo check --manifest-path $(CARGO_MANIFEST)

.PHONY: dev dev-mock test test-e2e check
