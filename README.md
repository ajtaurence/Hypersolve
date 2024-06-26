# About
Hypersolve is an efficient 3-phase [2<sup>4</sup> Rubik's cube](https://hypercubing.xyz/puzzles/2x2x2x2/) solver written in rust. Currently it is being developed as a command line application but may also have a gui built for it later. Hypersolve is capable of solving random state scrambles in around 21-24 moves STM within a few seconds! It can also generate true random state scrambles, not relying on sufficiently long random sequences of moves. These random state scrambles come with a verification key that can be used to prove that the scrambles came from Hypersolve and are untampered, making scramble cheating impossible. For a full list of features see [commands](#commands). For build instructions see [building](#building).


# Commands

## fast-solve
Finds relatively short solutions very quickly by iteratively searching for solutions shorter than the last one, terminating with the optimal solution

## optimal-solve
Finds the shortest possible solutions in order of increasing length

## bound
Computes bounds on the length of the optimal solution to a scramble

## scramble
Generates a true random state scramble with a 128 bit hexadecimal verification key for [verifying](#verify-scramble) that the scramble was randomly generated and untampered

## verify-scramble
Verifies that a scramble is a true random state scramble generated by Hypersolve using the verification key

## invert
Computes the inverse of a move sequence

## convert
Converts the moves to the given notation

# Hardware Details

## Ram
Hypersolve will use about 1.5 GB of ram.

## Disk Space
Hypersolve requires about 1.5 GB of disk space to store several lookup tables which are computed the first time they are needed.


# Verification Details
Hypersolve verification works by producing (or receiving) a random 128 bit key when a scramble is requested. The key is then hashed using the [SHA256](https://en.wikipedia.org/wiki/SHA-2) hashing algorithm. The first 128 bits are kept and the rest are discarded. The hashed key is converted into an index ranging from 0 to 3,357,894,533,384,932,272,635,904,000 which uniquely identifies a 2<sup>4</sup> state. Now that a 2<sup>4</sup> state has been selected, Hypersolve quickly finds a sequence which solves that state. It then inverts that solution to obtain the scramble which results in that state.

During the verification process, the verification key is provided along with the scramble. Hypersolve simply re-generates the scramble from the key and ensures that the generated scramble matches the provided scramble.

This process ensures that it is essentially impossible to force Hypersolve to output a scramble for a particular desired state since this would require reversing the hash function. Additionally, one cannot pass off the scramble as their solution and their solution as the scramble by inverting them since the scramble must not just result in the correct state, but have the exact expected form.

# Building

1. Install Rust / Cargo
1. Run

```sh
git clone https://github.com/ajtaurence/Hypersolve.git
cd Hypersolve
cargo build --release
```

