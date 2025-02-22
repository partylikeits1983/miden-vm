# Miden stdlib
Standard library for Miden VM.

Miden standard library provides a set of procedures which can be used by any Miden program. These procedures build on the core instruction set of [Miden assembly](../assembly) expanding the functionality immediately available to the user.

The goals of Miden standard library are:
* Provide highly-optimized and battle-tested implementations of commonly-used primitives.
* Reduce the amount of code that needs to be shared between parties for proving and verifying program execution.

The second goal can be achieved because calls to procedures in the standard library can always be serialized as 32 bytes, regardless of how large the procedure is.

## Available modules
Currently, Miden standard library contains just a few modules, which are listed below. Over time, we plan to add many more modules which will include various cryptographic primitives, additional numeric data types and operations, and many others.

- [std::crypto::hashes::blake3](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/crypto/hashes/blake3.masm)
- [std::crypto::hashes::keccak256](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/crypto/hashes/keccak256.masm)
- [std::crypto::hashes::sha256](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/crypto/hashes/sha256.masm)
- [std::crypto::fri::frie2f4](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/crypto/fri/frie2f4.masm)
- [std::math::u256](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/math/u256.masm)
- [std::math::u64](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/math/u64.masm)
- [std::math::secp256k1](https://github.com/0xPolygonMiden/miden-vm/tree/main/stdlib/asm/math/secp256k1)
- [std::mem](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/mem.masm)
- [std::sys](https://github.com/0xPolygonMiden/miden-vm/blob/main/stdlib/asm/sys.masm)

## Status
At this point, all implementations listed above are considered to be experimental and are subject to change.

## License
This project is [MIT licensed](../LICENSE).
