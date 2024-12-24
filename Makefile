all:
	(cd lunacore_compiler && cargo build --release)
	cp lunacore_compiler/target/release/compiler .

	(cd lunacore_emulator && cargo build --release)
	cp lunacore_emulator/target/release/emulator .
